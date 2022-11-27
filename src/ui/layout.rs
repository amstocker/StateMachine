use crate::ui::{Transform, UITransform};


pub enum RelativePosition {
    Above,
    Below,
    Left,
    Right
}

#[derive(Debug)]
pub struct ThreePanelLayoutTransform {
    pub main_panel_transform: UITransform,
    pub side_panel_transform: UITransform,
    pub bottom_panel_transform: UITransform
}

pub struct ThreePanelLayout {
    pub vertical_divide: f32,
    pub horizontal_divide: f32
}

impl ThreePanelLayout {
    pub fn transform(&self) -> ThreePanelLayoutTransform {
        let vertical = VerticalTransform::new(self.vertical_divide);
        let horizontal = HorizontalTransform::new(self.horizontal_divide);
        ThreePanelLayoutTransform { 
            main_panel_transform: vertical.top.then(horizontal.right),
            side_panel_transform: vertical.top.then(horizontal.left),
            bottom_panel_transform: vertical.bottom
        }
    }
}

#[derive(Debug)]
pub struct HorizontalTransform {
    left: UITransform,
    right: UITransform
}

impl HorizontalTransform {
    pub fn new(divide: f32) -> Self {
        Self {
            left: UITransform {
                translate: (0.0, 0.0),
                scale: (divide, 1.0)
            },
            right: UITransform {
                translate: (divide, 0.0),
                scale: (1.0 - divide, 1.0)
            }
        }
    }
}

pub struct Horizontal {
    pub divide: f32
}

impl Horizontal {
    pub fn transform(&self) -> HorizontalTransform {
        HorizontalTransform::new(self.divide)
    }
}

#[derive(Debug)]
pub struct VerticalTransform {
    top: UITransform,
    bottom: UITransform
}

impl VerticalTransform {
    pub fn new(divide: f32) -> Self {
        Self {
            top: UITransform {
                translate: (0.0, divide),
                scale: (1.0, 1.0 - divide)
            },
            bottom: UITransform {
                translate: (0.0, 0.0),
                scale: (1.0, divide)
            }
        }
    }
}

pub struct Vertical {
    pub divide: f32
}

impl Vertical {
    pub fn transform(&self) -> VerticalTransform {
        VerticalTransform::new(self.divide)
    }
}