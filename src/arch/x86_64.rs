use crate::arch::{CpuInfo, CpuInterrupt, Result};
use crate::memory;
use alloc::boxed::Box;
use core::arch::{asm, global_asm};
use core::fmt::Debug;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::ptr::{slice_from_raw_parts_mut, NonNull};
use seq_macro::seq;
use spin::mutex::{TicketMutex, TicketMutexGuard};

pub(crate) fn interrupt_num(int: CpuInterrupt) -> Option<u64> {
    match int {
        CpuInterrupt::DivideByZero => Some(0),
        CpuInterrupt::Debug => Some(1),
        CpuInterrupt::NonMaskableInterrupt => Some(2),
        CpuInterrupt::Breakpoint => Some(3),
        CpuInterrupt::Overflow => Some(4),
        CpuInterrupt::BoundRangeExceeded => Some(5),
        CpuInterrupt::InvalidOpcode => Some(6),
        CpuInterrupt::DeviceUnavailable => Some(7),
        CpuInterrupt::InvalidTss => Some(10),
        CpuInterrupt::SegmentNotPresent => Some(11),
        CpuInterrupt::StackSegmentFault => Some(12),
        CpuInterrupt::GeneralProtectionFault => Some(13),
        CpuInterrupt::PageFault => Some(14),
        CpuInterrupt::FloatingPointException => Some(16),
        CpuInterrupt::AlignmentCheck => Some(17),
        CpuInterrupt::MachineCheck => Some(18),
        CpuInterrupt::SimdException => Some(19),
        CpuInterrupt::VirtualizationException => Some(20),
        CpuInterrupt::ControlProtectionException => Some(21),
        CpuInterrupt::HypervisorInjectionException => Some(28),
        CpuInterrupt::VmmCommunicationException => Some(29),
        CpuInterrupt::SecurityException => Some(30),
        CpuInterrupt::Syscall => Some(128),
    }
}
pub fn interrupt_from_num(num: u64) -> Option<CpuInterrupt> {
    match num {
        0 => Some(CpuInterrupt::DivideByZero),
        1 => Some(CpuInterrupt::Debug),
        2 => Some(CpuInterrupt::NonMaskableInterrupt),
        3 => Some(CpuInterrupt::Breakpoint),
        4 => Some(CpuInterrupt::Overflow),
        5 => Some(CpuInterrupt::BoundRangeExceeded),
        6 => Some(CpuInterrupt::InvalidOpcode),
        7 => Some(CpuInterrupt::DeviceUnavailable),
        10 => Some(CpuInterrupt::InvalidTss),
        11 => Some(CpuInterrupt::SegmentNotPresent),
        12 => Some(CpuInterrupt::StackSegmentFault),
        13 => Some(CpuInterrupt::GeneralProtectionFault),
        14 => Some(CpuInterrupt::PageFault),
        16 => Some(CpuInterrupt::FloatingPointException),
        17 => Some(CpuInterrupt::AlignmentCheck),
        18 => Some(CpuInterrupt::MachineCheck),
        19 => Some(CpuInterrupt::SimdException),
        20 => Some(CpuInterrupt::VirtualizationException),
        21 => Some(CpuInterrupt::ControlProtectionException),
        28 => Some(CpuInterrupt::HypervisorInjectionException),
        29 => Some(CpuInterrupt::VmmCommunicationException),
        30 => Some(CpuInterrupt::SecurityException),
        128 => Some(CpuInterrupt::Syscall),
        _ => None,
    }
}

extern "x86-interrupt" {
    seq!(N in 0..=255 { fn int_~N(); });
}

struct PageTableEntry {
    data: u64,
}
impl PageTableEntry {
    pub fn new() -> Self {
        PageTableEntry { data: 0 }
    }
    pub fn from_data(data: u64) -> Self {
        PageTableEntry { data }
    }
    pub fn set_present(&mut self, present: bool) {
        self.data &= !(1 << 0);
        self.data |= present as u64;
    }
    pub fn is_present(&self) -> bool {
        self.data & 1 == 1
    }
    pub fn set_writeable(&mut self, writeable: bool) {
        self.data &= !(1 << 1);
        self.data |= (writeable as u64) << 1;
    }
    pub fn is_writeable(&self) -> bool {
        (self.data >> 1) & 1 == 1
    }
    pub fn set_user(&mut self, user: bool) {
        self.data &= !(1 << 2);
        self.data |= (user as u64) << 2;
    }
    pub fn is_user(&self) -> bool {
        (self.data >> 2) & 1 == 1
    }
    pub fn set_addr(&mut self, mut addr: usize) {
        addr &= 0x000ffffffffff000;
        self.data &= !0x000ffffffffff000;
        self.data |= addr as u64;
    }
    pub fn get_addr(&self) -> u64 {
        self.data & 0x000ffffffffff000
    }
    pub fn set_global(&mut self, global: bool) {
        self.data &= !(1 << 8);
        self.data |= (global as u64) << 8;
    }
    pub fn is_global(&self) -> bool {
        (self.data >> 8) & 1 == 1
    }
    pub fn set_execute(&mut self, execute: bool) {
        self.data &= !(1 << 63);
        self.data |= (execute as u64) << 63;
    }
    pub fn is_execute(&self) -> bool {
        (self.data >> 63) & 1 == 1
    }
}

