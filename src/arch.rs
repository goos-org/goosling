mod x86_64;

use core::fmt::Debug;
#[cfg(target_arch = "x86_64")]
use x86_64 as native;

type Result<T> = core::result::Result<T, Error>;

#[repr(transparent)]
pub struct PageTable(native::PageTable);
impl PageTable {
    pub fn new() -> Self {
        PageTable(native::PageTable::new())
    }
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
        interrupt: CpuInterrupt,
        handler: fn(Option<ErrorCode>, CpuInterrupt, &mut CpuState),
    ) -> Result<()> {
        self.0.set_interrupt_handler(interrupt, handler)
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
impl TryFrom<CpuInterrupt> for u64 {
    type Error = Error;
    fn try_from(value: CpuInterrupt) -> core::result::Result<Self, Self::Error> {
        match native::interrupt_num(value) {
            Some(num) => Ok(num),
            None => Err(Error::Unsupported),
        }
    }
}
impl TryFrom<u64> for CpuInterrupt {
    type Error = Error;
    fn try_from(value: u64) -> core::result::Result<Self, Self::Error> {
        match native::interrupt_from_num(value) {
            Some(num) => Ok(num),
            None => Err(Error::Unsupported),
        }
    }
}

#[derive(Debug)]
pub enum Error {
    MisAligned,
    Unsupported,
    Uninitialized,
}

#[repr(transparent)]
pub struct ErrorCode(native::ErrorCode);
impl ErrorCode {
    pub fn from(error_code: u64, interrupt_num: u64) -> Self {
        ErrorCode(native::ErrorCode::from(error_code, interrupt_num))
    }
}
impl Debug for ErrorCode {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.0.fmt(f)
    }
}

pub struct CpuInfo {
    pub userspace: bool,
    pub cpu_id: usize,
}

pub struct Cpu<'a>(&'a mut native::Cpu<'a>);
impl<'a> Cpu<'a> {
    pub fn new(
        page_table: &'a mut PageTable,
        interrupt_table: &'a mut InterruptTable,
        cpu_info: CpuInfo,
    ) -> Self {
        Cpu(native::Cpu::new(page_table, interrupt_table, cpu_info))
    }
    pub fn info(&self) -> &CpuInfo {
        self.0.info()
    }
    pub fn page_table(&mut self) -> &mut PageTable {
        self.0.page_table()
    }
    pub fn interrupt_table(&self) -> &InterruptTable {
        self.0.interrupt_table()
    }
    pub fn set_page_table(&mut self, page_table: &'a mut PageTable) {
        self.0.set_page_table(page_table)
    }
    pub fn set_interrupt_table(&mut self, interrupt_table: &'a mut InterruptTable) {
        self.0.set_interrupt_table(interrupt_table)
    }
    pub fn set_as_current_cpu(&self) {
        self.0.set_as_current_cpu()
    }
    pub fn get_current_cpu() -> Option<&'a mut Cpu<'a>> {
        native::Cpu::get_current_cpu()
    }
}
