use log::info;

use crate::config::PAGE_SIZE;
use crate::task::{add_current_map_area, exit_current_and_run_next, remove_current_map_area, set_current_priority, suspend_current_and_run_next};
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
pub fn sys_mmap(start: usize, len: usize, port: usize) -> i32 {
    if start % PAGE_SIZE > 0 {
        -1
    } else {
        add_current_map_area(start..start+len, port, true)
            .map(|l| l as i32).unwrap_or(-1)
    }
}

pub fn sys_munmap(start: usize, len: usize) -> i32 {
    if start % PAGE_SIZE > 0 {
        -1
    } else if remove_current_map_area(start..start+len) {
        ((len - 1 + PAGE_SIZE) / PAGE_SIZE * PAGE_SIZE) as i32
    } else {
        -1
    }
}