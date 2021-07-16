//! 中断模块

mod context;
mod handler;
mod timer;

pub use context::Context;
use log::info;

/// 初始化中断相关的子模块
///
/// - [`handler::init`]
/// - [`timer::init`]
pub fn init() {
    handler::init();
    timer::init();
    info!("mod interrupt initialized");
}
