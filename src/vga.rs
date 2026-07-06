use core::fmt::{self, Write};

// screen geometry
const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const LAST_LINE: usize = HEIGHT - 1;

// VGA text buffer
const VGA_ADDR: *mut ScreenChar = 0xB8000 as *mut ScreenChar;
const BACKGROUND_SHIFT: u8 = 4; // background color goes in the high nibble

// character handling
const ASCII_PRINTABLE: core::ops::RangeInclusive<u8> = 0x20..=0x7E;
const REPLACEMENT_CHAR: u8 = 0xFE; // ■ shown for non-printable bytes

pub static WRITER: spin::Mutex<Buffer> = spin::Mutex::new(Buffer::new());

#[allow(unused)]
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    LightMagenta = 13,
    Yellow = 14,
    White = 15,
}

#[doc(hidden)]
pub fn _print(args: core::fmt::Arguments) {
    WRITER.lock().write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[derive(Clone, Copy)]
#[allow(unused)]
pub struct ColorCode(u8);

impl ColorCode {
    pub const fn new(fg: Color, bg: Color) -> Self {
        ColorCode((bg as u8) << BACKGROUND_SHIFT | (fg as u8))
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
struct ScreenChar {
    ascii: u8,
    color: ColorCode,
}

impl ScreenChar {
    fn new(ascii: u8, color: ColorCode) -> Self {
        Self { ascii, color }
    }

    fn blank() -> Self {
        Self {
            ascii: b' ',
            color: ColorCode::new(Color::White, Color::Black),
        }
    }
}

pub struct Buffer {
    x: usize,
    y: usize,
    color: ColorCode,
}

impl fmt::Write for Buffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print_str(s);
        Ok(())
    }
}

impl Buffer {
    pub const fn new() -> Self {
        Self {
            x: 0,
            y: LAST_LINE,
            color: ColorCode::new(Color::White, Color::Black),
        }
    }

    pub fn print_str(&mut self, string: &str) {
        for c in string.chars() {
            self.print_char(c as u8);
        }
    }

    pub fn print_char(&mut self, c: u8) {
        match c {
            b'\n' => self.new_line(),
            c if ASCII_PRINTABLE.contains(&c) => {
                self.write_byte(c);
            }
            _ => {
                self.write_byte(REPLACEMENT_CHAR);
            }
        }
    }

    fn clear_screen(&mut self) {
        for y in 0..HEIGHT {
            self.clear_line(y);
        }
        self.move_to_bottom_line();
    }

    #[allow(unused)]
    pub fn set_color(&mut self, color: ColorCode) {
        self.color = color;
    }

    fn write_byte(&mut self, c: u8) {
        let cell = ScreenChar::new(c, self.color);
        self.write_cell(self.x, self.y, cell);
        self.advance_position();
    }

    fn advance_position(&mut self) {
        self.x += 1;
        if self.x >= WIDTH {
            self.new_line();
        }
    }

    fn new_line(&mut self) {
        for y in 1..HEIGHT {
            for x in 0..WIDTH {
                let cell = self.read_cell(x, y);
                self.write_cell(x, y - 1, cell);
            }
        }
        self.clear_line(HEIGHT - 1);
        self.move_to_bottom_line();
    }

    fn clear_line(&mut self, y: usize) {
        let blank = ScreenChar::blank();

        for x in 0..WIDTH {
            self.write_cell(x, y, blank);
        }
    }

    fn move_to_bottom_line(&mut self) {
        self.x = 0;
        self.y = LAST_LINE;
    }

    fn write_cell(&mut self, x: usize, y: usize, cell: ScreenChar) {
        unsafe { VGA_ADDR.add(index(x, y)).write_volatile(cell) }
    }

    fn read_cell(&self, x: usize, y: usize) -> ScreenChar {
        unsafe { VGA_ADDR.add(index(x, y)).read_volatile() }
    }
}

pub fn clear_screen() {
    WRITER.lock().clear_screen();
}

fn index(x: usize, y: usize) -> usize {
    y * WIDTH + x
}

#[test_case]
fn color_code_packs_nibles() {
    assert_eq!(ColorCode::new(Color::White, Color::Black).0, 0x0F);
    assert_eq!(ColorCode::new(Color::Yellow, Color::Blue).0, 0x1E);
}

#[test_case]
fn print_char_writes_printable_and_replaces_the_rest() {
    let mut buf = Buffer::new();
    buf.print_char(b'A');
    assert_eq!(buf.read_cell(0, LAST_LINE).ascii, b'A');
    assert_eq!(buf.x, 1);

    buf.move_to_bottom_line();
    buf.print_char(0x07);
    assert_eq!(buf.read_cell(0, LAST_LINE).ascii, REPLACEMENT_CHAR);
}

#[test_case]
fn newline_scrolls_up_and_resets_cursor() {
    let mut buf = Buffer::new();
    buf.clear_screen();
    buf.write_cell(0, LAST_LINE, ScreenChar::new(b'M', buf.color));
    buf.print_char(b'\n');
    assert_eq!(buf.read_cell(0, LAST_LINE - 1).ascii, b'M');
    assert_eq!(buf.read_cell(0, LAST_LINE).ascii, b' ');
    assert_eq!(buf.x, 0);
    assert_eq!(buf.y, LAST_LINE);
}

#[test_case]
fn advance_wraps_at_width() {
    let mut buf = Buffer::new();
    for _ in 0..WIDTH {
        buf.print_char(b'.');
    }
    assert_eq!(buf.x, 0);
    assert_eq!(buf.y, LAST_LINE);
}

#[test_case]
fn clear_screen_blanks_all_and_resets_cursor() {
    let mut buf = Buffer::new();
    buf.write_cell(0, 0, ScreenChar::new(b'A', buf.color));
    buf.x = 5;
    buf.y = 5;
    buf.clear_screen();
    assert_eq!(buf.read_cell(0, 0).ascii, b' ');
    assert_eq!(buf.x, 0);
    assert_eq!(buf.y, LAST_LINE);
}

#[test_case]
fn println_reaches_the_screen() {
    let s = "test_output";
    clear_screen();
    println!("{}", s);
    for (i, c) in s.bytes().enumerate() {
        assert_eq!(WRITER.lock().read_cell(i, LAST_LINE - 1).ascii, c);
    }
}
