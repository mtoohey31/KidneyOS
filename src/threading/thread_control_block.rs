
use core::alloc::{Layout, Allocator};
use core::mem::size_of;
use core::ptr::NonNull;

use alloc::alloc::Global;

use crate::constants::KB;
use crate::threading::context_switch::Context;
use crate::threading::thread_functions::{ThreadFunction, PrepareThreadContext, RunThreadContext, run_thread, prepare_thread};

pub type TID = u16;

// Current value marks the next avaliable TID value to use.
static mut NEXT_UNRESERVED_TID: TID = 0;

pub const THREAD_STACK_SIZE: usize = KB * 4;

pub enum ThreadStatus {
    Invalid,
    Running,
    Ready,
    Blocked,
    Dying
}

pub struct ThreadControlBlock {

    pub tid: TID,
    pub status: ThreadStatus,
    pub stack_pointer: NonNull<u8>,
    _stack_pointer_bottom: NonNull<[u8]>, // Kept to avoid dropping the stack and to detect overflows.
    pub context: Context, // Not always valid. TODO: Use type system here, worried about use in inline assembly and ownership.

}

pub fn allocate_tid() -> TID {

    unsafe {
        let new_tid = NEXT_UNRESERVED_TID;

        // TODO: Lock.
        NEXT_UNRESERVED_TID += 1;

        return new_tid;
    }

}

impl ThreadControlBlock {

    pub fn create(entry_function: ThreadFunction) -> Self {

        let tid: TID = allocate_tid();

        // Allocate a stack for this thread.
        // In x86 stacks from downward, so we must pass in the top of this memory to the thread.
        let stack_pointer_bottom;
        let stack_pointer_top;
        let layout = Layout::from_size_align(THREAD_STACK_SIZE, 8).unwrap();
        unsafe {
            stack_pointer_bottom = Global.allocate_zeroed(layout).expect("Could not allocate stack.");
            stack_pointer_top = NonNull::new(stack_pointer_bottom.as_ptr().cast::<u8>().add(THREAD_STACK_SIZE)).expect("Could not determine end of stack.");
        }

        // Create our new TCB.
        let mut new_thread = Self {
            tid,
            status: ThreadStatus::Invalid,
            stack_pointer: stack_pointer_top,
            _stack_pointer_bottom: stack_pointer_bottom,
            context: Context::empty_context()
        };

        // Now, we must build the stack frames for our new thread.
        // In order (of creation), we have:
        //  * run_thread frame
        //  * prepare_thread frame
        //  * switch_threads frame

        // TODO: Farm this out to a few functions.
        // TODO: Generalize the allocation function.

        let run_thread_stack_frame = new_thread.allocate_stack_space(core::mem::size_of::<RunThreadContext>());
        unsafe {
            *run_thread_stack_frame.as_ptr().cast::<RunThreadContext>() = RunThreadContext {
                eip: 0,
                entry_function_pointer: entry_function as usize
            };
        }

        let prepare_thread_stack_frame = new_thread.allocate_stack_space(core::mem::size_of::<PrepareThreadContext>());
        unsafe {
            *prepare_thread_stack_frame.as_ptr().cast::<PrepareThreadContext>() = PrepareThreadContext {
                eip: run_thread as usize
            };
        }

        let switch_threads_stack_frame = new_thread.allocate_frame();
        unsafe {
            *switch_threads_stack_frame.as_ptr() = Context {
                edi: 0,
                esi: 0,
                ebx: 0,
                ebp: 0,
                eip: prepare_thread as usize,
            };
        }

        // Hand off to the schedulers.
        new_thread.status = ThreadStatus::Ready;
        return new_thread;
    }

    pub fn allocate_stack_space(&mut self, bytes: usize) -> NonNull<u8> {

        return self.shift_stack_pointer_down(bytes);

    }

    pub fn allocate_frame(&mut self) -> NonNull<Context> {

        return self.shift_stack_pointer_down(size_of::<Context>()).cast::<Context>();

    }

    pub fn shift_stack_pointer_down(&mut self, amount: usize) -> NonNull<u8> {
        unsafe{
            let raw_pointer = self.stack_pointer.as_ptr().cast::<u8>();
            let new_pointer = NonNull::new(raw_pointer.sub(amount)).expect("Error shifting stack pointer.");
            self.stack_pointer = new_pointer;
            return self.stack_pointer;
        }
    }

}
