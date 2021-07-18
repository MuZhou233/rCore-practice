const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
const SYSCALL_YIELD: usize = 124;
const SYSCALL_GET_TIME: usize = 169;
const SYSCALL_SET_PRIORITY: usize = 140;
const SYSCALL_MUNMAP: usize = 215;
const SYSCALL_MMAP: usize = 222;

mod fs;
mod process;

use fs::*;
use log::trace;
use process::*;

use crate::mm::write_translated_byte_buffer;
use crate::task::current_user_token;

pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    trace!("process syscall {}", syscall_id);
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        SYSCALL_YIELD => sys_yield(),
        SYSCALL_GET_TIME => {
            let mut time = TimeVal{ sec: 0, usec: 0 };
            let ret = sys_get_time(&mut time, args[1]);
            write_translated_byte_buffer(current_user_token(), args[0] as *const u8, core::mem::size_of::<TimeVal>(), 
            unsafe{ core::slice::from_raw_parts(&time as *const _ as *const u8, core::mem::size_of::<TimeVal>()) });
            ret
        },
        SYSCALL_SET_PRIORITY => sys_set_priority(args[0] as isize),
        SYSCALL_MUNMAP => sys_munmap(args[0], args[1]) as isize,
        SYSCALL_MMAP => sys_mmap(args[0], args[1], args[2]) as isize,
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}

