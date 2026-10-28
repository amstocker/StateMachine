use super::*;

fn setup() -> (SequencerController, Sequencer) {
    let (_assets, bank) = SoundBank::new(vec![]).unwrap();
    Sequencer::new(bank)
}

fn seek(location: u64) -> SequencerControlMessage {
    SequencerControlMessage::SyncPlayhead {
        index: ChannelItemIndex {
            channel_index: 0,
            item_index: 0,
        },
        playhead: Playhead {
            location,
            ..Default::default()
        },
    }
}

#[test]
fn saturated_controls_retry_in_order_and_reject_excess() {
    let (mut controller, mut engine) = setup();
    for i in 0..RING_BUFFER_CAPACITY + MAX_PENDING_CONTROLS {
        controller.send(seek(i as u64)).unwrap();
    }
    assert_eq!(controller.send(seek(99)), Err(EditError::QueueFull));
    assert_no_alloc::assert_no_alloc(|| engine.handle_control_messages());
    assert_eq!(
        engine.channels[0].playhead.location,
        (RING_BUFFER_CAPACITY - 1) as u64
    );
    controller.flush_pending().unwrap();
    assert_no_alloc::assert_no_alloc(|| engine.handle_control_messages());
    assert_eq!(
        engine.channels[0].playhead.location,
        (RING_BUFFER_CAPACITY + MAX_PENDING_CONTROLS - 1) as u64
    );
    controller.send(seek(42)).unwrap();
    engine.handle_control_messages();
    assert_eq!(engine.channels[0].playhead.location, 42);
}

#[test]
fn invalid_control_messages_are_rejected_before_publication_and_at_engine_boundary() {
    let (mut controller, mut engine) = setup();
    let index = ChannelItemIndex {
        channel_index: 0,
        item_index: 0,
    };
    let messages = [
        SequencerControlMessage::SyncPlayhead {
            index: ChannelItemIndex {
                channel_index: NUM_CHANNELS,
                ..index
            },
            playhead: Playhead::default(),
        },
        seek(u64::MAX),
        SequencerControlMessage::SyncClip {
            index: ChannelItemIndex {
                item_index: MAX_CLIPS_PER_CHANNEL,
                ..index
            },
            clip: Clip::default(),
        },
        SequencerControlMessage::SyncClip {
            index,
            clip: Clip {
                enabled: true,
                source_index: crate::sound::MAX_SOUNDS,
                ..Default::default()
            },
        },
        SequencerControlMessage::SyncJunction {
            index: ChannelItemIndex {
                item_index: MAX_JUNCTIONS_PER_CHANNEL,
                ..index
            },
            junction: Junction::default(),
        },
        SequencerControlMessage::SyncJunction {
            index,
            junction: Junction {
                enabled: true,
                location: 0,
                junction_type: JunctionType::Jump {
                    destination_channel_index: NUM_CHANNELS,
                    destination_location: 0,
                    split: true,
                },
            },
        },
    ];
    for message in messages {
        assert!(controller.send(message).is_err());
        assert!(engine.control_message_receiver.pop().is_err());
        // Exercise the callback's defensive check too.
        controller.control_message_sender.push(message).unwrap();
        assert_no_alloc::assert_no_alloc(|| engine.handle_control_messages());
    }
    assert_eq!(engine.channels[0].playhead.location, 0);
    assert!(engine.channels[0].clips.iter().all(|clip| !clip.enabled));
    assert!(engine.channels[0]
        .junctions
        .iter()
        .all(|junction| !junction.enabled));
}

#[test]
fn dropped_engine_rejects_controls() {
    let (mut controller, engine) = setup();
    drop(engine);
    assert_eq!(
        controller.send(seek(42)),
        Err(EditError::EngineDisconnected)
    );
}
