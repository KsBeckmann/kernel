pub mod idt;

mod handlers;

pub use idt::InterruptStackFrame;

use idt::{DivergingHandlerFuncWithErrCode, HandlerFunc, InterruptDescriptorTable};

const BREAKPOINT: usize = 3;
const DOUBLE_FAULT: usize = 8;

static IDT: spin::Mutex<InterruptDescriptorTable> =
    spin::Mutex::new(InterruptDescriptorTable::new());

pub fn init_idt() {
    set_breakpoint_handler(handlers::breakpoint);
    set_double_fault_handler(handlers::double_fault);

    load_idt();
}

pub fn set_breakpoint_handler(handler: HandlerFunc) {
    IDT.lock()[BREAKPOINT].set_handler(handler);
}

pub fn set_double_fault_handler(handler: DivergingHandlerFuncWithErrCode) {
    IDT.lock()[DOUBLE_FAULT].set_handler_with_error_code(handler);
}

fn load_idt() {
    let table = {
        let idt = IDT.lock();
        &*idt as *const InterruptDescriptorTable
    };

    unsafe { &*table }.load();
}

#[test_case]
fn test_breakpoint_exception() {
    unsafe { core::arch::asm!("int3") };
}
