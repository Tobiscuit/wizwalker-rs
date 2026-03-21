use crate::errors::Result;

/// RGBA color as used by the game engine.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// 3D position vector.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct XYZ {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

/// Orientation (pitch, roll, yaw) in radians.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Orient {
    pub pitch: f32,
    pub roll: f32,
    pub yaw: f32,
}

pub struct Point(pub i32, pub i32);

pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn center(&self) -> (i32, i32) {
        (
            self.left + (self.right - self.left) / 2,
            self.top + (self.bottom - self.top) / 2,
        )
    }
}

pub struct DynamicWindow;
impl DynamicWindow {
    pub async fn scale_to_client(&self) -> Result<Rect> {
        Ok(Rect { left: 0, top: 0, right: 100, bottom: 100 })
    }
}

