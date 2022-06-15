use crate::arch::traits::{CpuStateTrait, InterruptManagerTrait, InterruptTableTrait, UtilTrait};
use crate::arch::{traits, Error};
use crate::memory;
use core::arch::asm;
use core::ptr::slice_from_raw_parts_mut;

pub struct PagingManager {}
impl traits::PagingManagerTrait for PagingManager {
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
impl traits::PageTableTrait for PageTable {
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

#[repr(C, align(8))]
pub struct CpuState {
    pub rsp: u64,
    pub rbp: u64,
    pub rax: u64,
    pub rbx: u64,
    pub rcx: u64,
    pub rdx: u64,
    pub rsi: u64,
    pub rdi: u64,
    pub r8: u64,
    pub r9: u64,
    pub r10: u64,
    pub r11: u64,
    pub r12: u64,
    pub r13: u64,
    pub r14: u64,
    pub r15: u64,
    pub simd: *mut SimdState,
}
impl CpuStateTrait for CpuState {
    fn save() -> Self {
        unsafe {
            let simd_size = 512;
            let rsp;
            asm!("mov {0}, rsp", out(reg) rsp);
            let rbp;
            asm!("mov {0}, rbp", out(reg) rbp);
            let rax;
            asm!("mov {0}, rax", out(reg) rax);
            let rbx;
            asm!("mov {0}, rbx", out(reg) rbx);
            let rcx;
            asm!("mov {0}, rcx", out(reg) rcx);
            let rdx;
            asm!("mov {0}, rdx", out(reg) rdx);
            let rsi;
            asm!("mov {0}, rsi", out(reg) rsi);
            let rdi;
            asm!("mov {0}, rdi", out(reg) rdi);
            let r8;
            asm!("mov {0}, r8", out(reg) r8);
            let r9;
            asm!("mov {0}, r9", out(reg) r9);
            let r10;
            asm!("mov {0}, r10", out(reg) r10);
            let r11;
            asm!("mov {0}, r11", out(reg) r11);
            let r12;
            asm!("mov {0}, r12", out(reg) r12);
            let r13;
            asm!("mov {0}, r13", out(reg) r13);
            let r14;
            asm!("mov {0}, r14", out(reg) r14);
            let r15;
            asm!("mov {0}, r15", out(reg) r15);
            CpuState {
                rsp,
                rbp,
                rax,
                rbx,
                rcx,
                rdx,
                rsi,
                rdi,
                r8,
                r9,
                r10,
                r11,
                r12,
                r13,
                r14,
                r15,
                simd: SimdState::alloc(simd_size),
            }
        }
    }

    fn load(&self) {
        unsafe {
            asm!("mov rsp, {0}", in(reg) self.rsp);
            asm!("mov rbp, {0}", in(reg) self.rbp);
            asm!("mov rax, {0}", in(reg) self.rax);
            asm!("mov rbx, {0}", in(reg) self.rbx);
            asm!("mov rcx, {0}", in(reg) self.rcx);
            asm!("mov rdx, {0}", in(reg) self.rdx);
            asm!("mov rsi, {0}", in(reg) self.rsi);
            asm!("mov rdi, {0}", in(reg) self.rdi);
            asm!("mov r8, {0}", in(reg) self.r8);
            asm!("mov r9, {0}", in(reg) self.r9);
            asm!("mov r10, {0}", in(reg) self.r10);
            asm!("mov r11, {0}", in(reg) self.r11);
            asm!("mov r12, {0}", in(reg) self.r12);
            asm!("mov r13, {0}", in(reg) self.r13);
            asm!("mov r14, {0}", in(reg) self.r14);
            asm!("mov r15, {0}", in(reg) self.r15);
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
        handler: extern "x86-interrupt" fn(ExceptionStackFrame),
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

#[derive(PartialEq)]
pub struct InterruptTableDescriptor {
    pub size: u16,
    pub address: *mut [InterruptDescriptor],
}

pub struct InterruptTable {
    pub descriptor: InterruptTableDescriptor,
}
impl InterruptTableTrait for InterruptTable {
    type HandlerFn = extern "x86-interrupt" fn(ExceptionStackFrame);

    fn set_interrupt_handler(&mut self, interrupt_num: usize, handler: Self::HandlerFn) {
        (unsafe { &mut *(self.descriptor.address) })[interrupt_num] =
            InterruptDescriptor::new(handler, 0x28, 0, 0b1110, 0);
    }

    fn new() -> Self {
        let allocator = unsafe { memory::ALLOCATOR.as_mut().unwrap() };
        let data =
            slice_from_raw_parts_mut(allocator.alloc().unwrap() as *mut InterruptDescriptor, 4096);
        unsafe {
            core::ptr::write_bytes(data as *mut InterruptDescriptor, 0, 4096);
        }

        let descriptor = InterruptTableDescriptor {
            size: 0xFFE,
            address: data,
        };
        Self { descriptor }
    }
}

pub struct InterruptManager {}
impl InterruptManagerTrait for InterruptManager {
    type InterruptTable = InterruptTable;

    fn set_interrupt_table(interrupt_table: &Self::InterruptTable) -> Result<(), Error> {
        unsafe {
            asm!("lidt [{0}]", in(reg) &interrupt_table.descriptor as *const InterruptTableDescriptor);
        }
        Ok(())
    }

    fn get_interrupt_table<'a>() -> Result<&'a mut Self::InterruptTable, Error> {
        todo!()
    }
}