pub struct PageTable {
    pml4: [u64; 512],
}
impl PageTable {
    pub fn new() -> Self {
        PageTable { pml4: [0; 512] }
    }
    pub fn map_page(&mut self, virtual_addr: usize, physical_addr: usize) {
        unsafe {
            let pml4_index = virtual_addr >> 39;
            let mut pml4_entry = PageTableEntry::from_data(self.pml4[pml4_index]);
            if !pml4_entry.is_present() {
                if let Some(allocator) = &mut memory::ALLOCATOR {
                    let page = allocator.alloc().unwrap();
                    core::ptr::write_bytes(page as *mut u8, 0, 0x1000);
                    pml4_entry.set_addr(page as *mut _ as usize);
                    pml4_entry.set_present(true);
                } else {
                    unimplemented!("Can't allocate memory");
                }
            }
            let pml3 = &mut *slice_from_raw_parts_mut(pml4_entry.get_addr() as *mut u64, 512);
            let pml3_index = (virtual_addr >> 30) & 0x1ff;
            let mut pml3_entry = PageTableEntry::from_data(pml3[pml3_index]);
            if !pml3_entry.is_present() {
                if let Some(allocator) = &mut memory::ALLOCATOR {
                    let page = allocator.alloc().unwrap();
                    core::ptr::write_bytes(page as *mut u8, 0, 0x1000);
                    pml3_entry.set_addr(page as *mut _ as usize);
                    pml3_entry.set_present(true);
                } else {
                    unimplemented!("Can't allocate memory");
                }
            }
            let pml2 = &mut *slice_from_raw_parts_mut(pml3_entry.get_addr() as *mut u64, 512);
            let pml2_index = (virtual_addr >> 21) & 0x1ff;
            let mut pml2_entry = PageTableEntry::from_data(pml2[pml2_index]);
            if !pml2_entry.is_present() {
                if let Some(allocator) = &mut memory::ALLOCATOR {
                    let page = allocator.alloc().unwrap();
                    core::ptr::write_bytes(page as *mut u8, 0, 0x1000);
                    pml2_entry.set_addr(page as *mut _ as usize);
                    pml2_entry.set_present(true);
                } else {
                    unimplemented!("Can't allocate memory");
                }
            }
            let pml1 = &mut *slice_from_raw_parts_mut(pml2_entry.get_addr() as *mut u64, 512);
            let pml1_index = (virtual_addr >> 12) & 0x1ff;
            let mut pml1_entry = PageTableEntry::new();
            pml1_entry.set_present(true);
            pml1_entry.set_addr(physical_addr);
            pml1[pml1_index] = pml1_entry.data;
        }
    }
    pub fn unmap_page(&mut self, virtual_addr: usize) {
        unsafe {
            let pml4_index = virtual_addr >> 39;
            let pml4_entry = PageTableEntry::from_data(self.pml4[pml4_index]);
            if !pml4_entry.is_present() {
                return;
            }
            let pml3 = &mut *slice_from_raw_parts_mut(pml4_entry.get_addr() as *mut u64, 512);
            let pml3_index = (virtual_addr >> 30) & 0x1ff;
            let pml3_entry = PageTableEntry::from_data(pml3[pml3_index]);
            if !pml3_entry.is_present() {
                return;
            }
            let pml2 = &mut *slice_from_raw_parts_mut(pml3_entry.get_addr() as *mut u64, 512);
            let pml2_index = (virtual_addr >> 21) & 0x1ff;
            let pml2_entry = PageTableEntry::from_data(pml2[pml2_index]);
            if !pml2_entry.is_present() {
                return;
            }
            let pml1 = &mut *slice_from_raw_parts_mut(pml2_entry.get_addr() as *mut u64, 512);
            let pml1_index = (virtual_addr >> 12) & 0x1ff;
            pml1[pml1_index] = 0;
        }
    }
    pub fn get_physical_addr(&self, virtual_addr: usize) -> Option<usize> {
        unsafe {
            let pml4_index = virtual_addr >> 39;
            let pml4_entry = PageTableEntry::from_data(self.pml4[pml4_index]);
            if !pml4_entry.is_present() {
                return None;
            }
            let pml3 = &mut *slice_from_raw_parts_mut(pml4_entry.get_addr() as *mut u64, 512);
            let pml3_index = (virtual_addr >> 30) & 0x1ff;
            let pml3_entry = PageTableEntry::from_data(pml3[pml3_index]);
            if !pml3_entry.is_present() {
                return None;
            }
            let pml2 = &mut *slice_from_raw_parts_mut(pml3_entry.get_addr() as *mut u64, 512);
            let pml2_index = (virtual_addr >> 21) & 0x1ff;
            let pml2_entry = PageTableEntry::from_data(pml2[pml2_index]);
            if !pml2_entry.is_present() {
                return None;
            }
            let pml1 = &mut *slice_from_raw_parts_mut(pml2_entry.get_addr() as *mut u64, 512);
            let pml1_index = (virtual_addr >> 12) & 0x1ff;
            let pml1_entry = PageTableEntry::from_data(pml1[pml1_index]);
            if !pml1_entry.is_present() {
                return None;
            }
            Some(pml1_entry.get_addr() as usize)
        }
    }
}

