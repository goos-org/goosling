#![feature(naked_functions, asm_sym, alloc_error_handler, abi_x86_interrupt)]
#![no_std]
#![no_main]

extern crate alloc;

pub mod arch;
pub mod exceptions;
pub mod memory;
pub mod task;
pub mod terminals;

use crate::arch::{CpuInterrupt, Error, InterruptTable, Util};
use crate::memory::BitmapAllocator;
use crate::terminals::Terminal;
use alloc::string::ToString;
use limine::{LimineMemoryMapEntryType, LimineMmapRequest, LimineTerminal, LimineTerminalRequest};
use numtoa::NumToA;

static MMAP_REQUEST: LimineMmapRequest = LimineMmapRequest::new(0);
static TERMINAL_REQUEST: LimineTerminalRequest = LimineTerminalRequest::new(0);

fn pretty_print_size<TermWrite: Fn(&LimineTerminal, &str)>(
    size: usize,
    terminal: &Terminal<TermWrite>,
) {
    let mut bytes = [0u8; 20];
    if size < 1024 {
        terminal.print(size.numtoa_str(10, &mut bytes));
        terminal.print("B total");
    } else if size < 1024 * 1024 {
        terminal.print((size / 1024).numtoa_str(10, &mut bytes));
        terminal.print("KiB, ");
        terminal.print((size % 1024).numtoa_str(10, &mut bytes));
        terminal.print("B");
    } else if size < 1024 * 1024 * 1024 {
        terminal.print((size / 1024 / 1024).numtoa_str(10, &mut bytes));
        terminal.print("MiB, ");
        terminal.print(((size % (1024 * 1024)) / 1024).numtoa_str(10, &mut bytes));
        terminal.print("KiB");
    } else {
        terminal.print((size / 1024 / 1024 / 1024).numtoa_str(10, &mut bytes));
        terminal.print("GiB, ");
        terminal.print(((size % (1024 * 1024 * 1024)) / 1024 / 1024).numtoa_str(10, &mut bytes));
        terminal.print("MiB");
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let term = {
        let response = TERMINAL_REQUEST
            .get_response()
            .get()
            .unwrap_or_else(|| Util::halt_loop());
        let terminals = response.terminals().unwrap_or_else(|| Util::halt_loop());
        let term = terminals.get(0).unwrap_or_else(|| Util::halt_loop());
        Terminal::new(term, response.write().unwrap_or_else(|| Util::halt_loop()))
    };
    term.fail("Rust panicked");
    term.fail(info.payload().downcast_ref::<&str>().unwrap_or_else(|| {
        term.info(info.to_string().as_str());
        Util::halt_loop();
    }));
    Util::halt_loop();
}

#[no_mangle]
extern "C" fn main() -> ! {
    let terminal = TERMINAL_REQUEST.get_response().get().unwrap();
    let terminals = terminal.terminals().unwrap();
    let term_write = terminal.write().unwrap();
    let terminal = Terminal::new(&terminals[0], term_write);
    terminal.info("Hello, world!");
    terminal.ok("Booted from Limine");
    terminal.ok("Initialized terminal");
    terminal.info("Reading memory map");
    let mmap = MMAP_REQUEST.get_response().get().unwrap_or_else(|| {
        terminal.fail("Failed to read memory map");
        Util::halt_loop();
    });
    let mmap = mmap.mmap().unwrap_or_else(|| {
        terminal.fail("Failed to read memory map");
        Util::halt_loop();
    });
    terminal.ok("Read memory map");
    terminal.info("Reading available memory");
    let highest_mmap = mmap.last().unwrap_or_else(|| {
        terminal.fail("Failed to read memory map");
        Util::halt_loop();
    });
    let highest_mmap = highest_mmap.base + highest_mmap.len;
    terminal.ok("Read available memory");

    terminal.info("Allocating memory");
    let allocator_size = highest_mmap / 4096 / 8;
    let mut allocator_location = 0;
    for entry in mmap {
        if entry.typ == LimineMemoryMapEntryType::Usable && entry.len >= allocator_size {
            allocator_location = entry.base;
            break;
        }
    }
    terminal.ok("Allocated memory");

    terminal.info("Setting up bitmap allocator");
    let mut allocator =
        BitmapAllocator::new(allocator_location as *mut u8, allocator_size as usize);
    for entry in mmap {
        if entry.typ == LimineMemoryMapEntryType::Usable {
            allocator.free_range(entry.base as usize, (entry.base + entry.len) as usize);
        }
    }

    unsafe { memory::ALLOCATOR = Some(allocator) };
    let allocator = unsafe { memory::ALLOCATOR.as_mut().unwrap() };

    terminal.ok("Set up bitmap allocator");
    terminal.info_raw("Free memory: ");
    pretty_print_size(allocator.get_free() as usize, &terminal);
    terminal.println("");

    terminal.info("Initializing cpu");
    Util::init().unwrap_or_else(|e| {
        match e {
            Error::Unsupported => {
                terminal.fail("Failed to initialize cpu: unsupported cpu");
            }
            _ => {
                terminal.fail("Failed to initialize cpu: unknown error");
            }
        }
        Util::halt_loop();
    });
    terminal.ok("Initialized cpu");

    let mut idt = InterruptTable::new();
    exceptions::set_handlers(&mut idt);

    let value: usize = unsafe { *(core::ptr::null()) };

    Util::halt_loop();
}
