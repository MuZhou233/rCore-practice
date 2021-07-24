use crate::mm::{UserBuffer, translated_byte_buffer, translated_refmut, translated_str, write_translated_byte_buffer};
use crate::task::{current_user_token, current_task};
use crate::fs::{File, OSInode, OpenFlags, get_mail_sender, linkat, linknum, list_apps, make_pipe, open_file, unlinkat};
use alloc::sync::Arc;
use log::trace;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release Task lock manually to avoid deadlock
        drop(inner);
        if let Some(buffer) = translated_byte_buffer(token, buf, len) {
            file.write(
                UserBuffer::new(buffer)
            ) as isize
        } else {
            -1
        }
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release Task lock manually to avoid deadlock
        drop(inner);
        if let Some(buffer) = translated_byte_buffer(token, buf, len) {
            file.read(
                UserBuffer::new(buffer)
            ) as isize
        } else {
            -1
        }
    } else {
        -1
    }
}

pub fn sys_open(_dirfd: usize, path: *const u8, flags: u32, _mode: u32) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(
        path.as_str(),
        OpenFlags::from_bits(flags).unwrap()
    ) {
        let mut inner = task.acquire_inner_lock();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

pub fn sys_linkat(_olddirfd: i32, oldpath: *const u8, _newdirfd: i32, newpath: *const u8, _flags: u32) -> isize {
    let token = current_user_token();
    let oldpath = translated_str(token, oldpath);
    let newpath = translated_str(token, newpath);
    if let Some(_) = linkat(&oldpath, &newpath) {
        0
    } else {
        -1
    }
}

pub fn sys_unlinkat(_dirfd: i32, path: *const u8, _flags: u32) -> isize {
    let token = current_user_token();
    let path = translated_str(token, path);
    unlinkat(&path);
    0
}

#[repr(C)]
#[derive(Debug)]
pub struct Stat {
    /// 文件所在磁盘驱动器号
    pub dev: u64,
    /// inode 文件所在 inode 编号
    pub ino: u64,
    /// 文件类型
    pub mode: StatMode,
    /// 硬链接数量，初始为1
    pub nlink: u32,
    /// 无需考虑，为了兼容性设计
    pad: [u64; 7],
}
bitflags! {
    pub struct StatMode: u32 {
        const NULL  = 0;
        /// directory
        const DIR   = 0o040000;
        /// ordinary regular file
        const FILE  = 0o100000;
    }
}
pub fn sys_fstat(fd: i32, st: *const u8) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let inner = task.acquire_inner_lock();
    if let Some(file) = &inner.fd_table[fd as usize] {
        if let Some(file) = file.as_any().downcast_ref::<OSInode>() {
            let stat = Stat{
                dev: 0,
                ino: 0,
                mode: StatMode::FILE,
                nlink: linknum(&file) as u32,
                pad: [0u64; 7]
            };
            write_translated_byte_buffer(token, st, core::mem::size_of::<Stat>(), 
            unsafe{ core::slice::from_raw_parts(&stat as *const _ as *const u8, core::mem::size_of::<Stat>()) });
            return 0;
        }
    }

    -1
}

pub fn sys_pipe(pipe: *mut usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let mut inner = task.acquire_inner_lock();
    let (pipe_read, pipe_write) = make_pipe();
    let read_fd = inner.alloc_fd();
    inner.fd_table[read_fd] = Some(pipe_read);
    let write_fd = inner.alloc_fd();
    inner.fd_table[write_fd] = Some(pipe_write);
    *translated_refmut(token, pipe) = read_fd;
    *translated_refmut(token, unsafe { pipe.add(1) }) = write_fd;
    0
}

pub fn sys_mailread(buf: *mut u8, len: usize) -> isize {
    let task = current_task().unwrap();
    let token = current_user_token();
    let inner = task.acquire_inner_lock();
    if !inner.mail_box.readable() {
        -1
    } else if len == 0 {
        0
    } else if let Some(buffer) = translated_byte_buffer(token, buf, len) {
        inner.mail_box.read(
            UserBuffer::new(buffer)
        ) as isize
    } else {
        -1
    }
}

pub fn sys_mailwrite(pid: usize, buf: *mut u8, len: usize) -> isize {
    let token = current_user_token();
    if let Some(sender) = get_mail_sender(pid) {
        if !sender.writable() {
            -1
        } else if len == 0 {
            0
        } else if let Some(buffer) = translated_byte_buffer(token, buf, len) {
            sender.write(
                UserBuffer::new(buffer)
            ) as isize
        } else {
            -1
        }
    } else {
        -1
    }
}

pub fn sys_dup(fd: usize) -> isize {
    let task = current_task().unwrap();
    let mut inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    let new_fd = inner.alloc_fd();
    inner.fd_table[new_fd] = Some(Arc::clone(inner.fd_table[fd].as_ref().unwrap()));
    new_fd as isize
}