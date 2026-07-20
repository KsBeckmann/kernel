#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![feature(abi_x86_interrupt)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![cfg_attr(test, allow(dead_code))]

use core::arch::asm;

pub mod interrupts;
pub mod serial;
pub mod testing;
pub mod vga;

pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

// Entry point + panic handler used only when the library itself is tested
// (`cargo test` on the lib runs the vga unit tests).
#[cfg(test)]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start")]
pub extern "C" fn _start() -> ! {
    test_main();
    halt();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    testing::test_panic_handler(info)
}
