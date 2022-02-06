use chrono::prelude::*;
use petgraph::graph::NodeIndex;

pub enum TaskResultStatus {
    Cancelled,
    Failed,
    Invalid,
    Passed,
    Running,
}

pub struct TaskResult {
    pub start_time: DateTime<Local>,

    pub status: TaskResultStatus,

    pub end_time: Option<DateTime<Local>>,

    pub exit_code: i8,

    pub node_index: NodeIndex,

    pub stderr: String,

    pub stdout: String,
}

impl TaskResult {
    pub fn new(node_index: NodeIndex) -> Self {
        TaskResult {
            start_time: Local::now(),
            status: TaskResultStatus::Running,
            end_time: None,
            exit_code: -1,
            node_index,
            stderr: String::new(),
            stdout: String::new(),
        }
    }

    pub fn pass(&mut self) {
        self.status = TaskResultStatus::Passed;
        self.end_time = Some(Local::now());
    }

    pub fn fail(&mut self) {
        self.status = TaskResultStatus::Failed;
        self.end_time = Some(Local::now());
    }
}
