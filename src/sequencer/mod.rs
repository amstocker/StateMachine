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
    summary: SequencerState,
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
            summary: Default::default(),
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
        // increment active playheads
        // resolve collisions? maybe not, just let it work itself out

        // should calculate this number as approximately sample_rate / 60
        self.summary.total_frames_processed += 1;
        if self.summary.total_frames_processed % 500 == 0 {
            self.event_sender.push(SequencerEvent::Tick(self.summary)).unwrap();
        }
    }

    fn sum_output(&mut self) -> StereoFrame<Float> {
        self.update();
        StereoFrame::zero()
    }

}

impl StereoFrameGenerator<Float> for Sequencer {
    fn next_frame(&mut self) -> StereoFrame<Float> {
        self.handle_control_messsages();
        self.update();
        self.sum_output()
    }
}