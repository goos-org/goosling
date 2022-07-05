mod x86_64;

use crate::arch::native::ErrorCode;
#[cfg(target_arch = "x86_64")]
use x86_64 as native;

type Result<T> = core::result::Result<T, Error>;

#[repr(transparent)]
pub struct PageTable(native::PageTable);
impl PageTable {
    pub fn map_page(&mut self, virtual_addr: usize, physical_addr: usize) {
        self.0.map_page(virtual_addr, physical_addr);
    }
    pub fn unmap_page(&mut self, virtual_addr: usize) {
        self.0.unmap_page(virtual_addr);
    }
    pub fn get_physical_addr(&self, virtual_addr: usize) -> Option<usize> {
        self.0.get_physical_addr(virtual_addr)
    }
}

#[repr(transparent)]
pub struct Util(native::Util);
impl Util {
    pub fn init() -> Result<()> {
        native::Util::init()
    }
    pub fn halt_loop() -> ! {
        native::Util::halt_loop()
    }
}

#[repr(transparent)]
pub struct CpuState(native::CpuState);
impl CpuState {
    pub fn get_ip(&self) -> usize {
        self.0.get_ip()
    }
    pub fn set_ip(&mut self, ip: usize) {
        self.0.set_ip(ip);
    }
}

#[repr(transparent)]
pub struct InterruptTable(native::InterruptTable);
impl InterruptTable {
    pub fn set_interrupt_handler(
        &mut self,
        interrupt_num: usize,
        handler: fn(Option<ErrorCode>, u64, &mut CpuState),
    ) {
        self.0.set_interrupt_handler(interrupt_num, handler);
    }
    pub fn new() -> Self {
        InterruptTable(native::InterruptTable::new())
    }
}

pub enum CpuInterrupt {
    DivideByZero,
    Debug,
    NonMaskableInterrupt,
    Breakpoint,
    Overflow,
    BoundRangeExceeded,
    InvalidOpcode,
    DeviceUnavailable,
    InvalidTss,
    SegmentNotPresent,
    StackSegmentFault,
    GeneralProtectionFault,
    PageFault,
    FloatingPointException,
    AlignmentCheck,
    MachineCheck,
    SimdException,
    VirtualizationException,
    ControlProtectionException,
    HypervisorInjectionException,
    VmmCommunicationException,
    SecurityException,
    Syscall,
}

#[derive(Debug)]
pub enum Error {
    MisAligned,
    Unsupported,
    Uninitialized,
}

pub struct CpuInfo {
    pub userspace: bool,
    pub cpu_id: usize,
}

pub struct Cpu<'a>(native::Cpu<'a>);
impl<'a> Cpu<'a> {
    pub fn info(&self) -> &CpuInfo {
        self.0.info()
    }
    pub fn page_table(&self) -> &PageTable {
        self.0.page_table()
    }
    pub fn interrupt_table(&self) -> &InterruptTable {
        self.0.interrupt_table()
    }
    pub fn set_page_table(&mut self, page_table: &mut PageTable) {
        self.0.set_page_table(page_table)
    }
    pub fn set_interrupt_table(&mut self, interrupt_table: &mut InterruptTable) {
        self.0.set_interrupt_table(interrupt_table)
    }
    pub fn set_as_current_cpu(&self) {
        self.0.set_as_current_cpu()
    }
    pub fn get_current_cpu() -> Option<&'a Cpu<'a>> {
        native::Cpu::get_current_cpu()
    }
}
