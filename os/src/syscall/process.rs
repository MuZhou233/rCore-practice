use crate::config::PAGE_SIZE;
use crate::task::{
    suspend_current_and_run_next,
    exit_current_and_run_next,
    current_task,
    current_user_token,
    add_task,
    current_add_map_area,
    current_remove_map_area
};
use crate::timer::get_time_ms;
use crate::mm::{translated_refmut, translated_str, write_translated_byte_buffer};
use crate::loader::get_app_data_by_name;
use alloc::sync::Arc;

pub fn sys_exit(exit_code: i32) -> ! {
    exit_current_and_run_next(exit_code);
    panic!("Unreachable in sys_exit!");
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

pub fn sys_get_time(time_addr: usize, _tz: usize) -> isize {
    let mut time = TimeVal{ sec: 0, usec: 0 };
    let time_ms = get_time_ms();
    time.sec = time_ms / 1000;
    time.usec = (time_ms % 1000) * 1000;
    write_translated_byte_buffer(current_user_token(), time_addr as *const u8, core::mem::size_of::<TimeVal>(), 
    unsafe{ core::slice::from_raw_parts(&time as *const _ as *const u8, core::mem::size_of::<TimeVal>()) });
    0
}

pub fn sys_set_priority(prio: isize) -> isize {
    if prio > 1 {
        current_task().unwrap()
            .set_task_priority(prio as usize);
        prio
    } else {
        -1
    }
}

pub fn sys_mmap(start: usize, len: usize, port: usize) -> i32 {
    if start % PAGE_SIZE > 0 {
        -1
    } else {
        current_add_map_area(start..start+len, port, true)
            .map(|l| l as i32).unwrap_or(-1)
    }
}

pub fn sys_munmap(start: usize, len: usize) -> i32 {
    if start % PAGE_SIZE > 0 {
        -1
    } else if current_remove_map_area(start..start+len) {
        ((len - 1 + PAGE_SIZE) / PAGE_SIZE * PAGE_SIZE) as i32
    } else {
        -1
    }
}

pub fn sys_getpid() -> isize {
    current_task().unwrap().pid.0 as isize
}

pub fn sys_fork() -> isize {
    let current_task = current_task().unwrap();
    let new_task = current_task.fork();
    let new_pid = new_task.pid.0;
    // modify trap context of new_task, because it returns immediately after switching
    let trap_cx = new_task.acquire_inner_lock().get_trap_cx();
    // we do not have to move to next instruction since we have done it before
    // for child process, fork returns 0
    trap_cx.x[10] = 0;
    // add new task to scheduler
    add_task(new_task);
    new_pid as isize
}

pub fn sys_exec(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        task.exec(data);
        0
    } else {
        -1
    }
}

pub fn sys_spawn(path: *const u8) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(data) = get_app_data_by_name(path.as_str()) {
        let task = current_task().unwrap();
        let new_task = task.spawn(data);
        
        let new_pid = new_task.pid.0;
        // modify trap context of new_task, because it returns immediately after switching
        let trap_cx = new_task.acquire_inner_lock().get_trap_cx();
        // we do not have to move to next instruction since we have done it before
        // for child process, fork returns 0
        trap_cx.x[10] = 0;
        // add new task to scheduler
        add_task(new_task);
        new_pid as isize
    } else {
        -1
    }
}

/// If there is not a child process whose pid is same as given, return -1.
/// Else if there is a child process but it is still running, return -2.
pub fn sys_waitpid(pid: isize, exit_code_ptr: *mut i32) -> isize {
    let task = current_task().unwrap();
    // find a child process

    // ---- hold current PCB lock
    let mut inner = task.acquire_inner_lock();
    if inner.children
        .iter()
        .find(|p| {pid == -1 || pid as usize == p.getpid()})
        .is_none() {
        return -1;
        // ---- release current PCB lock
    }
    let pair = inner.children
        .iter()
        .enumerate()
        .find(|(_, p)| {
            // ++++ temporarily hold child PCB lock
            p.acquire_inner_lock().is_zombie() && (pid == -1 || pid as usize == p.getpid())
            // ++++ release child PCB lock
        });
    if let Some((idx, _)) = pair {
        let child = inner.children.remove(idx);
        // confirm that child will be deallocated after removing from children list
        assert_eq!(Arc::strong_count(&child), 1);
        let found_pid = child.getpid();
        // ++++ temporarily hold child lock
        let exit_code = child.acquire_inner_lock().exit_code;
        // ++++ release child PCB lock
        *translated_refmut(inner.memory_set.token(), exit_code_ptr) = exit_code;
        found_pid as isize
    } else {
        -2
    }
    // ---- release current PCB lock automatically
}