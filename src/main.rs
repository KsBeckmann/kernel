#![no_std]
#![no_main]

use core::panic::PanicInfo;

const HELLO: &str = "Hello from rust!";

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start")]
pub extern "C" fn _start() -> ! {
    let vga_buffer = 0xB8000 as *mut u8;

    for (i, byte) in HELLO.chars().enumerate() {
        unsafe {
            *vga_buffer.offset(i as isize * 2) = byte as u8;
            *vga_buffer.offset(i as isize * 2 + 1) = 0x0C;
        }
    }

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
