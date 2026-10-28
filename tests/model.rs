use state_machine::model::{fixtures::chopped_drums, *};
use std::num::NonZeroU32;

fn evaluate(project: &Project) -> Vec<EvaluatedEvent> {
    project.evaluate(EvaluationLimits::default()).unwrap()
}

fn times(events: &[EvaluatedEvent]) -> Vec<(u128, u128)> {
    events.iter().map(|event| event.at.ratio()).collect()
}

fn placement(source: SequenceId, start: u64, end: u64, at: u64) -> Placement {
    Placement {
        source,
        region: Region {
            start: Tick(start),
            end: Tick(end),
        },
        start: PlacementStart::At(Tick(at)),
        scale: TimeScale::IDENTITY,
        repeats: NonZeroU32::new(1).unwrap(),
    }
}

#[test]
fn chopped_score_resolves_through_instrument_and_separate_mixer() {
    let mut ids = IdGenerator::default();
    let fixture = chopped_drums(&mut ids);
    let events = evaluate(&fixture.project);
    assert_eq!(
        times(&events),
        vec![(0, 1), (960, 1), (1920, 1), (2880, 1), (3840, 1), (5760, 1)]
    );
    assert_eq!(fixture.project.sequence_order.len(), 2);
    assert_eq!(fixture.project.instruments.len(), 1);
    for event in events {
        assert_eq!(event.source, fixture.source);
        assert_eq!(event.deliveries.len(), 1);
        let delivery = &event.deliveries[0];
        assert_eq!(delivery.instrument, fixture.instrument);
        assert_eq!(delivery.audio_paths.len(), 1);
        assert_eq!(delivery.audio_paths[0].mixer_channels, [fixture.mixer]);
        assert_eq!(delivery.audio_paths[0].routes.len(), 2);
    }
}

#[test]
fn source_edit_updates_every_use_and_events_can_enter_and_leave_slices() {
    let mut ids = IdGenerator::default();
    let mut fixture = chopped_drums(&mut ids);
    let hits = &mut fixture
        .project
        .sequences
        .get_mut(&fixture.source)
        .unwrap()
        .events;
    hits.get_mut(&fixture.hits[2]).unwrap().at = Tick(1440);
    hits.get_mut(&fixture.hits[2]).unwrap().kind = EventKind::Trigger { velocity: 0.25 };
    let events = evaluate(&fixture.project);
    let affected: Vec<_> = events
        .into_iter()
        .filter(|event| event.event == fixture.hits[2])
        .collect();
    assert_eq!(times(&affected), [(1440, 1), (3360, 1)]);
    assert!(affected
        .iter()
        .all(|event| event.placement_path[0].placement == fixture.placements[0]));
    assert!(affected
        .iter()
        .all(|event| event.kind == EventKind::Trigger { velocity: 0.25 }));
    assert_ne!(affected[0].placement_path, affected[1].placement_path);
}

#[test]
fn placement_edit_changes_one_use_without_rewriting_source() {
    let mut ids = IdGenerator::default();
    let mut fixture = chopped_drums(&mut ids);
    let source = fixture.project.sequences[&fixture.source].clone();
    fixture
        .project
        .sequences
        .get_mut(&fixture.project.main_score)
        .unwrap()
        .placements
        .get_mut(&fixture.placements[1])
        .unwrap()
        .start = PlacementStart::At(Tick(2880));
    let events = evaluate(&fixture.project);
    assert_eq!(fixture.project.sequences[&fixture.source], source);
    assert_eq!(
        times(&events),
        [(0, 1), (960, 1), (1920, 1), (2880, 1), (2880, 1), (4800, 1)]
    );
}

#[test]
fn ids_survive_reordering_and_delete_restore_and_are_not_recycled() {
    let mut ids = IdGenerator::default();
    let mut fixture = chopped_drums(&mut ids);
    let before = evaluate(&fixture.project);
    fixture.project.sequence_order.reverse();
    assert_eq!(evaluate(&fixture.project), before);
    let source = fixture.project.sequences.get_mut(&fixture.source).unwrap();
    let removed = source.events.remove(&fixture.hits[0]).unwrap();
    let new_id: EventId = ids.allocate();
    assert!(!fixture.hits.contains(&new_id));
    source.events.insert(fixture.hits[0], removed);
    assert_eq!(evaluate(&fixture.project), before);
}

