use crate::arch::native::ErrorCode;
use crate::{CpuInterrupt, InterruptTable, InterruptTableTrait, Util};

pub fn set_handlers(idt: &mut InterruptTable) {
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::DivideByZero) {
        idt.set_interrupt_handler(int_num, divide_by_zero);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::Debug) {
        idt.set_interrupt_handler(int_num, debug);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::NonMaskableInterrupt) {
        idt.set_interrupt_handler(int_num, nmi);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::Breakpoint) {
        idt.set_interrupt_handler(int_num, breakpoint);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::Overflow) {
        idt.set_interrupt_handler(int_num, overflow);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::BoundRangeExceeded) {
        idt.set_interrupt_handler(int_num, bound_range_exceeded);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::InvalidOpcode) {
        idt.set_interrupt_handler(int_num, invalid_opcode);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::DeviceUnavailable) {
        idt.set_interrupt_handler(int_num, device_unavailable);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::InvalidTss) {
        idt.set_interrupt_handler(int_num, invalid_tss);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::SegmentNotPresent) {
        idt.set_interrupt_handler(int_num, segment_not_present);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::StackSegmentFault) {
        idt.set_interrupt_handler(int_num, stack_segment_fault);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::GeneralProtectionFault) {
        idt.set_interrupt_handler(int_num, general_protection_fault);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::PageFault) {
        idt.set_interrupt_handler(int_num, page_fault);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::FloatingPointException) {
        idt.set_interrupt_handler(int_num, fpu_exception);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::AlignmentCheck) {
        idt.set_interrupt_handler(int_num, alignment_check);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::MachineCheck) {
        idt.set_interrupt_handler(int_num, machine_check);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::SimdException) {
        idt.set_interrupt_handler(int_num, simd_exception);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::VirtualizationException) {
        idt.set_interrupt_handler(int_num, virtualization_exception);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::ControlProtectionException) {
        idt.set_interrupt_handler(int_num, control_protection_exception);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::HypervisorInjectionException) {
        idt.set_interrupt_handler(int_num, hypervisor_injection_exception);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::VmmCommunicationException) {
        idt.set_interrupt_handler(int_num, vmm_communication_exception);
    }
    if let Some(int_num) = Util::interrupt_num(CpuInterrupt::SecurityException) {
        idt.set_interrupt_handler(int_num, security_exception);
    }
}

pub fn divide_by_zero(error_code: Option<ErrorCode>, _: usize, instruction_pointer: &mut usize) {
    panic!(
        "Division by zero (#DE) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn debug(error_code: Option<ErrorCode>, _: usize, instruction_pointer: &mut usize) {
    panic!(
        "Debug exception (#DB) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn nmi(error_code: Option<ErrorCode>, _: usize, instruction_pointer: &mut usize) {
    panic!(
        "Non-maskable interrupt at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn breakpoint(error_code: Option<ErrorCode>, _: usize, instruction_pointer: &mut usize) {
    panic!(
        "Breakpoint (#BP) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn overflow(error_code: Option<ErrorCode>, _: usize, instruction_pointer: &mut usize) {
    panic!(
        "Overflow exception (#OF) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn bound_range_exceeded(
    error_code: Option<ErrorCode>,
    _: usize,
    instruction_pointer: &mut usize,
) {
    panic!(
        "Bound range exceeded exception (#BR) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn invalid_opcode(error_code: Option<ErrorCode>, _: usize, instruction_pointer: &mut usize) {
    panic!(
        "Invalid opcode exception (#UD) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn device_unavailable(
    error_code: Option<ErrorCode>,
    _: usize,
    instruction_pointer: &mut usize,
) {
    panic!(
        "Device unavailable exception (#NM) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn invalid_tss(error_code: Option<ErrorCode>, _: usize, instruction_pointer: &mut usize) {
    panic!(
        "Invalid TSS exception (#TS) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn segment_not_present(
    error_code: Option<ErrorCode>,
    _: usize,
    instruction_pointer: &mut usize,
) {
    panic!(
        "Segment not present exception (#NP) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn stack_segment_fault(
    error_code: Option<ErrorCode>,
    _: usize,
    instruction_pointer: &mut usize,
) {
    panic!(
        "Stack segment fault (#SS) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn general_protection_fault(
    error_code: Option<ErrorCode>,
    _: usize,
    instruction_pointer: &mut usize,
) {
    panic!(
        "General protection fault (#GP) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn page_fault(error_code: Option<ErrorCode>, _: usize, instruction_pointer: &mut usize) {
    panic!(
        "Page fault (#PF) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn fpu_exception(error_code: Option<ErrorCode>, _: usize, instruction_pointer: &mut usize) {
    panic!(
        "Floating point exception (#MF) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn alignment_check(error_code: Option<ErrorCode>, _: usize, instruction_pointer: &mut usize) {
    panic!(
        "Alignment check (#AC) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn machine_check(error_code: Option<ErrorCode>, _: usize, instruction_pointer: &mut usize) {
    panic!(
        "Machine check (#MC) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn simd_exception(error_code: Option<ErrorCode>, _: usize, instruction_pointer: &mut usize) {
    panic!(
        "SIMD exception (#XM/#XF) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn virtualization_exception(
    error_code: Option<ErrorCode>,
    _: usize,
    instruction_pointer: &mut usize,
) {
    panic!(
        "Virtualization exception (#VE) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn control_protection_exception(
    error_code: Option<ErrorCode>,
    _: usize,
    instruction_pointer: &mut usize,
) {
    panic!(
        "Control protection exception (#CP) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn hypervisor_injection_exception(
    error_code: Option<ErrorCode>,
    _: usize,
    instruction_pointer: &mut usize,
) {
    panic!(
        "Hypervisor injection exception (#HV) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn vmm_communication_exception(
    error_code: Option<ErrorCode>,
    _: usize,
    instruction_pointer: &mut usize,
) {
    panic!(
        "VMM communication exception (#VC) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}

pub fn security_exception(
    error_code: Option<ErrorCode>,
    _: usize,
    instruction_pointer: &mut usize,
) {
    panic!(
        "Security exception (#SX) at {:x}\nError code: {:?}",
        instruction_pointer, error_code
    )
}
