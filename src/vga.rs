const WIDTH: usize = 80;
const HEIGHT: usize = 25;
const VGA_ADDR: *mut ScreenChar = 0xB8000 as *mut ScreenChar;

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

#[derive(Clone, Copy)]
#[allow(unused)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(fg: Color, bg: Color) -> Self {
        ColorCode((bg as u8) << 4 | (fg as u8))
    }
}

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ScreenChar {
    ascii: u8,
    color: ColorCode,
}

pub struct Buffer {
    buffer: *mut ScreenChar,
    x: usize,
    y: usize,
}

impl Buffer {
    pub fn new() -> Self {
        let mut b = Self {
            buffer: VGA_ADDR,
            x: 0,
            y: 0,
        };
        b.reset_position();
        b
    }

    pub fn print_char(&mut self, c: char, color: ColorCode) {
        match c {
            '\n' => self.new_line(),
            _ => {
                let cell = ScreenChar {
                    ascii: c as u8,
                    color,
                };
                self.write_cell(self.x, self.y, cell);
                self.advance_position();
            } // TODO: print default char for invalids chars
        }
    }

    pub fn print_str(&mut self, string: &str, color: ColorCode) {
        for c in string.chars() {
            self.print_char(c, color);
        }
    }

    pub fn clear_screen(&mut self) {
        for y in 0..HEIGHT {
            self.clear_line(y);
        }
        self.reset_position();
    }

    fn clear_line(&mut self, y: usize) {
        let blank = ScreenChar {
            ascii: b' ',
            color: ColorCode::new(Color::White, Color::Black),
        };

        for x in 0..WIDTH {
            self.write_cell(x, y, blank);
        }
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
        self.reset_position();
    }

    fn reset_position(&mut self) {
        self.x = 0;
        self.y = HEIGHT - 1;
    }

    fn write_cell(&mut self, x: usize, y: usize, cell: ScreenChar) {
        unsafe {
            self.buffer.add(y * WIDTH + x).write_volatile(cell);
        }
    }

    fn read_cell(&self, x: usize, y: usize) -> ScreenChar {
        unsafe { self.buffer.add(y * WIDTH + x).read_volatile() }
    }
}
