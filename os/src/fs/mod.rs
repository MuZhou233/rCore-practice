mod mail;
mod pipe;
mod stdio;
mod inode;

use core::any::Any;

use crate::mm::UserBuffer;
pub trait File : Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn readable(&self) -> bool;
    fn writable(&self) -> bool;
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

pub use mail::{MailBox, update_mail_sender, get_mail_sender};
pub use pipe::{Pipe, make_pipe};
pub use stdio::{Stdin, Stdout};
pub use inode::{OSInode, open_file, OpenFlags, list_apps, linkat, unlinkat, linknum};