#[test]
fn switching_instrument_keeps_arrangement_and_control_timing() {
    let mut ids = IdGenerator::default();
    let mut fixture = chopped_drums(&mut ids);
    let before = evaluate(&fixture.project);
    let original_sequences = fixture.project.sequences.clone();
    let replacement_sample = ids.allocate();
    fixture.project.samples.insert(
        replacement_sample,
        SampleReference {
            name: "Snare".into(),
            path: "assets/samples/snare.wav".into(),
        },
    );
    let sampler = fixture
        .project
        .instruments
        .get_mut(&fixture.instrument)
        .unwrap();
    sampler.sample = replacement_sample;
    sampler.playback_rate = 0.5;
    sampler.source_offset = SourceFrame(100);
    assert_eq!(evaluate(&fixture.project), before);
    assert_eq!(fixture.project.sequences, original_sequences);
}

#[test]
fn slice_is_half_open_and_repeats_do_not_duplicate_boundary_events() {
    let mut ids = IdGenerator::default();
    let mut project = Project::new(&mut ids, "Boundaries", Tick(30));
    let source = ids.allocate();
    let mut sequence = Sequence::new("Source", Tick(30));
    for at in [9, 10, 19, 20] {
        sequence.events.insert(
            ids.allocate(),
            Event {
                at: Tick(at),
                kind: EventKind::Trigger { velocity: 1.0 },
            },
        );
    }
    project.sequences.insert(source, sequence);
    project.sequence_order.push(source);
    project
        .sequences
        .get_mut(&project.main_score)
        .unwrap()
        .placements
        .insert(
            ids.allocate(),
            Placement {
                repeats: NonZeroU32::new(2).unwrap(),
                ..placement(source, 10, 20, 5)
            },
        );
    assert_eq!(
        times(&evaluate(&project)),
        [(5, 1), (14, 1), (15, 1), (24, 1)]
    );
}

#[test]
fn nested_transforms_keep_fractional_ticks_until_final_frame_conversion() {
    let mut ids = IdGenerator::default();
    let mut project = Project::new(&mut ids, "Nested", Tick(12));
    let inner = ids.allocate();
    let outer = ids.allocate();
    let mut source = Sequence::new("Inner", Tick(6));
    source.events.insert(
        ids.allocate(),
        Event {
            at: Tick(1),
            kind: EventKind::Trigger { velocity: 1.0 },
        },
    );
    project.sequences.insert(inner, source);
    let mut wrapper = Sequence::new("Outer", Tick(4));
    wrapper.placements.insert(
        ids.allocate(),
        Placement {
            scale: TimeScale::new(1, 3).unwrap(),
            ..placement(inner, 0, 6, 0)
        },
    );
    project.sequences.insert(outer, wrapper);
    project.sequence_order.extend([inner, outer]);
    project
        .sequences
        .get_mut(&project.main_score)
        .unwrap()
        .placements
        .insert(
            ids.allocate(),
            Placement {
                scale: TimeScale::new(3, 1).unwrap(),
                ..placement(outer, 0, 4, 0)
            },
        );
    let events = evaluate(&project);
    assert_eq!(times(&events), [(1, 1)]);
    assert_eq!(events[0].placement_path.len(), 2);
    assert_eq!(
        project.tempo.device_frame(events[0].at, 48_000).unwrap(),
        DeviceFrame(25)
    );
    project.main_score = outer;
    assert_eq!(times(&evaluate(&project)), [(1, 3)]);
}

#[test]
fn device_rate_changes_preserve_musical_time_and_round_only_once() {
    let tempo = Tempo::from_milli_bpm(120_000).unwrap();
    for rate in [44_100, 48_000, 96_000] {
        assert_eq!(
            tempo.device_frame(Tick(1920).into(), rate).unwrap(),
            DeviceFrame(rate.into())
        );
    }
    assert_eq!(
        tempo.device_frame(Tick(960).into(), 1).unwrap(),
        DeviceFrame(1)
    );
    assert_eq!(
        tempo.device_frame(Tick(0).into(), 0),
        Err(ModelError::InvalidTime)
    );
    assert_eq!(Tempo::from_milli_bpm(0), Err(ModelError::InvalidTime));
    assert_eq!(TimeScale::new(0, 1), Err(ModelError::InvalidTime));
    assert_eq!(TimeScale::new(1, 0), Err(ModelError::InvalidTime));
}

