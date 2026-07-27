use super::idt::InterruptStackFrame;
use crate::println;

pub extern "x86-interrupt" fn breakpoint(frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT");
    println!("{:#X?}", frame);
}

pub extern "x86-interrupt" fn double_fault(frame: InterruptStackFrame, _error_code: u64) -> ! {
    println!("EXCEPTION: DOUBLE FAULT");
    println!("{:#X?}", frame);
    crate::halt();
}