pub struct Util {}
impl Util {
    pub fn init() -> Result<()> {
        unsafe {
            // GDT
            let gdt = &mut *slice_from_raw_parts_mut(
                memory::ALLOCATOR.as_mut().unwrap().alloc().unwrap() as *mut u8 as *mut usize,
                512,
            );
            gdt[0] = 0x00; // Null descriptor

            gdt[1] = 0x00aff3000000ffff; // Kernel code 16, 0x08
            gdt[2] = 0x000093000000ffff; // Kernel data 16, 0x10

            gdt[3] = 0x00cf9a000000ffff; // Kernel code 32, 0x18
            gdt[4] = 0x00cf93000000ffff; // Kernel data 32, 0x20

            gdt[5] = 0x00af9b000000ffff; // Kernel code 64, 0x28
            gdt[6] = 0x00af93000000ffff; // Kernel data 64, 0x30

            gdt[7] = 0x00cffa000000ffff; // User code 32, 0x38
            gdt[8] = 0x00cff3000000ffff; // User data 32, 0x40

            gdt[9] = 0x00affb000000ffff; // User code 64, 0x48
            gdt[10] = 0x00aff3000000ffff; // User data 64, 0x50

            // GDTR
            let gdtr_addr = &mut gdt[510] as *mut usize as *mut u128;
            *gdtr_addr = 0x50 | (gdt as *mut _ as *mut usize as u128) << 16;
            asm!("lgdt [{0}]", in(reg) gdtr_addr);

            asm!(
                "mov {1}, rsp",
                "push 0x30",
                "push {1}",
                "pushf",
                "push 0x28",
                "lea {1}, [rip + 2f]",
                "push {1}",
                "iretq",
                "2:",
                "mov ds, {0:x}",
                "mov es, {0:x}",
                "mov fs, {0:x}",
                "mov gs, {0:x}",
                in(reg) 0x30,
                out(reg) _
            );
        }
        Ok(())
    }
    pub fn halt_loop() -> ! {
        loop {
            unsafe {
                asm!("hlt", options(nomem, nostack));
            }
        }
    }
}

pub struct ExceptionStackFrame {
    pub instruction_pointer: u64,
    pub code_segment: u64,
    pub cpu_flags: u64,
    pub stack_pointer: u64,
    pub stack_segment: u64,
}

#[derive(Copy, Clone)]
pub struct InterruptDescriptor {
    data: u128,
}
impl InterruptDescriptor {
    pub fn new(
        handler: *mut u8,
        segment: usize,
        ist: u8,
        gate_type: u8,
        privilege_level: u8,
    ) -> Self {
        let mut data = 0;
        // Offset
        data |= (handler as usize & 0xFFFF) as u128;
        data |= (((handler as usize >> 16) & 0xFFFF) as u128) << 48;
        data |= ((handler as usize >> 32) as u128) << 64;
        // Segment
        data |= (segment as u128) << 16;
        // IST
        data |= (ist as u128 & 0b111) << 32;
        // Gate type
        data |= (gate_type as u128 & 0xF) << 40;
        // Privilege level
        data |= (privilege_level as u128 & 0b11) << 45;
        // Present
        data |= 1 << 47;
        Self { data }
    }
    pub fn none() -> Self {
        Self { data: 0 }
    }
}

