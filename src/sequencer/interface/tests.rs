use super::*;

fn setup() -> (SequencerInterface, Sequencer) {
    let (_assets, bank) = crate::sound::SoundBank::new(vec![]).unwrap();
    let (controller, engine) = Sequencer::new(bank);
    (SequencerInterface::init(controller), engine)
}

fn clip() -> Clip {
    Clip {
        enabled: true,
        channel_location_end: 100,
        source_scale: 1.0,
        ..Default::default()
    }
}

#[test]
fn full_channels_reject_new_items_without_changing_counts() {
    let (mut ui, _engine) = setup();
    for _ in 0..MAX_CLIPS_PER_CHANNEL {
        ui.add_clip(0, clip()).unwrap();
    }
    assert_eq!(ui.add_clip(0, clip()), Err(EditError::ClipsFull));
    let junction = Junction {
        enabled: true,
        location: 10,
        junction_type: JunctionType::Reflect,
    };
    for _ in 0..MAX_JUNCTIONS_PER_CHANNEL {
        ui.handle_create_junction(0, junction).unwrap();
    }
    assert_eq!(
        ui.handle_create_junction(0, junction),
        Err(EditError::JunctionsFull)
    );
    assert_eq!(
        ui.session.document().tracks()[0].clips().len(),
        MAX_CLIPS_PER_CHANNEL
    );
    assert_eq!(
        ui.session.document().tracks()[0].junctions().len(),
        MAX_JUNCTIONS_PER_CHANNEL
    );
    assert_eq!(
        ui.add_clip(NUM_CHANNELS, clip()),
        Err(EditError::InvalidChannel)
    );
    assert_eq!(
        ui.handle_create_junction(NUM_CHANNELS, junction),
        Err(EditError::InvalidChannel)
    );
    assert_eq!(ui.handle_clip_move(1, 0, 0), Err(EditError::InvalidClip));
    assert_eq!(
        ui.handle_clip_move(0, usize::MAX, 0),
        Err(EditError::InvalidClip)
    );
}

#[test]
fn cross_channel_junction_is_owned_by_drag_source() {
    let (mut ui, mut engine) = setup();
    ui.state = State::CreatingJunction {
        source_channel_index: 0,
        source_channel_location: 100,
    };
    ui.set_mouse_position(MousePosition { x: 0.5, y: 0.6 });
    ui.handle_mouse_input(&MouseButton::Left, &ElementState::Released);
    assert_eq!(ui.session.document().tracks()[0].junctions().len(), 1);
    assert_eq!(ui.session.document().tracks()[2].junctions().len(), 0);
    engine.handle_control_messages();
    let junction = engine.channels[0].junctions[0];
    assert_eq!(junction.location, 100);
    assert!(matches!(
        junction.junction_type,
        JunctionType::Jump {
            destination_channel_index: 2,
            destination_location: 250_000,
            split: true,
        }
    ));
    assert!(!engine.channels[2].junctions[0].enabled);
}

#[test]
fn rejected_edits_leave_ui_and_audio_models_unchanged() {
    let (mut ui, mut engine) = setup();
    ui.add_clip(0, clip()).unwrap();
    engine.handle_control_messages();
    // Fill both bounded queues without processing audio.
    for _ in 0..super::super::RING_BUFFER_CAPACITY + super::super::MAX_PENDING_CONTROLS {
        ui.handle_set_playhead(0, Playhead::default()).unwrap();
    }
    ui.set_mouse_position(MousePosition { x: 0.5, y: 0.5 });
    assert_eq!(ui.handle_clip_move(0, 0, 0), Err(EditError::QueueFull));
    assert_eq!(ui.add_clip(0, clip()), Err(EditError::QueueFull));
    assert_eq!(
        ui.handle_create_junction(0, Junction::default()),
        Err(EditError::QueueFull)
    );
    assert_eq!(
        ui.session.document().tracks()[0].clips()[0].channel_location_start,
        0
    );
    assert_eq!(ui.session.document().tracks()[0].clips().len(), 1);
    assert_eq!(ui.session.document().tracks()[0].junctions().len(), 0);
    assert_eq!(engine.channels[0].clips[0].channel_location_start, 0);
}

#[test]
fn invalid_clip_ranges_and_rates_are_rejected_without_mutation() {
    let (mut ui, _engine) = setup();
    for model in [
        Clip {
            channel_location_start: 200,
            ..clip()
        },
        Clip {
            channel_location_end: DEFAULT_CHANNEL_LENGTH + 1,
            ..clip()
        },
        Clip {
            source_scale: f32::NAN,
            ..clip()
        },
        Clip {
            source_scale: 0.0,
            ..clip()
        },
    ] {
        assert!(ui.add_clip(0, model).is_err());
    }
    assert_eq!(ui.session.document().tracks()[0].clips().len(), 0);
}

#[test]
fn hit_testing_follows_document_edits_without_cached_geometry() {
    let (mut ui, mut engine) = setup();
    ui.add_clip(
        0,
        Clip {
            channel_location_end: 100_000,
            ..clip()
        },
    )
    .unwrap();
    ui.set_mouse_position(MousePosition { x: 0.1, y: 0.15 });
    assert!(matches!(
        ui.get_potential_action(),
        Action::Channel {
            channel_action: ChannelAction::GrabClip { clip_index: 0 },
            ..
        }
    ));
    ui.session.move_clip(0, 0, 250_000).unwrap();
    assert!(matches!(
        ui.get_potential_action(),
        Action::Channel {
            channel_action: ChannelAction::SetPlayhead,
            ..
        }
    ));
    ui.set_mouse_position(MousePosition { x: 0.6, y: 0.15 });
    assert!(matches!(
        ui.get_potential_action(),
        Action::Channel {
            channel_action: ChannelAction::GrabClip { clip_index: 0 },
            ..
        }
    ));
    engine.handle_control_messages();
    assert_eq!(engine.channels[0].clips[0].channel_location_start, 250_000);
}
