use log::info;

use crate::task::{exit_current_and_run_next, set_current_priority, suspend_current_and_run_next};
use crate::timer::get_time_ms;

pub fn sys_exit(exit_code: i32) -> ! {
    info!("Application exited with code {}", exit_code);
    exit_current_and_run_next()
}

pub fn sys_yield() -> isize {
    suspend_current_and_run_next();
    0
}

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

pub fn sys_get_time(time: &mut TimeVal, _tz: usize) -> isize {
    let time_ms = get_time_ms();
    time.sec = time_ms / 1000;
    time.usec = (time_ms % 1000) * 1000;
    0
}

pub fn sys_set_priority(prio: isize) -> isize {
    if prio > 1 {
        set_current_priority(prio as usize);
        prio
    } else {
        -1
    }
}