use core::ops::Range;

use log::trace;

use crate::mm::{MemorySet, MapPermission, PhysPageNum, KERNEL_SPACE, VirtAddr};
use crate::trap::{TrapContext, trap_handler};
use crate::config::{APP_DEFAULT_PRIORITY, TRAP_CONTEXT, kernel_stack_position};
use super::TaskContext;

pub struct TaskControlBlock {
    pub task_cx_ptr: usize,
    pub task_status: TaskStatus,
    pub task_priority: usize,
    pub task_start_time: usize,
    pub task_total_time: usize,
    pub memory_set: MemorySet,
    pub trap_cx_ppn: PhysPageNum,
    pub base_size: usize,
}

impl TaskControlBlock {
    pub fn get_task_cx_ptr2(&self) -> *const usize {
        &self.task_cx_ptr as *const usize
    }
    pub fn get_trap_cx(&self) -> &'static mut TrapContext {
        self.trap_cx_ppn.get_mut()
    }
    pub fn get_user_token(&self) -> usize {
        self.memory_set.token()
    }
    pub fn add_map_area(&mut self, vddr: Range<usize>, port: usize, exact: bool) -> Option<usize> {
        if (port & !0x7 != 0) || (port & 0x7 == 0) {
            return None;
        }
        let mut permission = MapPermission::U;
        if port & 0x1 == 0x1 {permission |= MapPermission::R}
        if port & 0x2 == 0x2 {permission |= MapPermission::W}
        if port & 0x4 == 0x4 {permission |= MapPermission::X}
        trace!("add_map_area: permission {:?}", permission);
        self.memory_set.insert_framed_area(
            vddr.start.into(), 
            vddr.end.into(), 
            permission, 
            exact
        )
    }
    pub fn remove_map_area(&mut self, vddr: Range<usize>) -> bool {
        self.memory_set.remove_framed_area(
            vddr.start.into(), 
            vddr.end.into()
        )
    }
    pub fn new(elf_data: &[u8], app_id: usize) -> Self {
        // memory_set with elf program headers/trampoline/trap context/user stack
        let (memory_set, user_sp, entry_point) = MemorySet::from_elf(elf_data);
        let trap_cx_ppn = memory_set
            .translate(VirtAddr::from(TRAP_CONTEXT).into())
            .unwrap()
            .ppn();
        let task_status = TaskStatus::Ready;
        // map a kernel-stack in kernel space
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(app_id);
        KERNEL_SPACE
            .lock()
            .insert_framed_area(
                kernel_stack_bottom.into(),
                kernel_stack_top.into(),
                MapPermission::R | MapPermission::W,
                false
            );
        let task_cx_ptr = (kernel_stack_top - core::mem::size_of::<TaskContext>()) as *mut TaskContext;
        unsafe { *task_cx_ptr = TaskContext::goto_trap_return(); }
        let task_control_block = Self {
            task_cx_ptr: task_cx_ptr as usize,
            task_status,
            task_priority: APP_DEFAULT_PRIORITY,
            task_start_time: 0,
            task_total_time: 0,
            memory_set,
            trap_cx_ppn,
            base_size: user_sp,
        };
        // prepare TrapContext in user space
        let trap_cx = task_control_block.get_trap_cx();
        *trap_cx = TrapContext::app_init_context(
            entry_point,
            user_sp,
            KERNEL_SPACE.lock().token(),
            kernel_stack_top,
            trap_handler as usize,
        );
        task_control_block
    }
}

#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    Ready,
    Running,
    Exited,
}