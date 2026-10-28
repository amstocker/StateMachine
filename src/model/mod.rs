//! Editable composition data, independent of the UI, audio device, decoded
//! buffers, and runtime playheads. See MODEL.md for the initial semantics.

mod evaluate;
pub mod fixtures;
mod ids;
mod time;
mod validate;

pub use evaluate::*;
pub use ids::*;
pub use time::*;

use std::{collections::BTreeMap, num::NonZeroU32, path::PathBuf};

#[derive(Debug, Clone, PartialEq)]
pub struct Project {
    pub name: String,
    pub main_score: SequenceId,
    pub tempo: Tempo,
    pub sequences: BTreeMap<SequenceId, Sequence>,
    /// View ordering only; references always use IDs, never row numbers.
    pub sequence_order: Vec<SequenceId>,
    pub samples: BTreeMap<SampleId, SampleReference>,
    pub instruments: BTreeMap<InstrumentId, Sampler>,
    pub control_routes: BTreeMap<ControlRouteId, ControlRoute>,
    pub mixer_channels: BTreeMap<MixerChannelId, MixerChannel>,
    pub audio_routes: BTreeMap<AudioRouteId, AudioRoute>,
}

impl Project {
    pub fn new(ids: &mut IdGenerator, name: impl Into<String>, length: Tick) -> Self {
        let main_score = ids.allocate();
        Self {
            name: name.into(),
            main_score,
            tempo: Tempo::from_milli_bpm(120_000).unwrap(),
            sequences: BTreeMap::from([(main_score, Sequence::new("Main score", length))]),
            sequence_order: vec![main_score],
            samples: BTreeMap::new(),
            instruments: BTreeMap::new(),
            control_routes: BTreeMap::new(),
            mixer_channels: BTreeMap::new(),
            audio_routes: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sequence {
    pub name: String,
    pub length: Tick,
    pub events: BTreeMap<EventId, Event>,
    pub placements: BTreeMap<PlacementId, Placement>,
}

impl Sequence {
    pub fn new(name: impl Into<String>, length: Tick) -> Self {
        Self {
            name: name.into(),
            length,
            events: BTreeMap::new(),
            placements: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
    pub at: Tick,
    pub kind: EventKind,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EventKind {
    /// One-shot onset. Gates/sustained notes and reverse are deliberately deferred.
    Trigger {
        velocity: f32,
    },
    SetParameter {
        parameter: Parameter,
        value: f32,
    },
    /// Launch an OnLaunch placement owned by this sequence. Each execution
    /// requests an independent instance; this model contains no global playhead.
    Launch {
        placement: PlacementId,
    },
    /// Stop this placement's children belonging to the calling instance.
    Stop {
        placement: PlacementId,
    },
    StopCurrent,
}

impl EventKind {
    pub fn output_port(self) -> Option<ControlPort> {
        match self {
            Self::Trigger { .. } => Some(ControlPort::Trigger),
            Self::SetParameter { parameter, .. } => Some(ControlPort::Parameter(parameter)),
            _ => None,
        }
    }

    fn order(self) -> u8 {
        match self {
            Self::Stop { .. } | Self::StopCurrent => 0,
            Self::SetParameter { .. } => 1,
            Self::Launch { .. } => 2,
            Self::Trigger { .. } => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parameter {
    Gain,
    PlaybackRate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlPort {
    Trigger,
    Parameter(Parameter),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Region {
    pub start: Tick,
    pub end: Tick,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementStart {
    At(Tick),
    OnLaunch,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Placement {
    pub source: SequenceId,
    /// Half-open [start, end); source events stay in the source definition.
    pub region: Region,
    pub start: PlacementStart,
    pub scale: TimeScale,
    pub repeats: NonZeroU32,
}

impl Placement {
    pub fn duration(self) -> Result<ScoreTime, ModelError> {
        let length = self
            .region
            .end
            .0
            .checked_sub(self.region.start.0)
            .filter(|length| *length > 0)
            .ok_or(ModelError::InvalidRegion)?;
        ScoreTime::from(Tick(length))
            .scaled(self.scale)?
            .repeated(self.repeats.get())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleReference {
    pub name: String,
    /// A locator, not decoded samples. Missing files are an asset-resolution issue.
    pub path: PathBuf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Sampler {
    pub name: String,
    pub sample: SampleId,
    pub source_offset: SourceFrame,
    pub playback_rate: f32,
    pub gain: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlRoute {
    pub source: SequenceId,
    pub output: ControlPort,
    pub instrument: InstrumentId,
    pub input: ControlPort,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MixerChannel {
    pub name: String,
    pub gain: f32,
    pub pan: f32,
    pub mute: bool,
    pub solo: bool,
}

/// Audio and control ports cannot be substituted for one another. The first
/// audio model is stereo; effects and channel-layout negotiation come later.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioOutput {
    Instrument(InstrumentId),
    Mixer(MixerChannelId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioInput {
    Mixer(MixerChannelId),
    Master,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioRoute {
    pub source: AudioOutput,
    pub destination: AudioInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelError {
    MissingSequence(SequenceId),
    MissingSample(SampleId),
    MissingInstrument(InstrumentId),
    MissingMixer(MixerChannelId),
    MissingPlacement(PlacementId),
    DuplicateEvent(EventId),
    DuplicatePlacement(PlacementId),
    InvalidSequenceOrder,
    InvalidTime,
    InvalidRegion,
    TimeOverflow,
    OutOfBounds,
    InvalidValue,
    IncompatibleControlPorts(ControlRouteId),
    LaunchRequiresTriggeredPlacement(PlacementId),
    SequenceCycle,
    AudioCycle,
    EvaluationLimit,
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid project or evaluation: {self:?}")
    }
}

impl std::error::Error for ModelError {}
