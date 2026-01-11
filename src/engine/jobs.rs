use nix::unistd::Pid;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Running,
    Suspended,
}

impl fmt::Display for JobStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JobStatus::Running => write!(f, "Running"),
            JobStatus::Suspended => write!(f, "Stopped"),
        }
    }
}

use std::time::Instant;

#[derive(Debug, Clone)]
pub struct Job {
    pub id: usize,
    pub pgid: Pid,
    pub cmd: String,
    pub status: JobStatus,
    pub start_time: Instant,
}

pub struct JobManager {
    jobs: Vec<Job>,
    next_id: usize,
}

impl JobManager {
    pub fn new() -> Self {
        Self {
            jobs: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_job(&mut self, pgid: Pid, cmd: String, status: JobStatus) -> usize {
        let id = self.next_id;
        self.jobs.push(Job { id, pgid, cmd, status, start_time: Instant::now() });
        self.next_id += 1;
        id
    }

    pub fn remove_job(&mut self, pgid: Pid) {
        self.jobs.retain(|j| j.pgid != pgid);
        if self.jobs.is_empty() {
            self.next_id = 1;
        }
    }

    pub fn get_jobs(&self) -> &[Job] {
        &self.jobs
    }

    pub fn find_job_by_id(&self, id: usize) -> Option<&Job> {
        self.jobs.iter().find(|j| j.id == id)
    }
}
