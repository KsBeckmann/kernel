use core::mem::size_of;
use core::ops::{Index, IndexMut};

pub const IDT_ENTRIES: usize = 256;

const PRESENT: u8 = 1 << 7;
const INTERRUPT_GATE: u8 = 0xE;

pub type HandlerFunc = extern "x86-interrupt" fn(InterruptStackFrame);

pub type DivergingHandlerFuncWithErrCode =
    extern "x86-interrupt" fn(InterruptStackFrame, u64) -> !;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct Entry {
    offset_1: u16,
    selector: u16,
    ist: u8,
    type_attributes: u8,
    offset_2: u16,
    offset_3: u32,
    zero: u32,
}

impl Entry {
    pub const fn missing() -> Self {
        Entry {
            offset_1: 0,
            selector: 0,
            ist: 0,
            type_attributes: 0,
            offset_2: 0,
            offset_3: 0,
            zero: 0,
        }
    }

    pub fn set_handler(&mut self, handler: HandlerFunc) {
        self.set_addr(handler as u64);
    }

    pub fn set_handler_with_error_code(&mut self, handler: DivergingHandlerFuncWithErrCode) {
        self.set_addr(handler as u64);
    }

    pub fn set_stack_index(&mut self, slot: u8) {
        self.ist = slot + 1;
    }

    fn set_addr(&mut self, addr: u64) {
        self.offset_1 = addr as u16;
        self.offset_2 = (addr >> 16) as u16;
        self.offset_3 = (addr >> 32) as u32;

        self.selector = read_cs();
        self.ist = 0;
        self.type_attributes = PRESENT | INTERRUPT_GATE;
        self.zero = 0;
    }
}

#[repr(C)]
pub struct InterruptDescriptorTable([Entry; IDT_ENTRIES]);

impl InterruptDescriptorTable {
    pub const fn new() -> Self {
        InterruptDescriptorTable([Entry::missing(); IDT_ENTRIES])
    }

    pub fn load(&'static self) {
        let pointer = Pointer {
            limit: (size_of::<Self>() - 1) as u16,
            base: self as *const Self as u64,
        };

        unsafe {
            core::arch::asm!("lidt [{}]", in(reg) &pointer, options(readonly, nostack));
        }
    }
}

impl Index<usize> for InterruptDescriptorTable {
    type Output = Entry;

    fn index(&self, vector: usize) -> &Entry {
        &self.0[vector]
    }
}

impl IndexMut<usize> for InterruptDescriptorTable {
    fn index_mut(&mut self, vector: usize) -> &mut Entry {
        &mut self.0[vector]
    }
}

#[repr(C, packed)]
struct Pointer {
    limit: u16,
    base: u64,
}

#[derive(Debug)]
#[repr(C)]
pub struct InterruptStackFrame {
    instruction_pointer: u64,
    code_segment: u64,
    cpu_flags: u64,
    stack_pointer: u64,
    stack_segment: u64,
}

fn read_cs() -> u16 {
    let cs: u16;
    unsafe { core::arch::asm!("mov {0:x}, cs", out(reg) cs, options(nomem, nostack)) };
    cs
}
