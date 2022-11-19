use rtrb::{Consumer, Producer, RingBuffer};

use crate::instrument::InstrumentEvent;
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
    control_message_sender: Producer<SequencerControlMessage>
}

pub struct Sequencer {
    control_message_receiver: Consumer<SequencerControlMessage>,
    event_sender: EventSender<InstrumentEvent>,
    frames_processed: u64
}

impl Sequencer {
    pub fn new(event_sender: EventSender<InstrumentEvent>) -> (SequencerController, Self) {
        let (producer, consumer) = RingBuffer::new(1024);
        let sequencer_controller = SequencerController {
            control_message_sender: producer
        };
        let sequencer = Self {
            control_message_receiver: consumer,
            event_sender,
            frames_processed: 0
        };
        (sequencer_controller, sequencer)
    }
}

impl StereoFrameGenerator<Float> for Sequencer {
    fn next_frame(&mut self) -> StereoFrame<Float> {
        StereoFrame::zero()
    }
}