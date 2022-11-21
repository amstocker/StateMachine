use rtrb::{Consumer, Producer, RingBuffer};

use crate::sound::{SoundBank, MAX_SOUNDS, StereoFrame, StereoFrameGenerator, Float};

mod node;
mod event;
pub use node::*;
pub use event::*;

pub const MAX_NODES: usize = 8;

pub type NodeIndex = usize;

// Note: implement copy for array items, and use generic const parameters!
// (revisiting this note... it means that triggers etc should impl Copy,
//  so that N can be a generic const and array init is easier)


pub struct SequencerController {
    pub control_message_sender: Producer<SequencerControlMessage>,
    pub event_receiver: Consumer<SequencerEvent>
}

pub struct Sequencer {
    control_message_receiver: Consumer<SequencerControlMessage>,
    event_sender: Producer<SequencerEvent>,
    frames_processed: u64
}

impl Sequencer {
    pub fn new() -> (SequencerController, Self) {
        let (
            control_message_sender,
            control_message_receiver
        ) = RingBuffer::new(1024);

        let (
            event_sender,
            event_receiver
        ) = RingBuffer::new(1024);
        
        let sequencer_controller = SequencerController {
            control_message_sender,
            event_receiver
        };
        let sequencer = Self {
            control_message_receiver,
            event_sender,
            frames_processed: 0
        };
        
        (sequencer_controller, sequencer)
    }

    fn update(&mut self) {
        while let Ok(message) = self.control_message_receiver.pop() {
            match message {
                SequencerControlMessage::EnableSound(_) => todo!(),
                SequencerControlMessage::DisableSound(_) => todo!(),
                SequencerControlMessage::PlaySoundOnce(_) => todo!(),
                SequencerControlMessage::IncrSoundIndex(_) => todo!(),
                SequencerControlMessage::DecrSoundIndex(_) => todo!(),
            }
        }
    }
}

impl StereoFrameGenerator<Float> for Sequencer {
    fn next_frame(&mut self) -> StereoFrame<Float> {
        self.update();
        self.frames_processed += 1;
        if self.frames_processed % 10000 == 0 {
            self.event_sender.push(SequencerEvent::Tick(self.frames_processed)).unwrap();
        }
        StereoFrame::zero()
    }
}