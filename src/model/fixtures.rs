//! Small editable composition used by tests and `examples/control_score.rs`.
use super::*;

pub struct DrumFixture {
    pub project: Project,
    pub source: SequenceId,
    pub hits: [EventId; 4],
    pub placements: [PlacementId; 2],
    pub instrument: InstrumentId,
    pub mixer: MixerChannelId,
}

pub fn chopped_drums(ids: &mut IdGenerator) -> DrumFixture {
    let mut project = Project::new(ids, "Chopped control score", Tick(8 * TICKS_PER_BEAT));
    let source = ids.allocate();
    let hits = std::array::from_fn(|_| ids.allocate());
    let placements = std::array::from_fn(|_| ids.allocate());
    let sample = ids.allocate();
    let instrument = ids.allocate();
    let mixer = ids.allocate();
    let mut sequence = Sequence::new("Editable drum pattern", Tick(4 * TICKS_PER_BEAT));
    for (index, &id) in hits.iter().enumerate() {
        sequence.events.insert(
            id,
            Event {
                at: Tick(index as u64 * TICKS_PER_BEAT),
                kind: EventKind::Trigger {
                    velocity: if index % 2 == 0 { 1.0 } else { 0.6 },
                },
            },
        );
    }
    project.sequences.insert(source, sequence);
    project.sequence_order.push(source);
    let main = project.sequences.get_mut(&project.main_score).unwrap();
    main.placements.insert(
        placements[0],
        Placement {
            source,
            region: Region {
                start: Tick(0),
                end: Tick(2 * TICKS_PER_BEAT),
            },
            start: PlacementStart::At(Tick(0)),
            scale: TimeScale::IDENTITY,
            repeats: NonZeroU32::new(2).unwrap(),
        },
    );
    main.placements.insert(
        placements[1],
        Placement {
            source,
            region: Region {
                start: Tick(2 * TICKS_PER_BEAT),
                end: Tick(4 * TICKS_PER_BEAT),
            },
            start: PlacementStart::At(Tick(4 * TICKS_PER_BEAT)),
            scale: TimeScale::new(2, 1).unwrap(),
            repeats: NonZeroU32::new(1).unwrap(),
        },
    );
    project.samples.insert(
        sample,
        SampleReference {
            name: "Kick".into(),
            path: "assets/samples/kick.wav".into(),
        },
    );
    project.instruments.insert(
        instrument,
        Sampler {
            name: "Drum sampler".into(),
            sample,
            source_offset: SourceFrame(0),
            playback_rate: 1.0,
            gain: 1.0,
        },
    );
    project.control_routes.insert(
        ids.allocate(),
        ControlRoute {
            source,
            output: ControlPort::Trigger,
            instrument,
            input: ControlPort::Trigger,
        },
    );
    project.mixer_channels.insert(
        mixer,
        MixerChannel {
            name: "Drums".into(),
            gain: 1.0,
            pan: 0.0,
            mute: false,
            solo: false,
        },
    );
    project.audio_routes.insert(
        ids.allocate(),
        AudioRoute {
            source: AudioOutput::Instrument(instrument),
            destination: AudioInput::Mixer(mixer),
        },
    );
    project.audio_routes.insert(
        ids.allocate(),
        AudioRoute {
            source: AudioOutput::Mixer(mixer),
            destination: AudioInput::Master,
        },
    );
    DrumFixture {
        project,
        source,
        hits,
        placements,
        instrument,
        mixer,
    }
}
