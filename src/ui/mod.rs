pub mod fonts;
pub mod primitive;
pub mod component;
pub mod sequencer;
pub mod mouse;
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
    Top
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
            Top => 1.0
        }
    }
}

pub trait Transform {
    fn identity() -> Self;
    fn inverse(&self) -> Self;
    fn then(&self, other: Self) -> Self;
}

#[derive(Debug, Clone, Copy)]
pub struct UITransform {
    pub translate: (f32, f32),
    pub scale: (f32, f32)
}

pub trait Transformable {
    fn transform(&self) -> UITransform;
}

impl Transform for UITransform {
    fn identity() -> UITransform {
        UITransform {
            translate: (0.0, 0.0),
            scale: (1.0, 1.0)
        }
    }

    fn inverse(&self) -> UITransform {
        UITransform {
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

    fn then(&self, next: UITransform) -> UITransform {
        UITransform {
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
        UITransform::identity().into()
    }
}

impl Into<TransformInstance> for UITransform {
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