#[test]
fn rational_ordering_and_arithmetic_handle_large_values_without_wrapping() {
    for a in 0..10 {
        for b in 1..10 {
            for c in 0..10 {
                for d in 1..10 {
                    let left = ScoreTime::from(Tick(a))
                        .scaled(TimeScale::new(1, b).unwrap())
                        .unwrap();
                    let right = ScoreTime::from(Tick(c))
                        .scaled(TimeScale::new(1, d).unwrap())
                        .unwrap();
                    assert_eq!(
                        left.cmp(&right),
                        (a * u64::from(d)).cmp(&(c * u64::from(b)))
                    );
                }
            }
        }
    }
    let large = ScoreTime::from(Tick(u64::MAX))
        .scaled(TimeScale::new(u32::MAX, 1).unwrap())
        .unwrap()
        .repeated(u32::MAX)
        .unwrap();
    assert!(large > Tick(u64::MAX).into());
    assert_eq!(
        large.scaled(TimeScale::new(2, 1).unwrap()),
        Err(ModelError::TimeOverflow)
    );
    assert_eq!(
        Tempo::from_milli_bpm(1)
            .unwrap()
            .device_frame(large, u32::MAX),
        Err(ModelError::TimeOverflow)
    );
}

#[test]
fn independent_placements_of_same_source_have_distinct_occurrences() {
    let mut ids = IdGenerator::default();
    let mut fixture = chopped_drums(&mut ids);
    let root = fixture
        .project
        .sequences
        .get_mut(&fixture.project.main_score)
        .unwrap();
    let first = root.placements[&fixture.placements[0]];
    root.placements.insert(ids.allocate(), first);
    let events: Vec<_> = evaluate(&fixture.project)
        .into_iter()
        .filter(|event| event.event == fixture.hits[0] && event.at == Tick(0).into())
        .collect();
    assert_eq!(events.len(), 2);
    assert_ne!(events[0].placement_path, events[1].placement_path);
}

#[test]
fn launch_and_stop_remain_operations_and_simultaneous_order_is_explicit() {
    let mut ids = IdGenerator::default();
    let mut project = Project::new(&mut ids, "Operations", Tick(4));
    let source = ids.allocate();
    let mut child = Sequence::new("Child", Tick(1));
    child.events.insert(
        ids.allocate(),
        Event {
            at: Tick(0),
            kind: EventKind::Trigger { velocity: 0.2 },
        },
    );
    project.sequences.insert(source, child);
    project.sequence_order.push(source);
    let launch = ids.allocate();
    let main = project.sequences.get_mut(&project.main_score).unwrap();
    main.placements.insert(
        launch,
        Placement {
            start: PlacementStart::OnLaunch,
            ..placement(source, 0, 1, 0)
        },
    );
    // Deliberately create IDs in the opposite order from semantic priority.
    for kind in [
        EventKind::Trigger { velocity: 1.0 },
        EventKind::Launch { placement: launch },
        EventKind::SetParameter {
            parameter: Parameter::Gain,
            value: 0.5,
        },
        EventKind::Stop { placement: launch },
    ] {
        main.events
            .insert(ids.allocate(), Event { at: Tick(0), kind });
    }
    let events = evaluate(&project);
    assert_eq!(events.len(), 4); // The child is not eagerly played by this prototype.
    assert!(matches!(events[0].kind, EventKind::Stop { .. }));
    assert!(matches!(events[1].kind, EventKind::SetParameter { .. }));
    assert!(matches!(events[2].kind, EventKind::Launch { .. }));
    assert!(matches!(events[3].kind, EventKind::Trigger { .. }));
    assert!(events
        .iter()
        .all(|event| event.source == project.main_score));
}

#[test]
fn route_type_errors_and_dangling_destinations_are_rejected() {
    let mut ids = IdGenerator::default();
    let mut fixture = chopped_drums(&mut ids);
    let (&id, route) = fixture.project.control_routes.iter_mut().next().unwrap();
    route.input = ControlPort::Parameter(Parameter::Gain);
    assert_eq!(
        fixture.project.validate(),
        Err(ModelError::IncompatibleControlPorts(id))
    );
    let missing = ids.allocate();
    let route = fixture.project.control_routes.get_mut(&id).unwrap();
    route.input = ControlPort::Trigger;
    route.instrument = missing;
    assert_eq!(
        fixture.project.validate(),
        Err(ModelError::MissingInstrument(missing))
    );
    fixture
        .project
        .control_routes
        .get_mut(&id)
        .unwrap()
        .instrument = fixture.instrument;
    let missing_mixer = ids.allocate();
    fixture
        .project
        .audio_routes
        .values_mut()
        .next()
        .unwrap()
        .destination = AudioInput::Mixer(missing_mixer);
    assert_eq!(
        fixture.project.validate(),
        Err(ModelError::MissingMixer(missing_mixer))
    );
}

