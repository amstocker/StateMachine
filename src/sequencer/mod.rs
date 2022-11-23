mod channel;
mod event;

use rtrb::{Consumer, Producer, RingBuffer};

pub use channel::*;
pub use event::*;
use crate::sound::{SoundBank, StereoFrame, StereoFrameGenerator, Float};


pub const NUM_CHANNELS: usize = 4;
pub const DEFAULT_CHANNEL_LENGTH: u64 = 500_000;

const SYNC_INTERVAL: u64 = 500;  // frames
const RING_BUFFER_CAPACITY: usize = 1024;

pub struct SequencerController {
    pub control_message_sender: Producer<SequencerControlMessage>,
    pub event_receiver: Consumer<SequencerEvent>
}

pub struct Sequencer {
    control_message_receiver: Consumer<SequencerControlMessage>,
    event_sender: Producer<SequencerEvent>,
    summary: SequencerSummary,
    channels: [Channel; NUM_CHANNELS],
    playhead_mutations: [PlayheadMutation; NUM_CHANNELS],
    sound_bank: SoundBank<Float>
}

#[derive(Debug, Default)]
struct PlayheadMutation {
    updated_this_frame: bool,
    playhead: Playhead
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

        let mut channels: [Channel; NUM_CHANNELS] = Default::default();
        for channel in channels.iter_mut() {
            channel.length = DEFAULT_CHANNEL_LENGTH;
        }
        let sequencer = Self {
            control_message_receiver,
            event_sender,
            summary: Default::default(),
            channels,
            playhead_mutations: Default::default(),
            sound_bank
        };
        
        (sequencer_controller, sequencer)
    }

    fn set_clip(&mut self, index: ChannelItemIndex, clip: Clip) {
        self.channels[index.channel_index]
            .clips[index.item_index] = clip;
    }

    fn set_junction(&mut self, index: ChannelItemIndex, junction: Junction) {
        self.channels[index.channel_index]
            .junctions[index.item_index] = junction;
    }

    fn set_playhead(&mut self, index: ChannelItemIndex, playhead: Playhead) {
        self.channels[index.channel_index].playhead_override_this_frame = true;
        self.channels[index.channel_index].playhead = playhead;
    }

    fn handle_control_messsages(&mut self) {
        while let Ok(message) = self.control_message_receiver.pop() {
            use SequencerControlMessage::*;
            match message {
                SyncClip { index, clip } => {
                    self.set_clip(index, clip);
                },
                SyncJunction { index, junction } => {
                    self.set_junction(index, junction);
                },
                SyncPlayhead { index, playhead } => {
                    self.set_playhead(index, playhead);
                }
            }
        }
    }

    fn step_playheads_single_frame(&mut self) {
        for channel in &mut self.channels {
            channel.step_playhead_single_frame();
        }
    }

    fn handle_junctions_single_frame(&mut self) {
        for (channel_index, channel) in self.channels.iter_mut().enumerate() {
            if let Some(junction) = channel.get_current_junction() {
                match junction.junction_type {
                    JunctionType::Jump {
                        destination_channel_index,
                        destination_location,
                        split
                    } => {
                        self.playhead_mutations[destination_channel_index] = PlayheadMutation {
                            updated_this_frame: true,
                            playhead: Playhead { 
                                state: PlayheadState::Playing,
                                location: destination_location,
                                direction: channel.playhead.direction
                            }
                        };
                        if !split {
                            channel.stop();
                        }
                    },
                    JunctionType::Reflect => {
                        self.playhead_mutations[channel_index] = PlayheadMutation {
                            updated_this_frame: true,
                            playhead: Playhead { 
                                direction: match channel.playhead.direction {
                                    PlayheadDirection::Right => PlayheadDirection::Left,
                                    PlayheadDirection::Left => PlayheadDirection::Right,
                                },
                                ..channel.playhead
                            }
                        }
                    },
                    JunctionType::Stop => {
                        channel.stop();
                    }
                }
            }
        }
    }

    fn handle_playhead_mutations_single_frame(&mut self) {
        for (channel, mutation) in self.channels.iter_mut()
            .zip(self.playhead_mutations.iter_mut())
        {
            if mutation.updated_this_frame {
                channel.playhead = mutation.playhead;
                mutation.updated_this_frame = false;
            }
        }
    }

    fn update_summary_single_frame(&mut self) {
        for (channel, playhead) in self.channels.iter().zip(self.summary.playheads.iter_mut()) {
            *playhead = channel.playhead;
        }
        self.summary.total_frames_processed += 1;
    }

    fn update_single_frame(&mut self) {
        self.sound_bank.update();
        self.step_playheads_single_frame();
        self.handle_junctions_single_frame();
        self.handle_playhead_mutations_single_frame();
        self.update_summary_single_frame();        
    }

    fn sum_output_single_frame(&mut self) -> StereoFrame<Float> {
        let mut out_frame = StereoFrame::zero();
        for channel in &self.channels {
            out_frame += channel.get_current_sound_bank_index()
                                .and_then(
                                    |index| self.sound_bank.get_frame(index)
                                )
                                .unwrap_or_default();
        }
        out_frame
    }

    fn send_summary(&mut self) {
        self.event_sender.push(SequencerEvent::Tick(self.summary)).unwrap();
    }
}

impl StereoFrameGenerator<Float> for Sequencer {
    fn next_frame(&mut self) -> StereoFrame<Float> {
        if self.summary.total_frames_processed % SYNC_INTERVAL == 0 {
            self.handle_control_messsages();
            self.send_summary();
        }
        self.update_single_frame();
        self.sum_output_single_frame()
    }
}