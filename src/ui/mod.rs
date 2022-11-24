pub mod fonts;
pub mod render;
pub mod sequencer;
pub mod mouse;
pub mod util;

mod application;
pub use application::*;


pub enum Depth {
    Back,
    Mid,
    Front
}

impl Depth {
    pub fn z(&self) -> f32 {
        match self {
            Depth::Back => 0.0,
            Depth::Mid => 0.5,
            Depth::Front => 1.0,
        }
    }
}