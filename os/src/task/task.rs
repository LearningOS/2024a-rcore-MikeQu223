//! Types related to task management

use super::TaskContext;
use crate::{config::MAX_SYSCALL_NUM, timer::get_time_ms};

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    /// task_info
    pub task_info: TaskInfo,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}

/// Task information
#[allow(dead_code)]
#[derive(Clone,Copy,Debug)]
pub struct TaskInfo {
    /// Task status in it's life cycle
    pub status: TaskStatus,
    /// The numbers of syscall called by task
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    pub start_time: usize,
}

impl TaskInfo {
    /// 创建一个新的 TaskInfo 实例
    pub fn new() -> Self {
        TaskInfo {
            status: TaskStatus::UnInit,
            syscall_times: [0; MAX_SYSCALL_NUM],
            start_time: 0,
        }
    }

    /// 获取任务的运行时间
    pub fn get_time(&mut self) -> usize {
        if self.status == TaskStatus::Running {
            get_time_ms() - self.start_time
        } else {
            0
        }
    }

    /// 设置任务开始
    pub fn start(&mut self) {
        self.status = TaskStatus::Running;
        self.start_time = get_time_ms();
    }
}