#[repr(C, packed)]
pub struct InterruptTableDescriptor {
    pub size: u16,
    pub address: *mut InterruptDescriptor,
}

global_asm!(include_str!("interrupt.asm"));

// Stack on interrupt:
//     ptr   | register
// --------------------
// rsp + 168 | ss
// rsp + 160 | rsp
// rsp + 152 | rflags
// rsp + 144 | cs
// rsp + 136 | rip
// rsp + 128 | error_code
// rsp + 120 | interrupt_number
// rsp + 112 | rax
// rsp + 104 | rbx
// rsp + 96  | rcx
// rsp + 88  | rdx
// rsp + 80  | rsi
// rsp + 72  | rdi
// rsp + 64  | rbp
// rsp + 56  | r8
// rsp + 48  | r9
// rsp + 40  | r10
// rsp + 32  | r11
// rsp + 24  | r12
// rsp + 16  | r13
// rsp + 8   | r14
// rsp + 0   | r15
#[no_mangle]
#[naked]
unsafe fn int_handle() -> ! {
    asm!(
        "push rax",
        "push rbx",
        "push rcx",
        "push rdx",
        "push rsi",
        "push rdi",
        "push rbp",
        "push r8",
        "push r9",
        "push r10",
        "push r11",
        "push r12",
        "push r13",
        "push r14",
        "push r15",
        "lea rdi, [rsp]",
        "cld",
        "call {int_handle_rust}",
        "pop r15",
        "pop r14",
        "pop r13",
        "pop r12",
        "pop r11",
        "pop r10",
        "pop r9",
        "pop r8",
        "pop rbp",
        "pop rdi",
        "pop rsi",
        "pop rdx",
        "pop rcx",
        "pop rbx",
        "pop rax",
        "add rsp, 16",
        "iretq",
        int_handle_rust = sym int_handle_rust,
        options(noreturn)
    )
}
extern "sysv64" fn int_handle_rust(interrupt_param: &mut InterruptParam) {
    if let Some(handler) = Cpu::get_current_cpu().unwrap().interrupt_table().0.handlers
        [interrupt_param.interrupt_number as usize]
    {
        let mut state = super::CpuState(CpuState {
            ss: interrupt_param.ss,
            rsp: interrupt_param.rsp,
            rflags: interrupt_param.rflags,
            cs: interrupt_param.cs,
            rip: interrupt_param.rip,
            rax: interrupt_param.rax,
            rbx: interrupt_param.rbx,
            rcx: interrupt_param.rcx,
            rdx: interrupt_param.rdx,
            rsi: interrupt_param.rsi,
            rdi: interrupt_param.rdi,
            rbp: interrupt_param.rbp,
            r8: interrupt_param.r8,
            r9: interrupt_param.r9,
            r10: interrupt_param.r10,
            r11: interrupt_param.r11,
            r12: interrupt_param.r12,
            r13: interrupt_param.r13,
            r14: interrupt_param.r14,
            r15: interrupt_param.r15,
        });
        handler(
            match interrupt_param.interrupt_number {
                10 | 11 | 12 | 13 | 14 | 29 | 30 => Some(super::ErrorCode(ErrorCode::from(
                    interrupt_param.error_code,
                    interrupt_param.interrupt_number,
                ))),
                _ => None,
            },
            CpuInterrupt::try_from(interrupt_param.interrupt_number).unwrap(),
            &mut state,
        );
        interrupt_param.ss = state.0.ss;
        interrupt_param.rsp = state.0.rsp;
        interrupt_param.rflags = state.0.rflags;
        interrupt_param.cs = state.0.cs;
        interrupt_param.rip = state.0.rip;
        interrupt_param.rax = state.0.rax;
        interrupt_param.rbx = state.0.rbx;
        interrupt_param.rcx = state.0.rcx;
        interrupt_param.rdx = state.0.rdx;
        interrupt_param.rsi = state.0.rsi;
        interrupt_param.rdi = state.0.rdi;
        interrupt_param.rbp = state.0.rbp;
        interrupt_param.r8 = state.0.r8;
        interrupt_param.r9 = state.0.r9;
        interrupt_param.r10 = state.0.r10;
        interrupt_param.r11 = state.0.r11;
        interrupt_param.r12 = state.0.r12;
        interrupt_param.r13 = state.0.r13;
        interrupt_param.r14 = state.0.r14;
        interrupt_param.r15 = state.0.r15;
    } else {
        panic!(
            "No handler for interrupt 0x{:x}",
            interrupt_param.interrupt_number
        );
    }
}

