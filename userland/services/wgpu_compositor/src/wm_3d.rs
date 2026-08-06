#![no_std]
#![allow(unused)]

extern crate alloc;
use alloc::vec::Vec;

pub struct Window3D {
    pub id: u32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub width: f32,
    pub height: f32,
}

impl Window3D {
    pub fn new(id: u32, x: f32, y: f32, z: f32, width: f32, height: f32) -> Self {
        Self {
            id,
            x,
            y,
            z,
            width,
            height,
        }
    }
}

pub struct Wm3D {
    windows: Vec<Window3D>,
}

impl Default for Wm3D {
    fn default() -> Self {
        Self::new()
    }
}

impl Wm3D {
    pub fn new() -> Self {
        Self {
            windows: Vec::new(),
        }
    }

    pub fn add_window(&mut self, window: Window3D) {
        self.windows.push(window);
    }

    pub fn layout_windows(&mut self) {
        // Mock 3D layout logic
        let mut current_z = 0.0;
        for window in &mut self.windows {
            window.z = current_z;
            current_z += 1.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wm_3d() {
        let mut wm = Wm3D::new();
        wm.add_window(Window3D::new(1, 0.0, 0.0, 0.0, 100.0, 100.0));
        wm.add_window(Window3D::new(2, 50.0, 50.0, 0.0, 100.0, 100.0));
        wm.layout_windows();

        assert_eq!(wm.windows[0].z, 0.0);
        assert_eq!(wm.windows[1].z, 1.0);
    }
}
