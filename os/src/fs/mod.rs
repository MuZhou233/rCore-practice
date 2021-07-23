mod mail;
mod pipe;
mod stdio;

use crate::mm::UserBuffer;
pub trait File : Send + Sync {
    fn read(&self, buf: UserBuffer) -> usize;
    fn write(&self, buf: UserBuffer) -> usize;
}

pub use mail::{MailBox, update_mail_sender, get_mail_sender};
pub use pipe::{Pipe, make_pipe};
pub use stdio::{Stdin, Stdout};