#[repr(C)]
struct InterruptParam {
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    r11: u64,
    r10: u64,
    r9: u64,
    r8: u64,
    rbp: u64,
    rdi: u64,
    rsi: u64,
    rdx: u64,
    rcx: u64,
    rbx: u64,
    rax: u64,
    interrupt_number: u64,
    error_code: u64,
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

pub struct CpuState {
    ss: u64,
    rsp: u64,
    rflags: u64,
    cs: u64,
    rip: u64,
    rax: u64,
    rbx: u64,
    rcx: u64,
    rdx: u64,
    rsi: u64,
    rdi: u64,
    rbp: u64,
    r8: u64,
    r9: u64,
    r10: u64,
    r11: u64,
    r12: u64,
    r13: u64,
    r14: u64,
    r15: u64,
}
impl CpuState {
    pub fn get_ip(&self) -> usize {
        self.rip as usize
    }
    pub fn set_ip(&mut self, ip: usize) {
        self.rip = ip as u64;
    }
}

pub struct InterruptTable {
    pub descriptor: InterruptTableDescriptor,
    pub handlers: [Option<fn(Option<super::ErrorCode>, CpuInterrupt, &mut super::CpuState)>; 256],
}
impl InterruptTable {
    pub fn set_interrupt_handler(
        &mut self,
        interrupt: CpuInterrupt,
        handler: fn(Option<super::ErrorCode>, CpuInterrupt, &mut super::CpuState),
    ) -> Result<()> {
        self.handlers[u64::try_from(interrupt)? as usize] = Some(handler);
        Ok(())
    }

