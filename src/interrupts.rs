use crate::println;

const BREAKPOINT: usize = 3;
const DOUBLE_FAULT: usize = 8;
const IDT_ENTRIES: usize = 256;

const PRESENT: u8 = 1 << 7;
const INTERRUPT_GATE: u8 = 0xE;

#[repr(C)]
#[derive(Clone, Copy)]
struct InterruptDescriptor64 {
    offset_1: u16,
    selector: u16,
    ist: u8,
    type_attributes: u8,
    offset_2: u16,
    offset_3: u32,
    zero: u32,
}

impl InterruptDescriptor64 {
    const fn missing() -> Self {
        InterruptDescriptor64 {
            offset_1: 0,
            selector: 0,
            ist: 0,
            type_attributes: 0,
            offset_2: 0,
            offset_3: 0,
            zero: 0,
        }
    }

    fn set_handler(&mut self, handler: extern "x86-interrupt" fn(InterruptStackFrame)) {
        let addr = handler as u64;
        self.offset_1 = addr as u16;
        self.offset_2 = (addr >> 16) as u16;
        self.offset_3 = (addr >> 32) as u32;

        self.selector = read_cs();
        self.ist = 0;
        self.type_attributes = PRESENT | INTERRUPT_GATE;
        self.zero = 0;
    }

    fn set_handler_with_error_code(
        &mut self,
        handler: extern "x86-interrupt" fn(InterruptStackFrame, u64) -> !,
    ) {
        let addr = handler as u64;
        self.offset_1 = addr as u16;
        self.offset_2 = (addr >> 16) as u16;
        self.offset_3 = (addr >> 32) as u32;

        self.selector = read_cs();
        self.ist = 0;
        self.type_attributes = PRESENT | INTERRUPT_GATE;
        self.zero = 0;
    }
}

fn read_cs() -> u16 {
    let cs: u16;
    unsafe { core::arch::asm!("mov {0:x}, cs", out(reg) cs, options(nomem, nostack)) };
    cs
}

static IDT: spin::Mutex<[InterruptDescriptor64; IDT_ENTRIES]> =
    spin::Mutex::new([InterruptDescriptor64::missing(); IDT_ENTRIES]);

#[repr(C, packed)]
struct IdtPointer {
    limit: u16,
    base: u64,
}

pub fn init_idt() {
    set_breakpoint_handler(breakpoint_handler);
    set_double_fault_handler(double_fault_handler);

    load_idt();
}

fn load_idt() {
    let idt = IDT.lock();

    let pointer = IdtPointer {
        limit: (core::mem::size_of::<[InterruptDescriptor64; IDT_ENTRIES]>() - 1) as u16,
        base: idt.as_ptr() as u64,
    };
    unsafe {
        core::arch::asm!("lidt [{}]", in(reg) &pointer, options(readonly, nostack));
    }
}

pub fn set_breakpoint_handler(handler: extern "x86-interrupt" fn(InterruptStackFrame)) {
    IDT.lock()[BREAKPOINT].set_handler(handler);
}

pub fn set_double_fault_handler(handler: extern "x86-interrupt" fn(InterruptStackFrame, u64) -> !) {
    IDT.lock()[DOUBLE_FAULT].set_handler_with_error_code(handler);
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

extern "x86-interrupt" fn breakpoint_handler(frame: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT");
    println!("{:#X?}", frame);
}

extern "x86-interrupt" fn double_fault_handler(frame: InterruptStackFrame, _error_code: u64) -> ! {
    println!("EXCEPTION: DOUBLE FAULT HANDLER");
    println!("{:#X?}", frame);
    crate::halt();
}

#[test_case]
fn test_breakpoint_exception() {
    unsafe { core::arch::asm!("int3") };
}
