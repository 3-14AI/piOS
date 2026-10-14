#![cfg_attr(not(test), no_std)]

pub mod libc;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
