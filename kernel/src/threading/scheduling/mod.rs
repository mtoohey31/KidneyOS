mod fifo_scheduler;
mod scheduler;

pub use fifo_scheduler::FIFOScheduler;
pub use scheduler::Scheduler;

use alloc::boxed::Box;

use super::{context_switch::switch_threads, thread_control_block::ThreadStatus};
use crate::interrupts::{intr_get_level, mutex_irq::hold_interrupts, IntrLevel};
use crate::system::unwrap_system;

pub fn create_scheduler() -> Box<dyn Scheduler + Send> {
    assert_eq!(intr_get_level(), IntrLevel::IntrOff);

    // SAFETY: Interrupts should be off.
    Box::new(FIFOScheduler::new())
}

/// Voluntarily relinquishes control of the CPU to another processor in the scheduler.
fn scheduler_yield(status_for_current_thread: ThreadStatus) {
    let _guard = hold_interrupts(IntrLevel::IntrOff);

    let mut scheduler = unwrap_system().threads.scheduler.lock();

    while let Some(switch_to) = scheduler.pop() {
        // Check if the thread is not blocked.
        match switch_to.as_ref().status {
            ThreadStatus::Blocked => {
                // If the thread is blocked, push it back onto the scheduler.
                scheduler.push(switch_to);
            }
            _ => {
                drop(scheduler);
                // SAFETY: Threads and Scheduler must be initialized and active.
                // Interrupts must be disabled.
                unsafe {
                    // Do not switch to ourselves.
                    switch_threads(status_for_current_thread, switch_to);
                }
                break;
            }
        }
    }

    // Note: _guard falls out of scope and re-enables interrupts if previously enabled
}

// Voluntarily relinquishes control of the CPU and marks current thread as ready.
pub fn scheduler_yield_and_continue() {
    scheduler_yield(ThreadStatus::Ready);
}

/// Voluntarily relinquishes control of the CPU and marks the current thread to die.
pub fn scheduler_yield_and_die() -> ! {
    scheduler_yield(ThreadStatus::Dying);

    panic!("A thread was rescheduled after dying.");
}

/// Voluntarily relinquishes control of the CPU and marks the current thread as blocked.
#[allow(unused)]
pub fn scheduler_yield_and_block() {
    scheduler_yield(ThreadStatus::Blocked);
}
