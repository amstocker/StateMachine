pub mod fonts;
pub mod render;
pub mod sequencer;
pub mod mouse;
pub mod util;

mod application;
pub use application::*;


#[derive(Debug, Default, Clone, Copy)]
pub enum Depth {
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
            Back => 0.1,
            Mid => 0.5,
            Front => 0.8,
            Menu => 0.9,
            Modal => 0.99,
            Top => 1.0
        }
    }
}