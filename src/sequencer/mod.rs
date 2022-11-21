mod channel;
mod event;

use rtrb::{Consumer, Producer, RingBuffer};

pub use channel::*;
pub use event::*;
use crate::sound::{SoundBank, MAX_SOUNDS, StereoFrame, StereoFrameGenerator, Float};


pub const NUM_CHANNELS: usize = 4;

pub struct SequencerController {
    pub control_message_sender: Producer<SequencerControlMessage>,
    pub event_receiver: Consumer<SequencerEvent>
}

pub struct Sequencer {
    control_message_receiver: Consumer<SequencerControlMessage>,
    event_sender: Producer<SequencerEvent>,
    state: SequencerState,
    channels: [Channel; NUM_CHANNELS]
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
            state: Default::default(),
            channels: Default::default()
        };
        
        (sequencer_controller, sequencer)
    }

    fn update(&mut self) {
        while let Ok(message) = self.control_message_receiver.pop() {
            match message {

            }
        }
    }
}

impl StereoFrameGenerator<Float> for Sequencer {
    fn next_frame(&mut self) -> StereoFrame<Float> {
        self.update();
        self.state.total_frames_processed += 1;
        if self.state.total_frames_processed % 800 == 0 {
            self.event_sender.push(SequencerEvent::Tick(self.state)).unwrap();
        }
        StereoFrame::zero()
    }
}