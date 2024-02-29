
#[derive(Clone, Copy, Debug)]
pub struct TouchPosMapper {
    width: u16,
    height: u16,
    touch_limit_x: (u16, u16),
    touch_limit_y: (u16, u16),
    scale: (f32, f32),
    offset: (f32, f32),
}

impl TouchPosMapper {
    pub fn new(width: u16, height: u16, touch_limit_x: (u16, u16), touch_limit_y: (u16, u16)) -> Self {
        let scale_x = width as f32 / (touch_limit_x.1 - touch_limit_x.0) as f32;
        let offset_x = -(touch_limit_x.0 as f32) * width as f32 / (touch_limit_x.1 - touch_limit_x.0) as f32;

        let scale_y = height as f32 / (touch_limit_y.1 - touch_limit_y.0) as f32;
        let offset_y = -(touch_limit_y.0 as f32) * height as f32 / (touch_limit_y.1 - touch_limit_y.0) as f32;

        TouchPosMapper {
            width,
            height,
            touch_limit_x,
            touch_limit_y,
            scale: (scale_x, scale_y),
            offset: (offset_x, offset_y),
        }
    }

    pub fn map_touch_pos(self, x: u16, y: u16, width: u16, height: u16, orientation: u8) -> (u16, u16) {
        let x_scaled = ((x as f32 * self.scale.0 + self.offset.0) as u16).min(self.width);
        let y_scaled = ((y as f32 * self.scale.1 + self.offset.1) as u16).min(self.height);

        match orientation {
            0 => (x_scaled, y_scaled),
            1 => (y_scaled, height - x_scaled),
            2 => (width - x_scaled, height - y_scaled),
            3 => (width - y_scaled, x_scaled),
            _ => (x, y)
        }
    }
}