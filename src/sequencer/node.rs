pub const PLAYHEADS_PER_NODE: usize = 4;
pub const TRIGGERS_PER_NODE: usize = 4;


/*
 * A Node a reference to a particular sound in the SoundBank
 * that exists within the sequencer.  Each node has some number
 * of configurable Playheads which can be triggered at any time,
 * but only one Playhead may be playing at a given time.
 * 
 * Each Node can also emit up to some number of triggers, either
 * at the start or end of the play cycle.
 */
#[derive(Default)]
pub struct Node {
    pub enabled: bool,
    pub sound_index: usize,
    pub playheads: [Playhead; PLAYHEADS_PER_NODE],
    pub triggers: [Trigger; TRIGGERS_PER_NODE]
}

#[derive(Default)]
pub struct Playhead {
    pub is_pending: bool,
    pub is_playing: bool,
    pub current_frame_index: usize,
    pub frames_until_play: usize,
    pub triggered_this_frame: bool,
    pub triggered_last_frame: bool
}

#[derive(Default)]
pub struct Trigger {
    pub enabled: bool,
    pub target_index: usize,
    pub target_playhead_index: usize,
    pub when: When,
    pub delay: Delay
}

pub enum When {
    StartOfCycle,
    EndOfCycle
}

impl Default for When {
    fn default() -> Self {
        Self::StartOfCycle
    }
}

pub enum Delay {
    Frames(usize)
}

// Minimum delay is 1 frame
impl Default for Delay {
    fn default() -> Self {
        Self::Frames(1)
    }
}
