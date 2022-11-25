pub mod fonts;
pub mod render;
pub mod sequencer;
pub mod mouse;
pub mod util;

mod application;
pub use application::*;


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