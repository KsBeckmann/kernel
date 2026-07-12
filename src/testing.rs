use crate::halt;
use crate::{serial_print, serial_println};
use core::panic::PanicInfo;

pub fn test_runner(tests: &[&dyn Testable]) {
    serial_println!("running {} test(s)", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

pub trait Testable {
    fn run(&self);
}

impl<T: Fn()> Testable for T {
    fn run(&self) {
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        serial_println!("[ok]");
    }
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    unsafe {
        core::arch::asm!(
            "out dx, eax",
            in("dx") 0xF4u16,
            in("eax") exit_code as u32,
            options(nomem, nostack, preserves_flags),
        );
    }
}

pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

// Shared panic handler for test binaries (lib unit tests + each integration test).
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n{}", info);
    exit_qemu(QemuExitCode::Failed);
    halt();
}
