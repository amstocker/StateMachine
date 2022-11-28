use crate::ui::{Transform, Position};

use super::ApplyTransform;


#[derive(Debug)]
pub enum HorizontalPosition {
    Left,
    Right
}

#[derive(Debug)]
pub enum VerticalPosition {
    Top,
    Bottom
}

#[derive(Debug)]
pub enum ThreePanelPosition {
    Main,
    Side,
    Bottom
}

#[derive(Debug)]
pub struct ThreePanelLayout {
    pub vertical: VerticalLayout,
    pub horizontal: HorizontalLayout,
    main_panel_transform: Transform,
    side_panel_transform: Transform,
    bottom_panel_transform: Transform
}

impl ThreePanelLayout {
    pub fn new(vertical_divide: f32, horizontal_divide: f32) -> ThreePanelLayout {
        let vertical = VerticalLayout::new(vertical_divide);
        let horizontal = HorizontalLayout::new(horizontal_divide);
        ThreePanelLayout { 
            vertical,
            horizontal,
            main_panel_transform: vertical.top.then(horizontal.right),
            side_panel_transform: vertical.top.then(horizontal.left),
            bottom_panel_transform: vertical.bottom
        }
    }

    pub fn select(&self, position: Position) -> ThreePanelPosition {
        match self.vertical.select(position) {
            VerticalPosition::Top => match self.horizontal.select(position.apply(self.vertical.top)) {
                HorizontalPosition::Left => ThreePanelPosition::Side,
                HorizontalPosition::Right => ThreePanelPosition::Main,
            },
            VerticalPosition::Bottom => ThreePanelPosition::Bottom,
        }
    }

    pub fn get(&self, panel_position: ThreePanelPosition) -> Transform {
        match panel_position {
            ThreePanelPosition::Main => self.main_panel_transform,
            ThreePanelPosition::Side => self.side_panel_transform,
            ThreePanelPosition::Bottom => self.bottom_panel_transform,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct HorizontalLayout {
    pub divide: f32,
    left: Transform,
    right: Transform
}

impl HorizontalLayout {
    pub fn new(divide: f32) -> Self {
        Self {
            divide,
            left: Transform {
                translate: (0.0, 0.0),
                scale: (divide, 1.0)
            },
            right: Transform {
                translate: (divide, 0.0),
                scale: (1.0 - divide, 1.0)
            }
        }
    }

    pub fn select(&self, Position(x, _): Position) -> HorizontalPosition {
        if x < self.divide {
            HorizontalPosition::Left
        } else {
            HorizontalPosition::Right
        }
    }

    pub fn get(&self, horizontal_position: HorizontalPosition) -> Transform {
        match horizontal_position {
            HorizontalPosition::Left => self.left,
            HorizontalPosition::Right => self.right,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VerticalLayout {
    pub divide: f32,
    top: Transform,
    bottom: Transform
}

impl VerticalLayout {
    pub fn new(divide: f32) -> Self {
        Self {
            divide,
            top: Transform {
                translate: (0.0, 0.0),
                scale: (1.0, divide)
            },
            bottom: Transform {
                translate: (0.0, divide),
                scale: (1.0, 1.0 - divide)
            }
        }
    }

    pub fn select(&self, Position(_, y): Position) -> VerticalPosition {
        if y < self.divide {
            VerticalPosition::Top
        } else {
            VerticalPosition::Bottom
        }
    }

    pub fn get(&self, vertical_position: VerticalPosition) -> Transform {
        match vertical_position {
            VerticalPosition::Top => self.top,
            VerticalPosition::Bottom => self.bottom,
        }
    }
}
