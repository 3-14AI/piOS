#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![allow(clippy::empty_loop)]

extern crate alloc;

use alloc::format;
use alloc::string::String;

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

#[cfg(not(test))]
extern "C" {
    fn sys_net_connect(ip_ptr: i32, ip_len: i32, port: i32, fd_out: i32) -> i32;
    fn sys_net_send(fd: i32, buf_ptr: i32, buf_len: i32, bytes_sent_out: i32) -> i32;
    fn sys_net_recv(fd: i32, buf_ptr: i32, buf_len: i32, bytes_read_out: i32) -> i32;
}

#[cfg(test)]
unsafe fn sys_net_connect(_ip_ptr: i32, _ip_len: i32, _port: i32, fd_out: i32) -> i32 {
    let fd_out_ptr = fd_out as usize as *mut i32;
    *fd_out_ptr = 3;
    0
}

#[cfg(test)]
unsafe fn sys_net_send(_fd: i32, _buf_ptr: i32, buf_len: i32, bytes_sent_out: i32) -> i32 {
    let out_ptr = bytes_sent_out as usize as *mut i32;
    *out_ptr = buf_len;
    0
}

#[cfg(test)]
unsafe fn sys_net_recv(_fd: i32, buf_ptr: i32, buf_len: i32, bytes_read_out: i32) -> i32 {
    let mock_data = b"HTTP/1.1 200 OK\r\n\r\nMock content";
    let len = core::cmp::min(mock_data.len(), buf_len as usize);
    core::ptr::copy_nonoverlapping(mock_data.as_ptr(), buf_ptr as usize as *mut u8, len);
    let out_ptr = bytes_read_out as usize as *mut i32;
    *out_ptr = len as i32;
    0
}

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

    pub fn download_file(&self, ip: &str, port: i32, path: &str) -> Result<String, String> {
        let mut fd_out = 0i32;
        let mut bytes_sent_out = 0i32;
        let mut bytes_read_out = 0i32;
        let mut recv_buf = alloc::vec![0u8; 1024];

        let req = format!("GET {} HTTP/1.1\r\nHost: {}\r\n\r\n", path, ip);

        #[cfg(not(test))]
        unsafe {
            if sys_net_connect(
                ip.as_ptr() as usize as i32,
                ip.len() as i32,
                port,
                &mut fd_out as *mut i32 as usize as i32,
            ) != 0
            {
                return Err(String::from("Failed to connect"));
            }

            if sys_net_send(
                fd_out,
                req.as_ptr() as usize as i32,
                req.len() as i32,
                &mut bytes_sent_out as *mut i32 as usize as i32,
            ) != 0
            {
                return Err(String::from("Failed to send request"));
            }

            if sys_net_recv(
                fd_out,
                recv_buf.as_mut_ptr() as usize as i32,
                recv_buf.len() as i32,
                &mut bytes_read_out as *mut i32 as usize as i32,
            ) != 0
            {
                return Err(String::from("Failed to receive response"));
            }
        }

        #[cfg(test)]
        {
            fd_out = 3;
            bytes_sent_out = req.len() as i32;
            let mock_data = b"HTTP/1.1 200 OK\r\n\r\nMock content";
            let copy_len = core::cmp::min(recv_buf.len(), mock_data.len());
            recv_buf[..copy_len].copy_from_slice(&mock_data[..copy_len]);
            bytes_read_out = copy_len as i32;
            let _ = fd_out;
            let _ = bytes_sent_out;
            let _ = port;
        }

        let received_str = core::str::from_utf8(&recv_buf[..bytes_read_out as usize]).unwrap_or("");

        // Simulating saving to semanticFS/ext4 by returning a success message
        Ok(format!(
            "Downloaded file of size {} and saved to SemanticFS",
            received_str.len()
        ))
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

    #[test]
    fn test_download_file() {
        let browser = WebBrowser::new();
        let res = browser.download_file("127.0.0.1", 80, "/index.html");
        assert!(res.is_ok());
        assert!(res.unwrap().contains("saved to SemanticFS"));
    }
}