#[test]
fn control_fan_out_is_independent_of_sequence_and_audio_channel_counts() {
    let mut ids = IdGenerator::default();
    let mut fixture = chopped_drums(&mut ids);
    let second = ids.allocate();
    fixture.project.instruments.insert(
        second,
        fixture.project.instruments[&fixture.instrument].clone(),
    );
    fixture.project.control_routes.insert(
        ids.allocate(),
        ControlRoute {
            source: fixture.source,
            output: ControlPort::Trigger,
            instrument: second,
            input: ControlPort::Trigger,
        },
    );
    fixture.project.audio_routes.insert(
        ids.allocate(),
        AudioRoute {
            source: AudioOutput::Instrument(second),
            destination: AudioInput::Master,
        },
    );
    for event in evaluate(&fixture.project) {
        assert_eq!(event.deliveries.len(), 2);
        assert_eq!(event.deliveries[1].instrument, second);
        assert!(event.deliveries[1].audio_paths[0].mixer_channels.is_empty());
    }
}

#[test]
fn recursive_sequence_references_and_audio_feedback_are_rejected() {
    let mut ids = IdGenerator::default();
    let mut fixture = chopped_drums(&mut ids);
    let recursive = ids.allocate();
    fixture
        .project
        .sequences
        .get_mut(&fixture.source)
        .unwrap()
        .placements
        .insert(
            recursive,
            Placement {
                start: PlacementStart::OnLaunch,
                ..placement(fixture.project.main_score, 0, 1, 0)
            },
        );
    assert_eq!(
        fixture.project.evaluate(EvaluationLimits::default()),
        Err(ModelError::SequenceCycle)
    );
    fixture
        .project
        .sequences
        .get_mut(&fixture.source)
        .unwrap()
        .placements
        .remove(&recursive);
    fixture.project.audio_routes.insert(
        ids.allocate(),
        AudioRoute {
            source: AudioOutput::Mixer(fixture.mixer),
            destination: AudioInput::Mixer(fixture.mixer),
        },
    );
    assert_eq!(fixture.project.validate(), Err(ModelError::AudioCycle));
}

#[test]
fn validation_rejects_duplicate_id_ownership_ranges_and_nonfinite_values() {
    let mut ids = IdGenerator::default();
    let fixture = chopped_drums(&mut ids);
    let mut project = fixture.project.clone();
    let event = project.sequences[&fixture.source].events[&fixture.hits[0]];
    project
        .sequences
        .get_mut(&project.main_score)
        .unwrap()
        .events
        .insert(fixture.hits[0], event);
    assert_eq!(
        project.validate(),
        Err(ModelError::DuplicateEvent(fixture.hits[0]))
    );
    let mut project = fixture.project.clone();
    project
        .sequences
        .get_mut(&project.main_score)
        .unwrap()
        .placements
        .get_mut(&fixture.placements[0])
        .unwrap()
        .region
        .end = Tick(5000);
    assert_eq!(project.validate(), Err(ModelError::InvalidRegion));
    let mut project = fixture.project.clone();
    project
        .sequences
        .get_mut(&fixture.source)
        .unwrap()
        .events
        .get_mut(&fixture.hits[0])
        .unwrap()
        .at = Tick(3840);
    assert_eq!(project.validate(), Err(ModelError::OutOfBounds));
    for value in [f32::NAN, f32::INFINITY, -0.1] {
        let mut project = fixture.project.clone();
        project
            .instruments
            .get_mut(&fixture.instrument)
            .unwrap()
            .gain = value;
        assert_eq!(project.validate(), Err(ModelError::InvalidValue));
    }
}

#[test]
fn evaluation_limits_cover_event_explosion_nesting_and_empty_repetition() {
    let mut ids = IdGenerator::default();
    let fixture = chopped_drums(&mut ids);
    for limits in [
        EvaluationLimits {
            max_events: 2,
            ..Default::default()
        },
        EvaluationLimits {
            max_depth: 1,
            ..Default::default()
        },
    ] {
        assert_eq!(
            fixture.project.evaluate(limits),
            Err(ModelError::EvaluationLimit)
        );
    }
    let mut project = Project::new(&mut ids, "Empty repeats", Tick(u64::from(u32::MAX)));
    let source = ids.allocate();
    project
        .sequences
        .insert(source, Sequence::new("Empty", Tick(1)));
    project.sequence_order.push(source);
    project
        .sequences
        .get_mut(&project.main_score)
        .unwrap()
        .placements
        .insert(
            ids.allocate(),
            Placement {
                repeats: NonZeroU32::new(u32::MAX).unwrap(),
                ..placement(source, 0, 1, 0)
            },
        );
    assert_eq!(
        project.evaluate(EvaluationLimits {
            max_steps: 10,
            ..Default::default()
        }),
        Err(ModelError::EvaluationLimit)
    );
}
