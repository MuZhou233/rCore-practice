use super::File;
use lazy_static::*;
use alloc::{collections::VecDeque, sync::Arc, vec::Vec};
use spin::Mutex;
use crate::mm::{
    UserBuffer,
};

pub struct MailBox {
    role: MailerRole,
    inner: Arc<Mutex<MailBoxInner>>,
}

impl MailBox {
    pub fn new() -> (Self, Self) {
        let inner = Arc::new(Mutex::new(MailBoxInner::new()));
        (
            MailBox{role: MailerRole::Sender, inner: inner.clone()},
            MailBox{role: MailerRole::Reciver, inner}
        )
    }
    pub fn is_full(&self) -> bool {
        self.inner.lock().status == MailBoxStatus::FULL
    }
    pub fn is_empty(&self) -> bool {
        self.inner.lock().status == MailBoxStatus::EMPTY
    }
    fn clone_inner(&self) -> MailBoxInner {
        self.inner.lock().clone()
    }
    pub fn from_existed(exist: &Self) -> (Self, Self) {
        let inner = Arc::new(Mutex::new(exist.clone_inner()));
        (
            MailBox{role: MailerRole::Sender, inner: inner.clone()},
            MailBox{role: MailerRole::Reciver, inner}
        )
    }
}

lazy_static! {
    static ref MAIL_SENDER: Mutex<Vec<(usize, Arc<MailBox>)>> = Mutex::new(Vec::new());
}

pub fn update_mail_sender(new_number: usize, new_mail_box: MailBox) {
    if new_mail_box.role != MailerRole::Sender {
        panic!("set none sender to MAIL_SENDER");
    }
    for (number, mail_box) in MAIL_SENDER.lock().iter_mut() {
        if *number == new_number {
            *mail_box = Arc::new(new_mail_box);
            return;
        }
    }
    MAIL_SENDER.lock().push((new_number, Arc::new(new_mail_box)));
}

pub fn get_mail_sender(number: usize) -> Option<Arc<MailBox>> {
    for (num, mail_box) in MAIL_SENDER.lock().iter() {
        if *num == number {
            return Some(mail_box.clone());
        }
    }
    None
}

const MAIL_BOX_SIZE: usize = 16;
const MAIL_CONTENT_SIZE: usize = 256;

#[derive(Copy, Clone, PartialEq)]
enum MailBoxStatus {
    FULL,
    EMPTY,
    NORMAL,
}

#[derive(Copy, Clone, PartialEq)]
enum MailerRole {
    Sender,
    Reciver,
}

pub struct MailBoxInner {
    queue: VecDeque<MailContent>,
    status: MailBoxStatus,
}

#[derive(Clone)]
struct MailContent(Vec<u8>);

impl MailContent{
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl MailBoxInner {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            status: MailBoxStatus::EMPTY,
        }
    }
    fn send_mail(&mut self, mail: MailContent) -> bool {
        if self.status == MailBoxStatus::FULL {
            return false
        }
        self.queue.push_back(mail);
        if self.queue.len() == MAIL_BOX_SIZE {
            self.status = MailBoxStatus::FULL;
        } else {
            self.status = MailBoxStatus::NORMAL;
        }
        true
    }
    fn recive_mail(&mut self) -> Option<MailContent> {
        if self.status == MailBoxStatus::EMPTY {
            return None
        }
        let c = self.queue.pop_front();
        if self.queue.len() == 0 {
            self.status = MailBoxStatus::EMPTY;
        } else {
            self.status = MailBoxStatus::NORMAL;
        }
        c
    }
}

impl Clone for MailBoxInner {
    fn clone(&self) -> Self {
        let mut new = Self::new();
        new.status = self.status;
        for c in self.queue.iter() {
            new.queue.push_back(c.clone())
        }
        new
    }
}

impl File for MailBox {
    fn read(&self, buf: UserBuffer) -> usize {
        if self.is_empty() {
            return 0
        }
        let mut buf_iter = buf.into_iter();
        let mut read_size = 0usize;

        let mut inner = self.inner.lock();
        if let Some(mail_content) = inner.recive_mail() {
            for byte in mail_content.0.iter() {
                if let Some(byte_ref) = buf_iter.next() {
                    unsafe{ *byte_ref = *byte };
                    read_size += 1;
                } else {
                    break;
                }
            }
            read_size
        } else {
            0
        }
    }
    fn write(&self, buf: UserBuffer) -> usize {
        if self.is_full() {
            return 0
        }
        let mut buf_iter = buf.into_iter();
        let mail_content = {
            let mut mail_content = MailContent::new();
            for _ in 0.. MAIL_CONTENT_SIZE {
                if let Some(byte_ref) = buf_iter.next() {
                    mail_content.0.push(unsafe{ *byte_ref });
                } else {
                    break;
                }
            }
            mail_content
        };
        let write_size = mail_content.0.len();

        let mut inner = self.inner.lock();
        if inner.send_mail(mail_content) {
            write_size
        } else {
            0
        }
    }
}
