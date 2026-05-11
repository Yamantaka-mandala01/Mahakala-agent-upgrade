use crate::error::AppError;
use crate::memory::MemoryManager;
use crate::skills::{Skill, SkillCreate, SkillManager};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningExperience {
    pub id: String,
    pub task_description: String,
    pub tools_used: Vec<String>,
    pub skills_used: Vec<String>,
    pub success: bool,
    pub feedback: String,
    pub timestamp: i64,
    pub lessons_learned: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTemplate {
    pub name: String,
    pub description: String,
    pub category: String,
    pub tools_required: Vec<String>,
    pub steps: Vec<String>,
    pub prompts: Vec<String>,
}

pub struct LearningLoop {
    memory: Arc<MemoryManager>,
    skills: Arc<SkillManager>,
    experiences: Arc<Mutex<Vec<LearningExperience>>>,
    skill_templates: Arc<Mutex<Vec<SkillTemplate>>>,
}

impl LearningLoop {
    pub fn new(memory: Arc<MemoryManager>, skills: Arc<SkillManager>) -> Self {
        let templates = vec![
            SkillTemplate {
                name: "code_review_template".to_string(),
                description: "Automated code review skill template".to_string(),
                category: "devops".to_string(),
                tools_required: vec!["file_read".to_string(), "file_write".to_string()],
                steps: vec![
                    "Read the target file".to_string(),
                    "Analyze code structure and patterns".to_string(),
                    "Identify potential issues".to_string(),
                    "Generate review report".to_string(),
                    "Save review results".to_string(),
                ],
                prompts: vec![
                    "Please review the following code and provide feedback on: 1) Code quality 2) Potential bugs 3) Performance issues 4) Best practices".to_string(),
                ],
            },
            SkillTemplate {
                name: "data_analysis_template".to_string(),
                description: "Data analysis and visualization template".to_string(),
                category: "research".to_string(),
                tools_required: vec!["file_read".to_string(), "shell_exec".to_string()],
                steps: vec![
                    "Load data file".to_string(),
                    "Clean and preprocess data".to_string(),
                    "Perform statistical analysis".to_string(),
                    "Generate visualizations".to_string(),
                    "Write analysis report".to_string(),
                ],
                prompts: vec![
                    "Please analyze the following data and provide insights including: 1) Summary statistics 2) Trends 3) Anomalies 4) Recommendations".to_string(),
                ],
            },
        ];

        Self {
            memory,
            skills,
            experiences: Arc::new(Mutex::new(Vec::new())),
            skill_templates: Arc::new(Mutex::new(templates)),
        }
    }

    pub fn record_experience(&self, experience: LearningExperience) -> Result<(), AppError> {
        let mut experiences = self.experiences.lock();
        experiences.push(experience);
        Ok(())
    }

    pub fn analyze_experience(&self, experience: &LearningExperience) -> Result<String, AppError> {
        let lessons = if experience.success {
            format!(
                "Successfully completed task: {}\nTools used: {}\nSkills used: {}\nKey takeaway: The approach worked well and can be reused.",
                experience.task_description,
                experience.tools_used.join(", "),
                experience.skills_used.join(", ")
            )
        } else {
            format!(
                "Failed to complete task: {}\nTools used: {}\nSkills used: {}\nFeedback: {}\nKey takeaway: Need to improve the approach or use different tools.",
                experience.task_description,
                experience.tools_used.join(", "),
                experience.skills_used.join(", "),
                experience.feedback,
            )
        };

        self.memory.add_fact(&lessons, Some("learning"))?;
        Ok(lessons)
    }

    pub fn create_skill_from_experience(&self, experience: &LearningExperience) -> Result<Skill, AppError> {
        let skill_name = experience.task_description.to_lowercase().replace(" ", "_").replace(|c: char| !c.is_alphanumeric(), "");
        
        let template = self.skill_templates.lock()
            .iter()
            .find(|t| t.category == "general")
            .cloned()
            .unwrap_or_else(|| SkillTemplate {
                name: "general_template".to_string(),
                description: "General purpose skill template".to_string(),
                category: "general".to_string(),
                tools_required: vec![],
                steps: vec!["Execute task".to_string()],
                prompts: vec!["Please complete the task".to_string()],
            });

        let skill_create = SkillCreate {
            name: format!("auto_{}", skill_name),
            description: Some(format!("Auto-generated skill from experience: {}", experience.task_description)),
            category: Some("auto_generated".to_string()),
        };

        let skill = self.skills.add_custom_skill(skill_create)?;
        
        let lesson = format!(
            "Skill '{}' created from experience. Tools: {}, Steps: {}",
            skill.name,
            experience.tools_used.join(", "),
            template.steps.join(" -> ")
        );
        
        self.memory.add_fact(&lesson, Some("skill_creation"))?;
        
        Ok(skill)
    }

    pub fn get_learning_insights(&self) -> Result<Vec<String>, AppError> {
        let experiences = self.experiences.lock();
        let success_count = experiences.iter().filter(|e| e.success).count();
        let total = experiences.len();
        
        let mut insights = vec![
            format!("Total experiences: {}", total),
            format!("Success rate: {:.1}%", if total > 0 { (success_count as f64 / total as f64) * 100.0 } else { 0.0 }),
        ];

        let mut tool_usage: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        for exp in experiences.iter() {
            for tool in &exp.tools_used {
                *tool_usage.entry(tool.clone()).or_insert(0) += 1;
            }
        }

        let mut sorted_tools: Vec<_> = tool_usage.iter().collect();
        sorted_tools.sort_by(|a, b| b.1.cmp(a.1));
        
        for (tool, count) in sorted_tools.iter().take(5) {
            insights.push(format!("Most used tool: {} ({} times)", tool, count));
        }

        Ok(insights)
    }

    pub fn suggest_skill_improvements(&self) -> Result<Vec<String>, AppError> {
        let facts = self.memory.list_facts(Some("learning"), Some(50), None)?;
        let mut suggestions = vec![];

        for fact in facts.iter().take(10) {
            if fact.content.contains("Failed") || fact.content.contains("failed") {
                suggestions.push(format!("Review failed experience: {}", fact.content));
            }
        }

        if suggestions.is_empty() {
            suggestions.push("No improvements needed based on current learning data".to_string());
        }

        Ok(suggestions)
    }

    pub fn get_experiences(&self) -> Vec<LearningExperience> {
        self.experiences.lock().clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_record_experience() {
        let memory = Arc::new(MemoryManager::new(None).unwrap());
        let skills = Arc::new(SkillManager::new());
        let learning_loop = LearningLoop::new(memory, skills);

        let experience = LearningExperience {
            id: "test_1".to_string(),
            task_description: "Test task".to_string(),
            tools_used: vec!["file_read".to_string()],
            skills_used: vec![],
            success: true,
            feedback: "Worked well".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            lessons_learned: "Test lesson".to_string(),
        };

        let result = learning_loop.record_experience(experience);
        assert!(result.is_ok());
    }

    #[test]
    fn test_analyze_experience() {
        let memory = Arc::new(MemoryManager::new(None).unwrap());
        let skills = Arc::new(SkillManager::new());
        let learning_loop = LearningLoop::new(memory, skills);

        let experience = LearningExperience {
            id: "test_2".to_string(),
            task_description: "Analyze test".to_string(),
            tools_used: vec!["calculator".to_string()],
            skills_used: vec![],
            success: true,
            feedback: "Good".to_string(),
            timestamp: chrono::Utc::now().timestamp(),
            lessons_learned: "Analysis complete".to_string(),
        };

        let result = learning_loop.analyze_experience(&experience);
        assert!(result.is_ok());
    }
}
