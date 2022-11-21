mod channel;
mod event;

use rtrb::{Consumer, Producer, RingBuffer};

pub use channel::*;
pub use event::*;
use crate::sound::{SoundBank, MAX_SOUNDS, StereoFrame, StereoFrameGenerator, Float};


pub const NUM_CHANNELS: usize = 4;
const RING_BUFFER_CAPACITY: usize = 1024;

pub struct SequencerController {
    pub control_message_sender: Producer<SequencerControlMessage>,
    pub event_receiver: Consumer<SequencerEvent>
}

pub struct Sequencer {
    control_message_receiver: Consumer<SequencerControlMessage>,
    event_sender: Producer<SequencerEvent>,
    state_summary: SequencerState,
    channels: [Channel; NUM_CHANNELS],
    sound_bank: SoundBank<Float>
}

impl Sequencer {
    pub fn new(sound_bank: SoundBank<Float>) -> (SequencerController, Self) {
        let (
            control_message_sender,
            control_message_receiver
        ) = RingBuffer::new(RING_BUFFER_CAPACITY);

        let (
            event_sender,
            event_receiver
        ) = RingBuffer::new(RING_BUFFER_CAPACITY);
        
        let sequencer_controller = SequencerController {
            control_message_sender,
            event_receiver
        };
        let sequencer = Self {
            control_message_receiver,
            event_sender,
            state_summary: Default::default(),
            channels: Default::default(),
            sound_bank
        };
        
        (sequencer_controller, sequencer)
    }

    fn handle_control_messsages(&mut self) {
        while let Ok(message) = self.control_message_receiver.pop() {
            match message {

            }
        }
    }

    fn update(&mut self) {
        self.handle_control_messsages();
    }
}

impl StereoFrameGenerator<Float> for Sequencer {
    fn next_frame(&mut self) -> StereoFrame<Float> {
        self.update();
        self.state_summary.total_frames_processed += 1;
        if self.state_summary.total_frames_processed % 800 == 0 {
            self.event_sender.push(SequencerEvent::Tick(self.state_summary)).unwrap();
        }
        StereoFrame::zero()
    }
}