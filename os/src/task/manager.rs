use crate::config::BIG_STRIDE;

use super::TaskControlBlock;
use alloc::collections::BinaryHeap;
use alloc::sync::Arc;
use core::cmp::Reverse;
use spin::Mutex;
use lazy_static::*;

pub struct TaskManager {
    ready_queue: BinaryHeap<Reverse<Arc<TaskControlBlock>>>,
}

/// A simple FIFO scheduler.
impl TaskManager {
    pub fn new() -> Self {
        Self { ready_queue: BinaryHeap::new(), }
    }
    pub fn add(&mut self, task: Arc<TaskControlBlock>) {
        self.ready_queue.push(Reverse(task));
    }
    pub fn fetch(&mut self) -> Option<Arc<TaskControlBlock>> {
        self.ready_queue.pop().map(|task| {
            let task = task.0;
            task.set_task_stride(task.get_task_stride() + BIG_STRIDE / task.get_task_priority());
            task
        })
    }
}

lazy_static! {
    pub static ref TASK_MANAGER: Mutex<TaskManager> = Mutex::new(TaskManager::new());
}

pub fn add_task(task: Arc<TaskControlBlock>) {
    TASK_MANAGER.lock().add(task);
}

pub fn fetch_task() -> Option<Arc<TaskControlBlock>> {
    TASK_MANAGER.lock().fetch()
}