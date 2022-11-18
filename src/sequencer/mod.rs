use std::time::Instant;

use crate::application::{Float, ApplicationEvent};
use crate::sound::{SoundBank, MAX_SOUNDS};
use crate::output::{StereoFrame, StereoFrameGenerator};

mod node;
mod control_message;
pub use node::*;
pub use control_message::*;
use winit::event_loop::EventLoopProxy;

pub const MAX_NODES: usize = 8;

// Note: implement copy for array items, and use generic const parameters!


pub struct Sequencer {
    event_loop_proxy: EventLoopProxy<ApplicationEvent>,
    frames_processed: u64
}

impl Sequencer {
    pub fn new(event_loop_proxy: EventLoopProxy<ApplicationEvent>) -> Self {
        Self {
            event_loop_proxy,
            frames_processed: 0
        }
    }
}

impl StereoFrameGenerator<Float> for Sequencer {
    fn next_frame(&mut self) -> StereoFrame<Float> {
        StereoFrame::zero()
    }
}