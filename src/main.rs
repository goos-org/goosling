#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

extern crate alloc;

use crate::debug::SerialWriter;
use crate::memory::{Heap, HeapGlobalAlloc};
use crate::sync::Mutex;
use alloc::boxed::Box;
use core::arch::asm;
use core::cell::UnsafeCell;
use core::fmt::Write;
use core::panic::PanicInfo;
use limine::{LimineFramebufferRequest, LimineHhdmRequest, LimineMmapRequest};
use x86_64::structures::paging::{OffsetPageTable, PageTable};
use x86_64::VirtAddr;

mod debug;
mod graphics;
mod memory;
mod sync;

static FRAMEBUFFER_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest::new(0);
static HHDM_REQUEST: LimineHhdmRequest = LimineHhdmRequest::new(0);
static MMAP_REQUEST: LimineMmapRequest = LimineMmapRequest::new(0);
static mut SERIAL: SerialWriter = SerialWriter::new();

#[panic_handler]
pub fn panic(info: &PanicInfo) -> ! {
    unsafe {
        SERIAL
            .write_str("Panic: ")
            .expect("Failed to write to serial");
        SERIAL
            .write_fmt(*info.message().unwrap())
            .expect("Failed to write to serial");
    }
    loop {
        x86_64::instructions::hlt();
    }
}

#[no_mangle]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "C" fn main() {
    let framebuffer = FRAMEBUFFER_REQUEST
        .get_response()
        .get()
        .unwrap()
        .framebuffers
        .get()
        .unwrap()
        .get()
        .unwrap();
    let framebuffer_ptr = framebuffer.address.as_ptr().unwrap() as *mut u8;
    let page_table_addr: usize;
    asm!("mov {}, cr3", out(reg) page_table_addr);
    let mut offset_page_table = OffsetPageTable::new(
        core::mem::transmute(page_table_addr),
        VirtAddr::from_ptr(HHDM_REQUEST.get_response().get().unwrap().offset as *const ()),
    );
    memory::GLOBAL_ALLOCATOR = HeapGlobalAlloc {
        val: Some(Mutex::new(UnsafeCell::new(Heap::new(
            MMAP_REQUEST.get_response().get().unwrap().mmap().unwrap(),
            &mut *(&mut offset_page_table as *mut _),
        )))),
    };
    framebuffer_ptr.write_bytes(0x23, (framebuffer.pitch * framebuffer.height) as usize);
    panic!("Done");
}
