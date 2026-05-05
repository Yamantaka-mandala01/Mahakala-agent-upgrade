use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLog {
    pub id: String,
    pub timestamp: i64,
    pub user_id: String,
    pub action: String,
    pub resource: String,
    pub resource_id: Option<String>,
    pub details: serde_json::Value,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub status: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub id: String,
    pub generated_at: i64,
    pub report_type: String,
    pub period_start: i64,
    pub period_end: i64,
    pub findings: Vec<ComplianceFinding>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceFinding {
    pub id: String,
    pub category: String,
    pub severity: String,
    pub description: String,
    pub recommendation: String,
    pub affected_resources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub max_log_entries: usize,
    pub retention_days: u32,
    pub enable_compliance_reports: bool,
    pub log_level: String,
}

impl Default for AuditConfig {
    fn default() -> Self {
        Self {
            max_log_entries: 100000,
            retention_days: 90,
            enable_compliance_reports: true,
            log_level: "info".to_string(),
        }
    }
}

pub struct AuditSystem {
    logs: Arc<Mutex<Vec<AuditLog>>>,
    reports: Arc<Mutex<Vec<ComplianceReport>>>,
    config: AuditConfig,
    counters: Arc<Mutex<HashMap<String, usize>>>,
}

impl AuditSystem {
    pub fn new(config: AuditConfig) -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
            reports: Arc::new(Mutex::new(Vec::new())),
            config,
            counters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn log_action(
        &self,
        user_id: String,
        action: String,
        resource: String,
        resource_id: Option<String>,
        details: serde_json::Value,
        ip_address: Option<String>,
        user_agent: Option<String>,
        status: String,
        severity: String,
    ) -> Result<String, AppError> {
        let id = uuid::Uuid::new_v4().to_string();
        let action_clone = action.clone();
        let log = AuditLog {
            id: id.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            user_id,
            action,
            resource,
            resource_id,
            details,
            ip_address,
            user_agent,
            status,
            severity,
        };

        let mut logs = self.logs.lock().map_err(|e| AppError::Internal(format!("Failed to lock logs: {}", e)))?;
        
        if logs.len() >= self.config.max_log_entries {
            logs.remove(0);
        }
        
        logs.push(log);

        let mut counters = self.counters.lock().map_err(|e| AppError::Internal(format!("Failed to lock counters: {}", e)))?;
        let counter = counters.entry(action_clone).or_insert(0);
        *counter += 1;

        Ok(id)
    }

    pub fn get_logs(&self, filters: Option<HashMap<String, String>>) -> Vec<AuditLog> {
        let logs = self.logs.lock().unwrap();
        
        match filters {
            Some(f) => {
                logs.iter().filter(|log| {
                    if let Some(user_id) = f.get("user_id") {
                        if log.user_id != *user_id {
                            return false;
                        }
                    }
                    if let Some(action) = f.get("action") {
                        if log.action != *action {
                            return false;
                        }
                    }
                    if let Some(resource) = f.get("resource") {
                        if log.resource != *resource {
                            return false;
                        }
                    }
                    if let Some(severity) = f.get("severity") {
                        if log.severity != *severity {
                            return false;
                        }
                    }
                    true
                }).cloned().collect()
            }
            None => logs.clone(),
        }
    }

    pub fn get_log(&self, log_id: &str) -> Result<AuditLog, AppError> {
        let logs = self.logs.lock().map_err(|e| AppError::Internal(format!("Failed to lock logs: {}", e)))?;
        logs.iter()
            .find(|log| log.id == log_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Audit log {} not found", log_id)))
    }

    pub fn delete_log(&self, log_id: &str) -> Result<(), AppError> {
        let mut logs = self.logs.lock().map_err(|e| AppError::Internal(format!("Failed to lock logs: {}", e)))?;
        let initial_len = logs.len();
        logs.retain(|log| log.id != log_id);
        
        if logs.len() == initial_len {
            Err(AppError::NotFound(format!("Audit log {} not found", log_id)))
        } else {
            Ok(())
        }
    }

    pub fn purge_old_logs(&self, days: u32) -> Result<usize, AppError> {
        let cutoff = chrono::Utc::now().timestamp() - (days as i64 * 24 * 60 * 60);
        let mut logs = self.logs.lock().map_err(|e| AppError::Internal(format!("Failed to lock logs: {}", e)))?;
        let initial_len = logs.len();
        logs.retain(|log| log.timestamp > cutoff);
        Ok(initial_len - logs.len())
    }

    pub fn generate_compliance_report(&self, report_type: String, period_start: i64, period_end: i64) -> Result<String, AppError> {
        if !self.config.enable_compliance_reports {
            return Err(AppError::Internal("Compliance reports are disabled".to_string()));
        }

        let logs = self.logs.lock().map_err(|e| AppError::Internal(format!("Failed to lock logs: {}", e)))?;
        let period_logs: Vec<&AuditLog> = logs.iter()
            .filter(|log| log.timestamp >= period_start && log.timestamp <= period_end)
            .collect();

        let mut findings = Vec::new();

        let failed_actions: Vec<&AuditLog> = period_logs.iter()
            .filter(|log| log.status == "failed" || log.status == "denied")
            .cloned()
            .collect();

        if !failed_actions.is_empty() {
            findings.push(ComplianceFinding {
                id: uuid::Uuid::new_v4().to_string(),
                category: "access_control".to_string(),
                severity: "high".to_string(),
                description: format!("{} failed or denied actions detected", failed_actions.len()),
                recommendation: "Review access control policies and user permissions".to_string(),
                affected_resources: failed_actions.iter()
                    .map(|log| log.resource.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect(),
            });
        }

        let high_severity: Vec<&AuditLog> = period_logs.iter()
            .filter(|log| log.severity == "high" || log.severity == "critical")
            .cloned()
            .collect();

        if !high_severity.is_empty() {
            findings.push(ComplianceFinding {
                id: uuid::Uuid::new_v4().to_string(),
                category: "security".to_string(),
                severity: "critical".to_string(),
                description: format!("{} high severity events detected", high_severity.len()),
                recommendation: "Investigate high severity events and implement additional security measures".to_string(),
                affected_resources: high_severity.iter()
                    .map(|log| log.resource.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect(),
            });
        }

        let unique_users: std::collections::HashSet<&str> = period_logs.iter()
            .map(|log| log.user_id.as_str())
            .collect();

        if unique_users.len() > 50 {
            findings.push(ComplianceFinding {
                id: uuid::Uuid::new_v4().to_string(),
                category: "user_activity".to_string(),
                severity: "medium".to_string(),
                description: format!("Unusual number of active users: {}", unique_users.len()),
                recommendation: "Review user activity patterns and verify all users are authorized".to_string(),
                affected_resources: unique_users.into_iter().map(|s| s.to_string()).collect(),
            });
        }

        let id = uuid::Uuid::new_v4().to_string();
        let findings_len = findings.len();
        let report = ComplianceReport {
            id: id.clone(),
            generated_at: chrono::Utc::now().timestamp(),
            report_type,
            period_start,
            period_end,
            findings,
            summary: format!("Report generated with {} findings from {} log entries", findings_len, period_logs.len()),
        };

        drop(logs);
        let mut reports = self.reports.lock().map_err(|e| AppError::Internal(format!("Failed to lock reports: {}", e)))?;
        reports.push(report);

        Ok(id)
    }

    pub fn get_reports(&self) -> Vec<ComplianceReport> {
        let reports = self.reports.lock().unwrap();
        reports.clone()
    }

    pub fn get_report(&self, report_id: &str) -> Result<ComplianceReport, AppError> {
        let reports = self.reports.lock().map_err(|e| AppError::Internal(format!("Failed to lock reports: {}", e)))?;
        reports.iter()
            .find(|report| report.id == report_id)
            .cloned()
            .ok_or_else(|| AppError::NotFound(format!("Compliance report {} not found", report_id)))
    }

    pub fn get_stats(&self) -> serde_json::Value {
        let logs = self.logs.lock().unwrap();
        let reports = self.reports.lock().unwrap();
        let counters = self.counters.lock().unwrap();

        let severity_counts: HashMap<String, usize> = logs.iter()
            .fold(HashMap::new(), |mut acc, log| {
                *acc.entry(log.severity.clone()).or_insert(0) += 1;
                acc
            });

        let status_counts: HashMap<String, usize> = logs.iter()
            .fold(HashMap::new(), |mut acc, log| {
                *acc.entry(log.status.clone()).or_insert(0) += 1;
                acc
            });

        let resource_counts: HashMap<String, usize> = logs.iter()
            .fold(HashMap::new(), |mut acc, log| {
                *acc.entry(log.resource.clone()).or_insert(0) += 1;
                acc
            });

        serde_json::json!({
            "total_logs": logs.len(),
            "total_reports": reports.len(),
            "severity_counts": severity_counts,
            "status_counts": status_counts,
            "resource_counts": resource_counts,
            "action_counts": *counters,
            "retention_days": self.config.retention_days,
            "max_log_entries": self.config.max_log_entries
        })
    }

    pub fn export_logs(&self, format: &str) -> Result<String, AppError> {
        let logs = self.logs.lock().map_err(|e| AppError::Internal(format!("Failed to lock logs: {}", e)))?;
        
        match format {
            "json" => {
                serde_json::to_string_pretty(&*logs)
                    .map_err(|e| AppError::Internal(format!("Failed to serialize logs: {}", e)))
            }
            "csv" => {
                let mut csv = String::from("id,timestamp,user_id,action,resource,resource_id,status,severity\n");
                for log in logs.iter() {
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{},{}\n",
                        log.id,
                        log.timestamp,
                        log.user_id,
                        log.action,
                        log.resource,
                        log.resource_id.as_deref().unwrap_or(""),
                        log.status,
                        log.severity
                    ));
                }
                Ok(csv)
            }
            _ => Err(AppError::Internal(format!("Unsupported export format: {}", format))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_system() -> AuditSystem {
        AuditSystem::new(AuditConfig {
            max_log_entries: 1000,
            retention_days: 30,
            enable_compliance_reports: true,
            log_level: "info".to_string(),
        })
    }

    #[test]
    fn test_log_action() {
        let system = create_test_system();
        let result = system.log_action(
            "user1".to_string(),
            "login".to_string(),
            "auth".to_string(),
            None,
            serde_json::json!({}),
            Some("127.0.0.1".to_string()),
            Some("test-agent".to_string()),
            "success".to_string(),
            "info".to_string(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_logs() {
        let system = create_test_system();
        let _ = system.log_action(
            "user1".to_string(),
            "login".to_string(),
            "auth".to_string(),
            None,
            serde_json::json!({}),
            None,
            None,
            "success".to_string(),
            "info".to_string(),
        );
        let _ = system.log_action(
            "user2".to_string(),
            "delete".to_string(),
            "resource".to_string(),
            Some("res1".to_string()),
            serde_json::json!({}),
            None,
            None,
            "success".to_string(),
            "high".to_string(),
        );

        let logs = system.get_logs(None);
        assert_eq!(logs.len(), 2);

        let mut filters = HashMap::new();
        filters.insert("user_id".to_string(), "user1".to_string());
        let filtered = system.get_logs(Some(filters));
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].user_id, "user1");
    }

    #[test]
    fn test_delete_log() {
        let system = create_test_system();
        let log_id = system.log_action(
            "user1".to_string(),
            "test".to_string(),
            "test".to_string(),
            None,
            serde_json::json!({}),
            None,
            None,
            "success".to_string(),
            "info".to_string(),
        ).unwrap();

        let result = system.delete_log(&log_id);
        assert!(result.is_ok());

        let logs = system.get_logs(None);
        assert!(logs.is_empty());
    }

    #[test]
    fn test_purge_old_logs() {
        let system = create_test_system();
        let _ = system.log_action(
            "user1".to_string(),
            "test".to_string(),
            "test".to_string(),
            None,
            serde_json::json!({}),
            None,
            None,
            "success".to_string(),
            "info".to_string(),
        );

        let removed = system.purge_old_logs(0).unwrap();
        assert!(removed > 0);

        let logs = system.get_logs(None);
        assert!(logs.is_empty());
    }

    #[test]
    fn test_generate_compliance_report() {
        let system = create_test_system();
        let _ = system.log_action(
            "user1".to_string(),
            "login".to_string(),
            "auth".to_string(),
            None,
            serde_json::json!({}),
            None,
            None,
            "failed".to_string(),
            "high".to_string(),
        );

        let now = chrono::Utc::now().timestamp();
        let report_id = system.generate_compliance_report(
            "security_audit".to_string(),
            now - 86400,
            now,
        ).unwrap();

        let report = system.get_report(&report_id).unwrap();
        assert_eq!(report.report_type, "security_audit");
        assert!(!report.findings.is_empty());
    }

    #[test]
    fn test_get_stats() {
        let system = create_test_system();
        let _ = system.log_action(
            "user1".to_string(),
            "login".to_string(),
            "auth".to_string(),
            None,
            serde_json::json!({}),
            None,
            None,
            "success".to_string(),
            "info".to_string(),
        );
        let _ = system.log_action(
            "user2".to_string(),
            "delete".to_string(),
            "resource".to_string(),
            Some("res1".to_string()),
            serde_json::json!({}),
            None,
            None,
            "success".to_string(),
            "high".to_string(),
        );

        let stats = system.get_stats();
        assert_eq!(stats["total_logs"].as_u64().unwrap(), 2);
        assert!(stats["severity_counts"].get("info").is_some());
        assert!(stats["severity_counts"].get("high").is_some());
    }

    #[test]
    fn test_export_logs_json() {
        let system = create_test_system();
        let _ = system.log_action(
            "user1".to_string(),
            "test".to_string(),
            "test".to_string(),
            None,
            serde_json::json!({}),
            None,
            None,
            "success".to_string(),
            "info".to_string(),
        );

        let exported = system.export_logs("json").unwrap();
        assert!(exported.contains("user1"));
        assert!(exported.contains("test"));
    }

    #[test]
    fn test_export_logs_csv() {
        let system = create_test_system();
        let _ = system.log_action(
            "user1".to_string(),
            "test".to_string(),
            "test".to_string(),
            None,
            serde_json::json!({}),
            None,
            None,
            "success".to_string(),
            "info".to_string(),
        );

        let exported = system.export_logs("csv").unwrap();
        assert!(exported.contains("id,timestamp,user_id,action,resource"));
        assert!(exported.contains("user1"));
    }

    #[test]
    fn test_get_log_by_id() {
        let system = create_test_system();
        let log_id = system.log_action(
            "user1".to_string(),
            "test".to_string(),
            "test".to_string(),
            None,
            serde_json::json!({}),
            None,
            None,
            "success".to_string(),
            "info".to_string(),
        ).unwrap();

        let log = system.get_log(&log_id).unwrap();
        assert_eq!(log.id, log_id);
        assert_eq!(log.user_id, "user1");
    }

    #[test]
    fn test_max_log_entries() {
        let system = AuditSystem::new(AuditConfig {
            max_log_entries: 5,
            retention_days: 30,
            enable_compliance_reports: true,
            log_level: "info".to_string(),
        });

        for i in 0..10 {
            let _ = system.log_action(
                format!("user{}", i),
                "test".to_string(),
                "test".to_string(),
                None,
                serde_json::json!({}),
                None,
                None,
                "success".to_string(),
                "info".to_string(),
            );
        }

        let logs = system.get_logs(None);
        assert_eq!(logs.len(), 5);
        assert_eq!(logs[0].user_id, "user5");
    }
}
