use core::borrow::BorrowMut;

use alloc::sync::Arc;

use super::process::Tid;
use super::thread_control_block::ThreadControlBlock;
use super::{scheduling::scheduler_yield_and_block, thread_control_block::ThreadStatus};
use crate::sync::rwlock::sleep::RwLock;
use crate::system::unwrap_system_mut;

pub fn thread_sleep() {
    scheduler_yield_and_block();
}

pub fn thread_wakeup(tid: Tid) {
    let threads = unsafe { &mut unwrap_system_mut().threads };
    if let Some(mut tcb) = threads.scheduler.get_mut(tid) {
        tcb.status = ThreadStatus::Ready;
    }
}
