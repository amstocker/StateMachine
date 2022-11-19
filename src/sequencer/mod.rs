use winit::event_loop::EventLoopProxy;

use crate::sound::{SoundBank, MAX_SOUNDS, StereoFrame, StereoFrameGenerator, Float};
use crate::ui::EventSender;

mod node;
mod control_message;
pub use node::*;
pub use control_message::*;

pub const MAX_NODES: usize = 8;

// Note: implement copy for array items, and use generic const parameters!
// (revisiting this note... )


pub struct SequencerController {

}

pub struct Sequencer<E> where E: 'static {
    event_sender: EventSender<E>,
    frames_processed: u64
}

impl<E> Sequencer<E> {
    pub fn new(event_sender: EventSender<E>) -> Self {
        Self {
            event_sender,
            frames_processed: 0
        }
    }
}

impl<E> StereoFrameGenerator<Float> for Sequencer<E> {
    fn next_frame(&mut self) -> StereoFrame<Float> {
        StereoFrame::zero()
    }
}