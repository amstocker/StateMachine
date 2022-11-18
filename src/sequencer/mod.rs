use std::sync::Arc;
use std::time::Instant;

use rtrb::{RingBuffer, Consumer, Producer};

use crate::application::{Float};
use crate::sound::{SoundBank, MAX_SOUNDS};
use crate::output::{StereoFrame, StereoFrameGenerator};

mod node;
mod control_message;
pub use node::*;
pub use control_message::*;

pub const MAX_NODES: usize = 8;

// Note: implement copy for array items, and use generic const parameters!

pub struct SequencerController {
    producer: Producer<SequencerControlMessage>
}

impl SequencerController {
    pub fn play_sound_once(&mut self, sound_index: usize) {
        self.producer.push(SequencerControlMessage::PlaySoundOnce(sound_index)).unwrap();
    }
}


pub struct Sequencer {
    consumer: Consumer<SequencerControlMessage>,
    frames_processed: u64
}

impl Sequencer {
    pub fn new() -> (SequencerController, Self) {
        let (producer, consumer) = RingBuffer::new(1024);
        (
            SequencerController {
                producer
            },
            Self {
                consumer,
                frames_processed: 0
            }
        )
    }
}

impl StereoFrameGenerator<Float> for Sequencer {
    fn next_frame(&mut self) -> StereoFrame<Float> {
        StereoFrame::zero()
    }
}