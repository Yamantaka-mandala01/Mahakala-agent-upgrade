use crate::error::AppError;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub name: String,
    pub expression: String,
    pub command: String,
    pub running: bool,
    pub last_run: Option<i64>,
    pub next_run: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJobCreate {
    pub name: String,
    pub expression: String,
    pub command: String,
}

pub struct CronManager {
    jobs: Arc<Mutex<HashMap<String, CronJob>>>,
}

impl Default for CronManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CronManager {
    pub fn new() -> Self {
        Self {
            jobs: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn add_job(&self, job: CronJobCreate) -> Result<CronJob, AppError> {
        let now = chrono::Utc::now().timestamp();
        let cron_job = CronJob {
            id: Uuid::new_v4().to_string(),
            name: job.name,
            expression: job.expression,
            command: job.command,
            running: false,
            last_run: None,
            next_run: None,
            created_at: now,
        };

        let mut jobs = self.jobs.lock();
        jobs.insert(cron_job.id.clone(), cron_job.clone());

        Ok(cron_job)
    }

    pub fn start_job(&self, job_id: &str) -> Result<bool, AppError> {
        let mut jobs = self.jobs.lock();
        if let Some(job) = jobs.get_mut(job_id) {
            job.running = true;
            job.next_run = Some(chrono::Utc::now().timestamp() + 60);
            Ok(true)
        } else {
            Err(AppError::NotFound(format!("Job {} not found", job_id)))
        }
    }

    pub fn stop_job(&self, job_id: &str) -> Result<bool, AppError> {
        let mut jobs = self.jobs.lock();
        if let Some(job) = jobs.get_mut(job_id) {
            job.running = false;
            Ok(true)
        } else {
            Err(AppError::NotFound(format!("Job {} not found", job_id)))
        }
    }

    pub fn remove_job(&self, job_id: &str) -> Result<bool, AppError> {
        let mut jobs = self.jobs.lock();
        Ok(jobs.remove(job_id).is_some())
    }

    pub fn get_job(&self, job_id: &str) -> Option<CronJob> {
        let jobs = self.jobs.lock();
        jobs.get(job_id).cloned()
    }

    pub fn list_jobs(&self) -> Vec<CronJob> {
        let jobs = self.jobs.lock();
        jobs.values().cloned().collect()
    }

    pub fn run_now(&self, job_id: &str) -> Result<(), AppError> {
        let mut jobs = self.jobs.lock();
        if let Some(job) = jobs.get_mut(job_id) {
            job.last_run = Some(chrono::Utc::now().timestamp());
            Ok(())
        } else {
            Err(AppError::NotFound(format!("Job {} not found", job_id)))
        }
    }

    pub fn update_last_run(&self, job_id: &str) {
        let mut jobs = self.jobs.lock();
        if let Some(job) = jobs.get_mut(job_id) {
            let now = chrono::Utc::now().timestamp();
            job.last_run = Some(now);
        }
    }
}

#[derive(Clone)]
pub struct CronManagerHandle {
    inner: Arc<CronManager>,
}

impl CronManagerHandle {
    pub async fn new() -> Result<Self, AppError> {
        Ok(Self {
            inner: Arc::new(CronManager::new()),
        })
    }

    pub fn add_job(&self, job: CronJobCreate) -> Result<CronJob, AppError> {
        self.inner.add_job(job)
    }

    pub fn start_job(&self, job_id: &str) -> Result<bool, AppError> {
        self.inner.start_job(job_id)
    }

    pub fn stop_job(&self, job_id: &str) -> Result<bool, AppError> {
        self.inner.stop_job(job_id)
    }

    pub fn remove_job(&self, job_id: &str) -> Result<bool, AppError> {
        self.inner.remove_job(job_id)
    }

    pub fn get_job(&self, job_id: &str) -> Option<CronJob> {
        self.inner.get_job(job_id)
    }

    pub fn list_jobs(&self) -> Vec<CronJob> {
        self.inner.list_jobs()
    }

    pub fn run_now(&self, job_id: &str) -> Result<(), AppError> {
        self.inner.run_now(job_id)
    }

    pub fn update_last_run(&self, job_id: &str) {
        self.inner.update_last_run(job_id);
    }
}
