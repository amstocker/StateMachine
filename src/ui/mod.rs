pub mod fonts;
pub mod render;
pub mod sequencer;
pub mod mouse;
pub mod layout;
pub mod util;

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

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub translate: (f32, f32),
    pub scale: (f32, f32)
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            translate: (0.0, 0.0),
            scale: (1.0, 1.0)
        }
    }
}

impl Transform {
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
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct TransformInstance {
    transform: [f32; 4]
}

impl Default for TransformInstance {
    fn default() -> Self {
        Transform::default().into()
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