    pub fn new() -> Self {
        let allocator = unsafe { memory::ALLOCATOR.as_mut().unwrap() };
        let data = slice_from_raw_parts_mut(
            allocator.alloc().unwrap() as *mut u8 as *mut InterruptDescriptor,
            4096,
        );
        unsafe {
            core::ptr::write_bytes(data as *mut InterruptDescriptor, 0, 4096);
        }

        unsafe {
            seq!(N in 0..=255 {
                (&mut *data)[N] = InterruptDescriptor::new(
                    int_~N as *mut unsafe extern "x86-interrupt" fn() as *mut u8,
                    0x28,
                    0,
                    0xE,
                    0,
                );
            });
        }

        let descriptor = InterruptTableDescriptor {
            size: 0xFFF,
            address: data as *mut InterruptDescriptor,
        };
        Self {
            descriptor,
            handlers: [None; 256],
        }
    }
}

static mut INTERRUPT_TABLE: Option<*mut InterruptTable> = None;

#[derive(Debug)]
pub struct SegmentError {
    external: bool,
    table: u8,
    index: u16,
}
impl From<u64> for SegmentError {
    fn from(error_code: u64) -> Self {
        Self {
            external: error_code & 0b01 > 0,
            table: ((error_code >> 1) & 0b11) as u8,
            index: ((error_code >> 3) & 0x1FFF) as u16,
        }
    }
}

#[derive(Debug)]
pub enum ErrorCode {
    InvalidTss(SegmentError),
    SegmentNotPresent(SegmentError),
    StackSegmentFault(SegmentError),
    GeneralProtectionFault(SegmentError),
    PageFault {
        present: bool,
        write: bool,
        user: bool,
        reserved_write: bool,
        instruction_fetch: bool,
        protection_key: bool,
        shadow_stack: bool,
        software_guard_extensions: bool,
        address: usize,
    },
    ControlProtectionException(u8),
    VmmCommunicationException(u8),
    SecurityException(u8),
}
impl ErrorCode {
    pub fn from(error_code: u64, interrupt_num: u64) -> Self {
        match interrupt_num {
            10 => ErrorCode::InvalidTss(SegmentError::from(error_code)),
            11 => ErrorCode::SegmentNotPresent(SegmentError::from(error_code)),
            12 => ErrorCode::StackSegmentFault(SegmentError::from(error_code)),
            13 => ErrorCode::GeneralProtectionFault(SegmentError::from(error_code)),
            14 => {
                let mut address: usize;
                unsafe {
                    asm!("mov {0}, cr2", out(reg) address);
                }
                ErrorCode::PageFault {
                    present: error_code & 0x01 > 0,
                    write: error_code & 0x02 > 0,
                    user: error_code & 0x04 > 0,
                    reserved_write: error_code & 0x08 > 0,
                    instruction_fetch: error_code & 0x10 > 0,
                    protection_key: error_code & 0x20 > 0,
                    shadow_stack: error_code & 0x40 > 0,
                    software_guard_extensions: error_code & 0x4000 > 0,
                    address,
                }
            }
            29 => ErrorCode::VmmCommunicationException(error_code as u8),
            30 => ErrorCode::SecurityException(error_code as u8),
            _ => panic!(
                "Invalid error code {} for interrupt {}",
                error_code, interrupt_num
            ),
        }
    }
}

pub struct Cpu<'a> {
    pointer: Option<NonNull<super::Cpu<'a>>>,
    mutex: TicketMutex<()>,
    info: CpuInfo,
    page_table: PhantomData<&'a mut super::PageTable>,
    interrupt_table: &'a mut super::InterruptTable,
}
impl<'a> Cpu<'a> {
    pub fn new(
        page_table: &'a mut super::PageTable,
        interrupt_table: &'a mut super::InterruptTable,
        cpu_info: CpuInfo,
    ) -> Self {
        let this = Self {
            mutex: TicketMutex::new(()),
            pointer: None,
            info: cpu_info,
            page_table: PhantomData,
            interrupt_table,
        };
        this.set_page_table(page_table);
        this
    }
    pub fn info(&self) -> &CpuInfo {
        &self.info
    }
    pub fn page_table(&mut self) -> &mut super::PageTable {
        let page_table: *mut super::PageTable;
        unsafe {
            asm!("mov {0}, cr3", out(reg) page_table);
            &mut *page_table
        }
    }
    pub fn interrupt_table(&self) -> &super::InterruptTable {
        self.interrupt_table
    }
    pub fn set_page_table(&self, page_table: &'a mut super::PageTable) {
        let page_table_ptr = page_table as *mut super::PageTable;
        unsafe {
            asm!("mov {0}, cr3", in(reg) page_table_ptr);
        }
    }
    pub fn set_interrupt_table(&mut self, interrupt_table: &'a mut super::InterruptTable) {
        self.interrupt_table = interrupt_table;
        unsafe {
            asm!("lidt [{0}]", in(reg) &mut self.interrupt_table.0.descriptor);
        }
    }
    pub fn set_as_current_cpu(self) {
        let ptr = Box::into_raw(Box::new(self));
        unsafe {
            (&mut *ptr).pointer = NonNull::new(core::mem::transmute(ptr));
            asm!("wrgsbase {0}", in(reg) ptr);
        }
    }
    pub fn release_current_cpu() -> Option<super::Cpu<'a>> {
        let mut cpu = Self::get_current_cpu()?;
        unsafe {
            asm!("wrgsbase {0}", in(reg) 0);
        }
        let cpu = cpu.deref_mut();
        let mut cpu_box = unsafe { Box::from_raw(cpu as *mut super::Cpu) };
        (*cpu_box).0.pointer = None;
        Some(*cpu_box)
    }
    pub fn get_current_cpu() -> Option<super::CpuGuard<'a>> {
        let out: *mut super::Cpu<'a>;
        unsafe {
            asm!("mov {0}, gs:[0]", out(reg) out);
            let cpu =
                core::mem::transmute::<*mut super::Cpu, Option<NonNull<super::Cpu<'a>>>>(out)?;
            let guard = (&*out).0.mutex.lock();
            Some(super::CpuGuard(CpuGuard { cpu, guard }))
        }
    }
}
impl<'a> Drop for Cpu<'a> {
    fn drop(&mut self) {
        unsafe {
            asm!("wrgsbase {0}", in(reg) 0);
            asm!("mov {0}, cr3", in(reg) 0);
        }
    }
}

pub struct CpuGuard<'a> {
    cpu: NonNull<super::Cpu<'a>>,
    guard: TicketMutexGuard<'a, ()>,
}
impl<'a> Deref for CpuGuard<'a> {
    type Target = super::Cpu<'a>;
    fn deref(&self) -> &'a Self::Target {
        unsafe { self.cpu.as_ref() }
    }
}
impl<'a> DerefMut for CpuGuard<'a> {
    fn deref_mut(&mut self) -> &'a mut Self::Target {
        unsafe { self.cpu.as_mut() }
    }
}
