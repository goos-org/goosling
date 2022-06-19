use crate::arch::traits::{
    InterruptInfoTrait, InterruptManagerTrait, InterruptTableTrait, UtilTrait,
};
use crate::arch::{Error, InterruptHandler};
use crate::{memory, PageTableTrait, PagingManagerTrait};
use core::arch::{asm, global_asm};
use core::ptr::slice_from_raw_parts_mut;
use seq_macro::seq;

extern "x86-interrupt" {
    seq!(N in 0..=255 { fn int_~N(); });
}

pub struct PagingManager {}
impl PagingManagerTrait for PagingManager {
    type PageTable = PageTable;
    fn set_page_table(page_table: &Self::PageTable) -> Result<(), Error> {
        if page_table as *const PageTable as u64 % 0x1000 != 0 {
            return Err(Error::MisAligned);
        }
        unsafe {
            asm!("mov cr3, rax", in("rax") page_table as *const PageTable as u64);
        }
        Ok(())
    }

    fn get_page_table<'a>() -> Result<&'a mut PageTable, Error> {
        let addr: u64;
        unsafe { asm!("mov rax, cr3", out("rax") addr) };
        if addr % 0x1000 == 0 {
            Ok(unsafe { &mut *(addr as *mut PageTable) })
        } else {
            Err(Error::MisAligned)
        }
    }
}

pub struct PageTableEntry {
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
impl PageTableTrait for PageTable {
    fn map_page(&mut self, virtual_addr: usize, physical_addr: usize) {
        unsafe {
            let pml4_index = virtual_addr >> 39;
            let mut pml4_entry = PageTableEntry::from_data(self.pml4[pml4_index]);
            if !pml4_entry.is_present() {
                if let Some(allocator) = &mut memory::ALLOCATOR {
                    let page = allocator.alloc().unwrap();
                    core::ptr::write_bytes(page as *mut u8, 0, 0x1000);
                    pml4_entry.set_addr(page);
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
                    pml3_entry.set_addr(page);
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
                    pml2_entry.set_addr(page);
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
    fn unmap_page(&mut self, virtual_addr: usize) {
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
    fn get_physical_addr(&self, virtual_addr: usize) -> Option<usize> {
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

#[repr(C, align(64))]
pub struct SimdState {
    pub data: [u8],
}
impl SimdState {
    pub fn alloc(size: usize) -> *mut SimdState {
        if let Some(allocator) = unsafe { &mut memory::ALLOCATOR } {
            let page = allocator.alloc().unwrap();
            slice_from_raw_parts_mut(page as *mut u8, size) as *mut SimdState
        } else {
            unimplemented!("Can't allocate memory");
        }
    }
}

pub struct Util {}
impl UtilTrait for Util {
    fn init() -> Result<(), Error> {
        unsafe {
            let xsave_enabled = (core::arch::x86_64::__cpuid_count(1, 0).ecx & 1 << 26) != 0;
            if !xsave_enabled {
                return Err(Error::Unsupported);
            }
            let mut cr4: usize;
            asm!("mov {0}, cr4", out(reg) cr4);
            cr4 |= 1 << 18;
            asm!("mov cr4, {0}", in(reg) cr4);
        }
        Ok(())
    }
    fn halt_loop() -> ! {
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
impl PartialEq for InterruptTableDescriptor {
    fn eq(&self, other: &Self) -> bool {
        self.size == other.size && self.address as usize == other.address as usize
    }
}

seq!(N in 0..=255 { global_asm!(stringify!(int_~N: push 0x00; push N; jmp int_handle)); });
global_asm!(include_str!("interrupt.asm"));

// Stack on interrupt:
//     ptr   | register
// --------------------
// rsp + 176 | ss
// rsp + 168 | rsp
// rsp + 160 | rflags
// rsp + 152 | cs
// rsp + 144 | rip
// rsp + 136 | error_code
// rsp + 128 | interrupt_number
// rsp + 120 | rax
// rsp + 112 | rbx
// rsp + 104 | rcx
// rsp + 96  | rdx
// rsp + 88  | rsi
// rsp + 80  | rdi
// rsp + 72  | rbp
// rsp + 64  | r8
// rsp + 56  | r9
// rsp + 48  | r10
// rsp + 40  | r11
// rsp + 32  | r12
// rsp + 24  | r13
// rsp + 16  | r14
// rsp + 8   | r15
// rsp + 0   | (bottom of r15)
#[no_mangle]
extern "C" fn no_handler(_: usize, interrupt_num: usize) -> ! {
    panic!("No handler for interrupt {}", interrupt_num);
}

extern "C" {
    static mut HANDLERS: [u64; 256];
}

pub struct InterruptTable {
    pub descriptor: InterruptTableDescriptor,
    pub handlers: [Option<InterruptHandler>; 256],
}
impl InterruptTable {
    fn get_handler(&self, interrupt_number: u8) -> Option<InterruptHandler> {
        self.handlers[interrupt_number as usize]
    }
}
impl InterruptTableTrait for InterruptTable {
    fn set_interrupt_handler(&mut self, interrupt_num: usize, handler: InterruptHandler) {
        self.handlers[interrupt_num] = Some(handler);
    }

    fn new() -> Self {
        let allocator = unsafe { memory::ALLOCATOR.as_mut().unwrap() };
        let data =
            slice_from_raw_parts_mut(allocator.alloc().unwrap() as *mut InterruptDescriptor, 4096);
        unsafe {
            core::ptr::write_bytes(data as *mut InterruptDescriptor, 0, 4096);
        }

        unsafe {
            seq!(N in 0..=255 {
                (&mut *data)[N] = InterruptDescriptor::new(
                    int_~N as *mut unsafe extern "x86-interrupt" fn() as *mut u8,
                    0x30,
                    0,
                    0xE,
                    0,
                );
            });
        }

        let descriptor = InterruptTableDescriptor {
            size: 0xFFE,
            address: data as *mut InterruptDescriptor,
        };
        Self {
            descriptor,
            handlers: [None; 256],
        }
    }
}

static mut INTERRUPT_TABLE: Option<*mut InterruptTable> = None;

pub struct InterruptManager {}
impl InterruptManagerTrait for InterruptManager {
    type InterruptTable = InterruptTable;

    fn set_interrupt_table(interrupt_table: &mut Self::InterruptTable) -> Result<(), Error> {
        unsafe {
            INTERRUPT_TABLE = Some(interrupt_table);
            asm!("lidt [{0}]", in(reg) &interrupt_table.descriptor as *const InterruptTableDescriptor);
        }
        Ok(())
    }

    fn get_interrupt_table<'a>() -> Result<&'a mut Self::InterruptTable, Error> {
        Ok(unsafe { &mut *INTERRUPT_TABLE.ok_or(Error::Uninitialized)? })
    }

    fn enable_interrupts() {
        unsafe {
            asm!("sti", options(nostack, nomem));
        }
    }
}
