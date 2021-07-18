mod context;
mod switch;
mod task;

use crate::config::{BIG_STRIDE, MAX_APP_TIME};
use crate::loader::{get_num_app, get_app_data};
use crate::sbi::shutdown;
use crate::timer::get_time_ms;
use crate::trap::TrapContext;
use core::cell::RefCell;
use core::cmp::Reverse;
use alloc::collections::BinaryHeap;
use alloc::vec::Vec;
use lazy_static::*;
use log::{info, trace, warn};
use switch::__switch;
use task::{TaskControlBlock, TaskStatus};

pub use context::TaskContext;

pub struct TaskManager {
    inner: RefCell<TaskManagerInner>,
}

struct TaskManagerInner {
    tasks: Vec<TaskControlBlock>,
    stride: BinaryHeap<Reverse<(usize, usize)>>,
    current_task: usize,
}

unsafe impl Sync for TaskManager {}

lazy_static! {
    pub static ref TASK_MANAGER: TaskManager = {
        info!("init TASK_MANAGER");
        let num_app = get_num_app();
        info!("num_app = {}", num_app);
        let mut tasks: Vec<TaskControlBlock> = Vec::new();
        let mut stride = BinaryHeap::new();
        for i in 0..num_app {
            tasks.push(TaskControlBlock::new(
                get_app_data(i),
                i,
            ));
            stride.push(Reverse((0, i)));
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

    fn find_next_task(&self) -> Option<usize> {
        let mut inner = self.inner.borrow_mut();
        while let Some(Reverse((stride, next))) = inner.stride.pop() {
            if inner.tasks[next].task_status != TaskStatus::Ready {
                continue;
            }
            let priority = inner.tasks[next].task_priority;
            inner.stride.push(Reverse((stride + BIG_STRIDE / priority, next)));
            return Some(next);
        }
        None
    }

    fn get_current_token(&self) -> usize {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        inner.tasks[current].get_user_token()
    }

    fn get_current_trap_cx(&self) -> &mut TrapContext {
        let inner = self.inner.borrow();
        let current = inner.current_task;
        inner.tasks[current].get_trap_cx()
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

pub fn current_user_token() -> usize {
    TASK_MANAGER.get_current_token()
}

pub fn current_trap_cx() -> &'static mut TrapContext {
    TASK_MANAGER.get_current_trap_cx()
}