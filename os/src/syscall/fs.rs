use crate::mm::{UserBuffer, translated_byte_buffer, translated_refmut};
use crate::task::{current_user_token, current_task};
use crate::fs::{File, get_mail_sender, make_pipe};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.acquire_inner_lock();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
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
    if inner.mail_box.is_empty() {
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
        if sender.is_full() {
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