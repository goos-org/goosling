#![no_std]
#![no_main]

use limine::LimineMmapRequest;

#[used]
static MMAP: LimineMmapRequest = LimineMmapRequest::new(0);

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
extern "C" fn main() -> ! {
    unsafe {
        core::ptr::read_volatile(&MMAP);
    }
    panic!("Hello, world!");
}
