#[derive(Debug, Clone, Copy, PartialEq)]
pub struct XYZ {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl XYZ {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Orient {
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
}

impl Orient {
    pub fn new(pitch: f32, roll: f32, yaw: f32) -> Self {
        Self { pitch, roll, yaw }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}
