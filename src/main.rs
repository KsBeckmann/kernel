#![no_std]
#![no_main]

mod vga;

use core::panic::PanicInfo;
use vga::{Color, ColorCode};

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.start")]
pub extern "C" fn _start() -> ! {
    let mut vga_buffer = vga::Buffer::new();
    let color = ColorCode::new(Color::LightRed, Color::Black);

    vga_buffer.clear_screen();
    vga_buffer.print_str("line 1\n", color);
    vga_buffer.print_str("line 2\n", color);
    vga_buffer.print_str("line 3\n", color);
    vga_buffer.print_str("line 4", color);

    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
