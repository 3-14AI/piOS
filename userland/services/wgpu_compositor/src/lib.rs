// Window manager and compositor using wgpu


pub struct Window {
    pub id: u32,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

pub struct Compositor {
    pub instance: wgpu::Instance,
    pub windows: Vec<Window>,
    next_window_id: u32,
}

impl Compositor {
    pub fn new() -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        Self {
            instance,
            windows: Vec::new(),
            next_window_id: 1,
        }
    }

    pub fn create_window(&mut self, x: u32, y: u32, width: u32, height: u32) -> u32 {
        let id = self.next_window_id;
        self.next_window_id += 1;
        self.windows.push(Window {
            id,
            x,
            y,
            width,
            height,
        });
        id
    }

    pub fn destroy_window(&mut self, id: u32) -> bool {
        if let Some(pos) = self.windows.iter().position(|w| w.id == id) {
            self.windows.remove(pos);
            true
        } else {
            false
        }
    }

    pub fn get_window(&self, id: u32) -> Option<&Window> {
        self.windows.iter().find(|w| w.id == id)
    }

    pub fn get_window_mut(&mut self, id: u32) -> Option<&mut Window> {
        self.windows.iter_mut().find(|w| w.id == id)
    }
}

impl Default for Compositor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compositor_creation() {
        let compositor = Compositor::new();
        assert_eq!(compositor.windows.len(), 0);
    }

    #[test]
    fn test_window_management() {
        let mut compositor = Compositor::new();

        // Create a window
        let id = compositor.create_window(10, 20, 800, 600);
        assert_eq!(compositor.windows.len(), 1);

        // Check window properties
        let window = compositor.get_window(id).unwrap();
        assert_eq!(window.x, 10);
        assert_eq!(window.y, 20);
        assert_eq!(window.width, 800);
        assert_eq!(window.height, 600);

        // Modify window
        if let Some(win) = compositor.get_window_mut(id) {
            win.x = 30;
        }
        let window = compositor.get_window(id).unwrap();
        assert_eq!(window.x, 30);

        // Destroy window
        assert!(compositor.destroy_window(id));
        assert_eq!(compositor.windows.len(), 0);

        // Destroy non-existent window
        assert!(!compositor.destroy_window(999));
    }
}
