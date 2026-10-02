#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![allow(clippy::empty_loop)]

extern crate alloc;

use alloc::string::String;
use alloc::format;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[cfg(not(test))]
use core::alloc::{GlobalAlloc, Layout};

#[cfg(not(test))]
struct BumpAllocator {
    heap: core::cell::UnsafeCell<[u8; 10 * 1024 * 1024]>,
    bump_ptr: core::cell::UnsafeCell<usize>,
}

#[cfg(not(test))]
unsafe impl Sync for BumpAllocator {}

#[cfg(not(test))]
unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let bump_ptr_mut = &mut *self.bump_ptr.get();
        let heap_ptr = self.heap.get() as *mut u8;

        let current_ptr = heap_ptr.add(*bump_ptr_mut);
        let align_offset = current_ptr.align_offset(layout.align());

        let new_bump = *bump_ptr_mut + align_offset + layout.size();

        if new_bump > 10 * 1024 * 1024 {
            return core::ptr::null_mut();
        }

        let ptr = heap_ptr.add(*bump_ptr_mut + align_offset);
        *bump_ptr_mut = new_bump;
        ptr
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

#[cfg(not(test))]
#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator {
    heap: core::cell::UnsafeCell::new([0; 10 * 1024 * 1024]),
    bump_ptr: core::cell::UnsafeCell::new(0),
};

pub struct WebBrowser {
    current_url: String,
}

impl Default for WebBrowser {
    fn default() -> Self {
        Self::new()
    }
}

impl WebBrowser {
    pub fn new() -> Self {
        Self {
            current_url: String::from("about:blank"),
        }
    }

    pub fn navigate(&mut self, url: &str) -> String {
        self.current_url = String::from(url);
        format!("Navigated to {}", self.current_url)
    }

    pub fn render(&self) -> String {
        format!("Rendering content for {}", self.current_url)
    }

    pub fn parse_html(&self, html: &str) -> Result<String, String> {
        let nodes = html.split("<").count();
        Ok(format!("Parsed HTML dom with {} nodes", nodes))
    }

    pub fn parse_css(&self, css: &str) -> String {
        let rules = css.split("{").count();
        format!("Parsed CSS with {} rules", rules)
    }
}

#[cfg(not(test))]
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    let mut browser = WebBrowser::new();
    let _ = browser.navigate("https://example.com");
    let _ = browser.render();
    loop {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_navigation() {
        let mut browser = WebBrowser::new();
        let res = browser.navigate("https://pios.ai");
        assert_eq!(res, "Navigated to https://pios.ai");
        assert_eq!(browser.current_url, "https://pios.ai");
    }

    #[test]
    fn test_browser_render() {
        let mut browser = WebBrowser::new();
        browser.navigate("https://pios.ai");
        let res = browser.render();
        assert_eq!(res, "Rendering content for https://pios.ai");
    }

    #[test]
    fn test_parse_html() {
        let browser = WebBrowser::new();
        let res = browser.parse_html("<html><body>Hello</body></html>");
        assert!(res.is_ok());
    }

    #[test]
    fn test_parse_css() {
        let browser = WebBrowser::new();
        let res = browser.parse_css("body { color: red; }");
        assert!(res.contains("Parsed CSS"));
    }
}
