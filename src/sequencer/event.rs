use crate::sequencer::{
    Clip, Junction, JunctionType, Playhead, DEFAULT_CHANNEL_LENGTH, MAX_CLIPS_PER_CHANNEL,
    MAX_JUNCTIONS_PER_CHANNEL, NUM_CHANNELS,
};
use crate::sound::MAX_SOUNDS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditError {
    InvalidChannel,
    InvalidClip,
    InvalidJunction,
    InvalidSound,
    InvalidLocation,
    InvalidRate,
    ClipsFull,
    JunctionsFull,
    QueueFull,
    EngineDisconnected,
}

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidChannel => "invalid channel",
            Self::InvalidClip => "invalid clip",
            Self::InvalidJunction => "invalid junction",
            Self::InvalidSound => "invalid sound slot",
            Self::InvalidLocation => "location is outside the channel",
            Self::InvalidRate => "playback rate must be finite and positive",
            Self::ClipsFull => "channel has no free clip slots",
            Self::JunctionsFull => "channel has no free junction slots",
            Self::QueueFull => "too many pending edits; try again",
            Self::EngineDisconnected => "audio engine is disconnected",
        })
    }
}

impl std::error::Error for EditError {}

#[derive(Debug, Clone, Copy)]
pub struct ChannelItemIndex {
    pub channel_index: usize,
    pub item_index: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum SequencerControlMessage {
    SyncClip {
        index: ChannelItemIndex,
        clip: Clip,
    },
    SyncJunction {
        index: ChannelItemIndex,
        junction: Junction,
    },
    SyncPlayhead {
        index: ChannelItemIndex,
        playhead: Playhead,
    },
}

impl SequencerControlMessage {
    /// Validate against the prototype's fixed channel sizes before queuing or
    /// applying an edit. This stays at the compatibility boundary until the
    /// new project model is connected to the scheduler in phases 3–4.
    pub fn validate(&self) -> Result<(), EditError> {
        let (Self::SyncClip { index, .. }
        | Self::SyncJunction { index, .. }
        | Self::SyncPlayhead { index, .. }) = self;
        if index.channel_index >= NUM_CHANNELS {
            return Err(EditError::InvalidChannel);
        }
        match *self {
            Self::SyncClip { index, clip } => {
                if index.item_index >= MAX_CLIPS_PER_CHANNEL {
                    return Err(EditError::InvalidClip);
                }
                if clip.enabled {
                    if clip.source_index >= MAX_SOUNDS {
                        return Err(EditError::InvalidSound);
                    }
                    if clip.channel_location_start >= clip.channel_location_end
                        || clip.channel_location_end > DEFAULT_CHANNEL_LENGTH
                    {
                        return Err(EditError::InvalidLocation);
                    }
                    if !clip.source_scale.is_finite() || clip.source_scale <= 0.0 {
                        return Err(EditError::InvalidRate);
                    }
                }
            }
            Self::SyncJunction { index, junction } => {
                if index.item_index >= MAX_JUNCTIONS_PER_CHANNEL {
                    return Err(EditError::InvalidJunction);
                }
                if junction.enabled {
                    if junction.location >= DEFAULT_CHANNEL_LENGTH {
                        return Err(EditError::InvalidLocation);
                    }
                    if let JunctionType::Jump {
                        destination_channel_index,
                        destination_location,
                        ..
                    } = junction.junction_type
                    {
                        if destination_channel_index >= NUM_CHANNELS {
                            return Err(EditError::InvalidChannel);
                        }
                        if destination_location >= DEFAULT_CHANNEL_LENGTH {
                            return Err(EditError::InvalidLocation);
                        }
                    }
                }
            }
            Self::SyncPlayhead { playhead, .. } => {
                if playhead.location >= DEFAULT_CHANNEL_LENGTH {
                    return Err(EditError::InvalidLocation);
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SequencerSummary {
    pub playheads: [Playhead; NUM_CHANNELS],
    pub total_frames_processed: u64,
}

#[derive(Debug, Clone, Copy)]
pub enum SequencerEvent {
    Tick(SequencerSummary),
}
