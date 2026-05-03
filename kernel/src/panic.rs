#[allow(unused_imports)]
use core::panic::PanicInfo;

#[cfg(not(any(test, target_os = "linux", target_os = "windows", target_os = "macos")))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
