use log::{trace, warn};

use crate::batch::get_app_address;

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let app_addr = get_app_address();
    if !app_addr.contains(&(buf as usize)) || !app_addr.contains(&(buf as usize+len)) {
        warn!("Try to print illegal address {:?}", buf);
        return -1;
    }

    match fd {
        FD_STDOUT => {
            trace!("sys_write: required buf from {:?} len {}", buf, len);
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        },
        _ => {
            warn!("Unsupported fd in sys_write!");
            -1
        }
    }
}