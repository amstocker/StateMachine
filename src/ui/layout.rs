use super::Transform;


pub struct GlobalLayoutTransforms {
    pub main_panel_transform: Transform,
    pub side_panel_transform: Transform,
    pub bottom_panel_transform: Transform
}

pub struct GlobalLayout {
    pub vertical_divide: f32,
    pub horizontal_divide: f32
}

impl GlobalLayout {
    pub fn build_transforms(&self) -> GlobalLayoutTransforms {
        let v = self.vertical_divide;
        let h = self.horizontal_divide;
        GlobalLayoutTransforms { 
            main_panel_transform: Transform {
                translate: (h, v),
                scale: (1.0 - h, 1.0 - v)
            },
            side_panel_transform: Transform {
                translate: (0.0, v),
                scale: (h, 1.0 - v)
            },
            bottom_panel_transform: Transform {
                translate: (0.0, 0.0),
                scale: (1.0, v)
            }
        }
    }
}
