use crate::errors::Result;

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
