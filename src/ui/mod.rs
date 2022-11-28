pub mod fonts;
pub mod primitive;
pub mod component;
pub mod input;
pub mod layout;

mod application;
pub use application::*;
use bytemuck::{Pod, Zeroable};


#[derive(Debug, Default, Clone, Copy)]
pub enum Depth {
    Bottom,
    Back,
    #[default] Mid,
    Front,
    Menu,
    Modal,
    Top,
    Custom(f32)
}

impl Depth {
    pub fn z(&self) -> f32 {
        use Depth::*;
        match self {
            Bottom => 0.0,
            Back => 0.1,
            Mid => 0.5,
            Front => 0.7,
            Menu => 0.8,
            Modal => 0.9,
            Top => 1.0,
            Custom(z) => *z,
        }
    }
}

pub trait ApplyTransform {
    fn apply(&self, transform: Transform) -> Self;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Position (pub f32, pub f32);

impl ApplyTransform for Position {
    fn apply(&self, transform: Transform) -> Position {
        Position (
            transform.translate.0 + transform.scale.0 * self.0,
            transform.translate.1 + transform.scale.1 * self.1
        )
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Size (pub f32, pub f32);

impl ApplyTransform for Size {
    fn apply(&self, transform: Transform) -> Size {
        Size (
            transform.scale.0 * self.0,
            transform.scale.1 * self.1
        )
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Rectangle {
    position: Position,
    size: Size
}

impl ApplyTransform for Rectangle {
    fn apply(&self, transform: Transform) -> Self {
        Rectangle { 
            position: self.position.apply(transform),
            size: self.size.apply(transform)
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translate: (f32, f32),
    pub scale: (f32, f32)
}

pub trait Transformable {
    fn transform(&self) -> Transform;
}

impl Transform {
    pub fn identity() -> Transform {
        Transform {
            translate: (0.0, 0.0),
            scale: (1.0, 1.0)
        }
    }

    pub fn inverse(&self) -> Transform {
        Transform {
            translate: (
                -1.0 * self.translate.0 / self.scale.0,
                -1.0 * self.translate.1 / self.scale.1
            ),
            scale: (
                1.0 / self.scale.0,
                1.0 / self.scale.1
            )
        }
    }

    pub fn then(&self, next: Transform) -> Transform {
        Transform {
            translate: (
                next.scale.0 * self.translate.0 + next.translate.0,
                next.scale.1 * self.translate.1 + next.translate.1
            ),
            scale: (
                self.scale.0 * next.scale.0,
                self.scale.1 * next.scale.1
            )
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TransformInstance {
    transform: [f32; 4]
}

impl Default for TransformInstance {
    fn default() -> Self {
        Transform::identity().into()
    }
}

impl Into<TransformInstance> for Transform {
    fn into(self) -> TransformInstance {
        TransformInstance {
            transform: [
                self.translate.0,
                self.translate.1,
                self.scale.0,
                self.scale.1
            ]
        }
    }
}
