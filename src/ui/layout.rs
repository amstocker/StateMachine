use super::Transform;


#[derive(Debug)]
pub struct ThreePanelLayoutTransform {
    pub main_panel_transform: Transform,
    pub side_panel_transform: Transform,
    pub bottom_panel_transform: Transform
}

pub struct ThreePanelLayout {
    pub vertical_divide: f32,
    pub horizontal_divide: f32
}

impl ThreePanelLayout {
    pub fn transform(&self) -> ThreePanelLayoutTransform {
        let vertical = Vertical {
            divide: self.vertical_divide
        }.transform();
        let horizontal = Horizontal {
            divide: self.horizontal_divide
        }.transform();
        ThreePanelLayoutTransform { 
            main_panel_transform: vertical.top.then(horizontal.right),
            side_panel_transform: vertical.top.then(horizontal.left),
            bottom_panel_transform: vertical.bottom
        }
    }
}

#[derive(Debug)]
pub struct HorizontalTransform {
    left: Transform,
    right: Transform
}

pub struct Horizontal {
    pub divide: f32
}

impl Horizontal {
    pub fn transform(&self) -> HorizontalTransform {
        let d = self.divide;
        HorizontalTransform {
            left: Transform {
                translate: (0.0, 0.0),
                scale: (d, 1.0)
            },
            right: Transform {
                translate: (d, 0.0),
                scale: (1.0 - d, 1.0)
            }
        }
    }
}

#[derive(Debug)]
pub struct VerticalTransform {
    top: Transform,
    bottom: Transform
}

pub struct Vertical {
    pub divide: f32
}

impl Vertical {
    pub fn transform(&self) -> VerticalTransform {
        let d = self.divide;
        VerticalTransform {
            top: Transform {
                translate: (0.0, d),
                scale: (1.0, 1.0 - d)
            },
            bottom: Transform {
                translate: (0.0, 0.0),
                scale: (1.0, d)
            }
        }
    }
}