#[allow(unused_imports)]
use core::panic::PanicInfo;

#[cfg(all(not(test), any(target_os = "none", target_os = "uefi")))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
