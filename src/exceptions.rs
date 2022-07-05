use crate::arch::{CpuInterrupt, CpuState, ErrorCode, InterruptTable, Util};

pub fn set_handlers(idt: &mut InterruptTable) {
    match idt.set_interrupt_handler(CpuInterrupt::DivideByZero, divide_by_zero) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::Debug, debug) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::NonMaskableInterrupt, nmi) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::Breakpoint, breakpoint) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::Overflow, overflow) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::BoundRangeExceeded, bound_range_exceeded) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::InvalidOpcode, invalid_opcode) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::DeviceUnavailable, device_unavailable) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::InvalidTss, invalid_tss) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::SegmentNotPresent, segment_not_present) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::StackSegmentFault, stack_segment_fault) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(
        CpuInterrupt::GeneralProtectionFault,
        general_protection_fault,
    ) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::PageFault, page_fault) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::FloatingPointException, fpu_exception) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::AlignmentCheck, alignment_check) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::MachineCheck, machine_check) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::SimdException, simd_exception) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(
        CpuInterrupt::VirtualizationException,
        virtualization_exception,
    ) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(
        CpuInterrupt::ControlProtectionException,
        control_protection_exception,
    ) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(
        CpuInterrupt::HypervisorInjectionException,
        hypervisor_injection_exception,
    ) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(
        CpuInterrupt::VmmCommunicationException,
        vmm_communication_exception,
    ) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
    match idt.set_interrupt_handler(CpuInterrupt::SecurityException, security_exception) {
        Ok(_) => (),
        Err(e) => panic!("{:?}", e),
    };
}

pub fn divide_by_zero(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Division by zero (#DE) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn debug(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Debug exception (#DB) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn nmi(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Non-maskable interrupt at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn breakpoint(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Breakpoint (#BP) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn overflow(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Overflow exception (#OF) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn bound_range_exceeded(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Bound range exceeded exception (#BR) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn invalid_opcode(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Invalid opcode exception (#UD) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn device_unavailable(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Device unavailable exception (#NM) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn invalid_tss(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Invalid TSS exception (#TS) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn segment_not_present(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Segment not present exception (#NP) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn stack_segment_fault(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Stack segment fault (#SS) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn general_protection_fault(
    error_code: Option<ErrorCode>,
    _: CpuInterrupt,
    state: &mut CpuState,
) {
    panic!(
        "General protection fault (#GP) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn page_fault(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Page fault (#PF) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn fpu_exception(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Floating point exception (#MF) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn alignment_check(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Alignment check (#AC) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn machine_check(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Machine check (#MC) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn simd_exception(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "SIMD exception (#XM/#XF) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn virtualization_exception(
    error_code: Option<ErrorCode>,
    _: CpuInterrupt,
    state: &mut CpuState,
) {
    panic!(
        "Virtualization exception (#VE) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn control_protection_exception(
    error_code: Option<ErrorCode>,
    _: CpuInterrupt,
    state: &mut CpuState,
) {
    panic!(
        "Control protection exception (#CP) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn hypervisor_injection_exception(
    error_code: Option<ErrorCode>,
    _: CpuInterrupt,
    state: &mut CpuState,
) {
    panic!(
        "Hypervisor injection exception (#HV) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn vmm_communication_exception(
    error_code: Option<ErrorCode>,
    _: CpuInterrupt,
    state: &mut CpuState,
) {
    panic!(
        "VMM communication exception (#VC) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}

pub fn security_exception(error_code: Option<ErrorCode>, _: CpuInterrupt, state: &mut CpuState) {
    panic!(
        "Security exception (#SX) at {:x}\nError code: {:?}",
        state.get_ip(),
        error_code
    )
}
