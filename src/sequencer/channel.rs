use crate::sound::{SoundBankIndex, Float};

pub const MAX_CLIPS_PER_CHANNEL: usize = 32;
pub const MAX_JUNCTIONS_PER_CHANNEL: usize = 32;


#[derive(Debug, Default, Clone, Copy)]
pub enum PlayheadState {
    Playing,
    #[default] Stopped
}

#[derive(Debug, Default, Clone, Copy)]
pub enum PlayheadDirection {
    #[default] Right,
    Left
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Playhead {
    pub state: PlayheadState,
    pub location: u64,
    pub direction: PlayheadDirection
}

#[derive(Default)]
pub struct Channel {
    pub clips: [Clip; MAX_CLIPS_PER_CHANNEL],
    pub junctions: [Junction; MAX_JUNCTIONS_PER_CHANNEL],
    pub playhead: Playhead,
    pub playhead_override_this_frame: bool,
    pub length: u64
}

impl Channel {
    pub fn step_playhead_single_frame(&mut self) {
        if self.playhead_override_this_frame {
            self.playhead_override_this_frame = false;
            return;
        }
        if self.is_playing() {
            let playhead = &mut self.playhead;
            match playhead.direction {
                PlayheadDirection::Right => {
                    playhead.location += 1;
                    if playhead.location >= self.length {
                        playhead.state = PlayheadState::Stopped;
                    }
                },
                PlayheadDirection::Left => {
                    if playhead.location <= 0 {
                        playhead.state = PlayheadState::Stopped;
                    } else {
                        playhead.location -= 1;
                    }
                }
            }
        }
    }

    pub fn is_playing(&self) -> bool {
        match self.playhead.state {
            PlayheadState::Playing => true,
            PlayheadState::Stopped => false
        }
    }

    pub fn stop(&mut self) {
        self.playhead.state = PlayheadState::Stopped;
    }

    pub fn get_current_junction(&self) -> Option<Junction> {
        if !self.is_playing() {
            return None;
        }
        for junction in self.junctions {
            if junction.enabled && (junction.location == self.playhead.location) {
                return Some(junction);
            }
        }
        None
    }

    pub fn get_current_sound_bank_index(&self) -> Option<SoundBankIndex> {
        if !self.is_playing() {
            return None;
        }
        // Can probably cache this data somewhere, cache invalid if playhead mutated
        for clip in &self.clips {
            if !clip.enabled {
                continue;
            }
            if self.playhead.location >= clip.channel_location_start &&
               self.playhead.location <  clip.channel_location_end
            {
                return Some(SoundBankIndex {
                    source_index: clip.source_index,
                    frame_index: (self.playhead.location - clip.channel_location_start) as usize
                });
            }
        }
        None
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Clip {
    pub enabled: bool,
    pub source_index: usize,

    // Start and end point in terms of realtime frames
    pub channel_location_start: u64,
    pub channel_location_end: u64,

    // Mapping of source to clip
    //  - length of clip cannot exceed length of scaled and shifted source
    pub source_scale: Float,
    pub source_shift: u64
}

#[derive(Debug, Default, Clone, Copy)]
pub enum JunctionType {
    Jump {
        destination_channel_index: usize,
        destination_location: u64,
        split: bool
    },
    Reflect,
    #[default] Stop
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Junction {
    pub enabled: bool,
    pub location: u64,
    pub junction_type: JunctionType,
}
