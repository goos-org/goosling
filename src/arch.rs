pub mod traits;
pub mod x86_64;

use crate::arch::native::ErrorCode;
//#[cfg(target_arch = "x86_64")]
pub use x86_64 as native;

type InterruptHandler = fn(Option<ErrorCode>, usize, &'static mut usize);

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
