#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;

use kernel::interrupts::{self, InterruptStackFrame};
use kernel::testing::{QemuExitCode, exit_qemu};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start")]
pub extern "C" fn _start() -> ! {
    kernel::serial_print!("double_fault::handler_runs... \t");

    interrupts::init_idt();
    interrupts::set_double_fault_handler(test_double_fault_handler);

    // Page fault (#14) with no handler installed: the CPU fails to deliver it
    // and escalates to a double fault
    unsafe { *(0xDEADBEEF as *mut u8) = 42 };

    kernel::serial_println!("[failed] no double fault ocurred");
    exit_qemu(QemuExitCode::Failed);
    kernel::halt();
}

extern "x86-interrupt" fn test_double_fault_handler(
    _frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    kernel::serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    kernel::halt();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::testing::test_panic_handler(info)
}
