mod channel;
pub mod compatibility;
mod control_loop;
mod event;
pub mod interface;

#[cfg(test)]
mod tests;

use std::collections::VecDeque;

use rtrb::{Consumer, Producer, PushError, RingBuffer};

use crate::sound::{Float, SoundBank, StereoFrame, StereoFrameGenerator};
pub use channel::*;
pub use event::*;

pub const NUM_CHANNELS: usize = 4;
pub const DEFAULT_CHANNEL_LENGTH: u64 = 500_000;

const SYNC_INTERVAL: u64 = 256; // frames
const RING_BUFFER_CAPACITY: usize = 1024;
const MAX_PENDING_CONTROLS: usize = 1024;

pub struct SequencerController {
    control_message_sender: Producer<SequencerControlMessage>,
    pending_controls: VecDeque<SequencerControlMessage>,
    pub event_receiver: Consumer<SequencerEvent>,
}

impl SequencerController {
    /// Accept an ordered edit, or reject it before the caller changes its model.
    /// This pending queue belongs to the UI, never the audio callback.
    pub fn send(&mut self, message: SequencerControlMessage) -> Result<(), EditError> {
        message.validate()?;
        self.flush_pending()?;
        if self.pending_controls.is_empty() {
            match self.control_message_sender.push(message) {
                Ok(()) => return Ok(()),
                Err(PushError::Full(message)) => self.pending_controls.push_back(message),
            }
        } else if self.pending_controls.len() < MAX_PENDING_CONTROLS {
            self.pending_controls.push_back(message);
        } else {
            return Err(EditError::QueueFull);
        }
        Ok(())
    }

    pub fn flush_pending(&mut self) -> Result<(), EditError> {
        if self.control_message_sender.is_abandoned() {
            return Err(EditError::EngineDisconnected);
        }
        while let Some(&message) = self.pending_controls.front() {
            if self.control_message_sender.push(message).is_err() {
                break;
            }
            self.pending_controls.pop_front();
        }
        Ok(())
    }
}

pub struct Sequencer {
    control_message_receiver: Consumer<SequencerControlMessage>,
    event_sender: Producer<SequencerEvent>,
    summary: SequencerSummary,
    channels: [Channel; NUM_CHANNELS],
    playhead_mutations: [PlayheadMutation; NUM_CHANNELS],
    sound_bank: SoundBank<Float>,
}

#[derive(Debug, Default)]
struct PlayheadMutation {
    updated_this_frame: bool,
    playhead: Playhead,
}

impl Sequencer {
    pub fn new(sound_bank: SoundBank<Float>) -> (SequencerController, Self) {
        let (control_message_sender, control_message_receiver) =
            RingBuffer::new(RING_BUFFER_CAPACITY);

        let (event_sender, event_receiver) = RingBuffer::new(RING_BUFFER_CAPACITY);

        let sequencer_controller = SequencerController {
            control_message_sender,
            pending_controls: VecDeque::with_capacity(MAX_PENDING_CONTROLS),
            event_receiver,
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
            sound_bank,
        };

        (sequencer_controller, sequencer)
    }

    fn set_clip(&mut self, index: ChannelItemIndex, clip: Clip) {
        self.channels[index.channel_index].clips[index.item_index] = clip;
    }

    fn set_junction(&mut self, index: ChannelItemIndex, junction: Junction) {
        self.channels[index.channel_index].junctions[index.item_index] = junction;
    }

    fn set_playhead(&mut self, index: ChannelItemIndex, playhead: Playhead) {
        self.channels[index.channel_index].playhead_override_this_frame = true;
        self.channels[index.channel_index].playhead = playhead;
    }

    fn handle_control_messages(&mut self) {
        // A producer that keeps filling the queue cannot prolong this indefinitely.
        for _ in 0..RING_BUFFER_CAPACITY {
            let Ok(message) = self.control_message_receiver.pop() else {
                break;
            };
            if message.validate().is_err() {
                continue;
            }
            use SequencerControlMessage::*;
            match message {
                SyncClip { index, clip } => {
                    self.set_clip(index, clip);
                }
                SyncJunction { index, junction } => {
                    self.set_junction(index, junction);
                }
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
                        split,
                    } => {
                        self.playhead_mutations[destination_channel_index] = PlayheadMutation {
                            updated_this_frame: true,
                            playhead: Playhead {
                                state: PlayheadState::Playing,
                                location: destination_location,
                                direction: channel.playhead.direction,
                            },
                        };
                        if !split {
                            channel.stop();
                        }
                    }
                    JunctionType::Reflect => {
                        self.playhead_mutations[channel_index] = PlayheadMutation {
                            updated_this_frame: true,
                            playhead: Playhead {
                                direction: match channel.playhead.direction {
                                    PlayheadDirection::Right => PlayheadDirection::Left,
                                    PlayheadDirection::Left => PlayheadDirection::Right,
                                },
                                ..channel.playhead
                            },
                        }
                    }
                    JunctionType::Stop => {
                        channel.stop();
                    }
                }
            }
        }
    }

    fn handle_playhead_mutations_single_frame(&mut self) {
        for (channel, mutation) in self
            .channels
            .iter_mut()
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
            out_frame += channel
                .get_current_sound_bank_index()
                .and_then(|index| self.sound_bank.get_frame(index))
                .unwrap_or_default();
        }
        out_frame
    }

    fn send_summary(&mut self) {
        // Status is disposable: UI stalls must not interrupt audio processing.
        let _ = self.event_sender.push(SequencerEvent::Tick(self.summary));
    }
}

impl StereoFrameGenerator<Float> for Sequencer {
    fn next_frame(&mut self) -> StereoFrame<Float> {
        if self.summary.total_frames_processed % SYNC_INTERVAL == 0 {
            self.handle_control_messages();
            self.send_summary();
        }
        self.update_single_frame();
        self.sum_output_single_frame()
    }
}
