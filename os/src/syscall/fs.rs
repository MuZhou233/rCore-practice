use core::ops::Range;

use log::{trace, warn};

use crate::batch::{get_app_address, get_user_stack_address};

const FD_STDOUT: usize = 1;

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    let app_addr = get_app_address();
    let stack_addr = get_user_stack_address();
    let write_addr = buf as usize..(buf as usize + len);

    if !range_in_range(&app_addr, &write_addr) &&
       !range_in_range(&stack_addr, &write_addr) {
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

fn range_in_range(outer: &Range<usize>, inner: &Range<usize>) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}