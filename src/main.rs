#![no_std]
#![no_main]

use core::panic::PanicInfo;
use limine::LimineFramebufferRequest;
use rand::prelude::SmallRng;
use rand::{Rng, SeedableRng};

mod memory;

static FRAMEBUFFER_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest::new(0);

#[panic_handler]
pub fn panic(_info: &PanicInfo) -> ! {
    loop {}
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
    let mut rng = SmallRng::from_seed([0; 32]);
    loop {
        for i in 0..framebuffer.pitch * framebuffer.height {
            framebuffer_ptr.add(i as usize).write_volatile(rng.gen());
        }
    }
}
