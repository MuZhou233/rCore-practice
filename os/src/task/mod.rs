mod context;
mod switch;
mod task;

use crate::config::{APP_DEFAULT_PRIORITY, BIG_STRIDE, MAX_APP_NUM, MAX_APP_TIME};
use crate::loader::{get_num_app, init_app_cx};
use crate::shutdown;
use crate::timer::get_time_ms;
use core::cell::RefCell;
use heapless::BinaryHeap;
use heapless::binary_heap::Min;
use lazy_static::*;
use log::{info, trace, warn};
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

pub struct TaskManager {
    inner: RefCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: [TaskControlBlock; MAX_APP_NUM],
    stride: BinaryHeap<(usize, usize), Min, MAX_APP_NUM>,
    current_task: usize,
}

unsafe impl Sync for TaskManager {}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        let num_app = get_num_app();
        let mut tasks = [
            TaskControlBlock { 
                task_cx_ptr: 0, 
                task_status: TaskStatus::UnInit, 
                task_priority: APP_DEFAULT_PRIORITY, 
                task_start_time: 0, 
                task_total_time: 0 
            };
            MAX_APP_NUM
        ];
        let mut stride = BinaryHeap::new();
        for i in 0..num_app {
            tasks[i].task_cx_ptr = init_app_cx(i) as * const _ as usize;
            tasks[i].task_status = TaskStatus::Ready;
            stride.push((0, i)).expect("Stride push failed while init");
        }

        TaskManager {
            inner: RefCell::new(TaskManagerInner {
                tasks,
                stride,
                current_task: 0,
            }),
        }
    };
}

impl TaskManager {
    fn run_first_task(&self) {
        info!("Applications start");
        self.inner.borrow_mut().tasks[0].task_status = TaskStatus::Running;
        self.inner.borrow_mut().tasks[0].task_start_time = get_time_ms();
        let next_task_cx_ptr2 = self.inner.borrow().tasks[0].get_task_cx_ptr2();
        let _unused: usize = 0;
        unsafe {
            __switch(
                &_unused as *const _,
                next_task_cx_ptr2,
            );
        }
    }

    fn set_current_priority(&self, prio: usize) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_priority = prio;
    }

    fn mark_current_suspended(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Ready;

        inner.tasks[current].task_total_time += get_time_ms() - inner.tasks[current].task_start_time;
        if inner.tasks[current].task_total_time > MAX_APP_TIME {
            warn!("Application {} be killed because runs too long", current);
            inner.tasks[current].task_status = TaskStatus::Exited;
            core::mem::drop(inner);
            if self.find_next_task().is_none() {
                shutdown()
            }
        }
    }

    fn mark_current_exited(&self) {
        let mut inner = self.inner.borrow_mut();
        let current = inner.current_task;
        inner.tasks[current].task_status = TaskStatus::Exited;
    }

    fn get_current_id(&self) -> usize {
        let inner = self.inner.borrow();
        inner.current_task.clone()
    }

    fn find_next_task(&self) -> Option<usize> {
        let mut inner = self.inner.borrow_mut();
        while let Some((stride, next)) = inner.stride.pop() {
            if inner.tasks[next].task_status != TaskStatus::Ready {
                continue;
            }
            let priority = inner.tasks[next].task_priority;
            inner.stride.push((stride + BIG_STRIDE / priority, next)).expect(
                "Stride push failed"
            );
            return Some(next);
        }
        None
    }

    fn run_next_task(&self) {
        if let Some(next) = self.find_next_task() {
            let mut inner = self.inner.borrow_mut();
            let current = inner.current_task;
            trace!("switch to app {}", next);
            inner.tasks[next].task_status = TaskStatus::Running;
            inner.tasks[next].task_start_time = get_time_ms();
            inner.current_task = next;
            let current_task_cx_ptr2 = inner.tasks[current].get_task_cx_ptr2();
            let next_task_cx_ptr2 = inner.tasks[next].get_task_cx_ptr2();
            core::mem::drop(inner);
            unsafe {
                __switch(
                    current_task_cx_ptr2,
                    next_task_cx_ptr2,
                );
            }
        }
    }
}

pub fn run_first_task() {
    TASK_MANAGER.run_first_task();
}

fn run_next_task() {
    TASK_MANAGER.run_next_task();
}

fn mark_current_suspended() {
    TASK_MANAGER.mark_current_suspended();
}

fn mark_current_exited() {
    TASK_MANAGER.mark_current_exited();
}

pub fn suspend_current_and_run_next() {
    mark_current_suspended();
    run_next_task();
}

pub fn exit_current_and_run_next() -> ! {
    mark_current_exited();
    run_next_task();
    shutdown()
}

pub fn set_current_priority(prio: usize) {
    TASK_MANAGER.set_current_priority(prio);
}

pub fn get_current_app_id() -> usize {
    TASK_MANAGER.get_current_id()
}