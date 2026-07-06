#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::testing::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![cfg_attr(test, allow(dead_code))]

mod serial;
#[cfg(test)]
mod testing;
mod vga;

use core::arch::asm;
#[cfg(not(test))]
use core::panic::PanicInfo;

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start")]
pub extern "C" fn _start() -> ! {
    #[cfg(not(test))]
    kernel_main();

    #[cfg(test)]
    test_main();

    halt();
}

#[cfg(not(test))]
fn kernel_main() {
    vga::clear_screen();
    println!("x = {}, y = {}", 2 + 2, 3 + 3);
}

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    halt();
}

pub fn halt() -> ! {
    loop {
        unsafe {
            asm!("hlt");
        }
    }
}

#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}
