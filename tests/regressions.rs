use std::io::Cursor;

use state_machine::sequencer::*;
use state_machine::sound::*;

fn constant_sound(length: usize) -> Sound<f32> {
    Sound {
        metadata: SoundMetadata {
            name: "test".into(),
            length,
        },
        data: vec![StereoFrame(0.25, 0.25); length].into_boxed_slice(),
    }
}

fn clip() -> Clip {
    Clip {
        enabled: true,
        source_index: 0,
        channel_location_start: 0,
        channel_location_end: DEFAULT_CHANNEL_LENGTH,
        source_scale: 1.0,
        source_shift: 0,
    }
}

#[test]
fn stalled_ui_does_not_stop_audio() {
    let (_assets, bank) =
        SoundBank::new(vec![constant_sound(DEFAULT_CHANNEL_LENGTH as usize)]).unwrap();
    let (mut controller, mut sequencer) = Sequencer::new(bank);
    let index = ChannelItemIndex {
        channel_index: 0,
        item_index: 0,
    };
    controller
        .send(SequencerControlMessage::SyncClip {
            index,
            clip: clip(),
        })
        .unwrap();
    controller
        .send(SequencerControlMessage::SyncPlayhead {
            index,
            playhead: Playhead {
                state: PlayheadState::Playing,
                ..Default::default()
            },
        })
        .unwrap();
    // More than ten seconds at 48 kHz, without consuming any status updates.
    assert_no_alloc::assert_no_alloc(|| {
        for _ in 0..480_000 {
            let frame = sequencer.next_frame();
            assert_eq!((frame.left(), frame.right()), (0.25, 0.25));
        }
    });
    // Once the UI resumes, fresh status must become available again.
    while controller.event_receiver.pop().is_ok() {}
    for _ in 0..256 {
        sequencer.next_frame();
    }
    let SequencerEvent::Tick(summary) = controller.event_receiver.pop().unwrap();
    assert!(summary.total_frames_processed >= 480_000);
    assert!(summary.playheads[0].location >= 479_999);
}

#[test]
fn resampling_handles_identity_upsampling_empty_and_short_sources() {
    let resample = |source: Vec<f32>, ratio| {
        LinearInterpolator::new(source.into_iter(), ratio)
            .unwrap()
            .collect::<Vec<_>>()
    };
    assert_eq!(resample(vec![0.0, 0.25, -0.5], 1.0), [0.0, 0.25, -0.5]);
    assert_eq!(resample(vec![0.0, 0.5], 2.0), [0.0, 0.25, 0.5, 0.25]);
    assert_eq!(resample(vec![], 0.5), []);
    assert_eq!(resample(vec![0.5], 1.0), [0.5]);
    assert_eq!(resample(vec![0.5], 0.25), [0.5]);
    assert_eq!(resample(vec![0.5], 2.0), [0.5, 0.25]);
    assert_eq!(resample(vec![0.0, 0.25, 0.5], 0.5), [0.0, 0.5]);
    let mut empty = LinearInterpolator::new(std::iter::empty::<f32>(), 1.0).unwrap();
    assert_eq!(empty.next(), None);
    assert_eq!(empty.next(), None);
}

#[test]
fn resampling_rejects_invalid_ratios() {
    for ratio in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert!(LinearInterpolator::new([0.5].into_iter(), ratio).is_err());
    }
}

#[test]
fn resampling_preserves_duration_at_common_sample_rates() {
    for (source_rate, target_rate) in [(44_100, 48_000), (48_000, 44_100), (96_000, 44_100)] {
        let result = LinearInterpolator::new(
            vec![0.25f32; source_rate].into_iter(),
            target_rate as f64 / source_rate as f64,
        )
        .unwrap()
        .collect::<Vec<_>>();
        assert_eq!(result.len(), target_rate, "{source_rate} -> {target_rate}");
        assert!(result
            .iter()
            .all(|sample| sample.is_finite() && (0.0..=0.25).contains(sample)));
    }
}

#[test]
fn sound_bank_reports_capacity_and_bounds_without_losing_existing_sounds() {
    let (mut controller, mut bank) = SoundBank::new(vec![]).unwrap();
    for index in 0..MAX_SOUNDS {
        assert_eq!(controller.add_sound(constant_sound(4)), Ok(index));
    }
    assert_eq!(
        controller.add_sound(constant_sound(4)),
        Err(SoundBankError::Full)
    );
    assert_no_alloc::assert_no_alloc(|| bank.update());
    assert_eq!(
        bank.get_frame(SoundBankIndex {
            source_index: MAX_SOUNDS - 1,
            frame_index: 0
        })
        .unwrap()
        .left(),
        0.25
    );
    assert!(bank
        .get_frame(SoundBankIndex {
            source_index: MAX_SOUNDS,
            frame_index: 0
        })
        .is_none());
    assert!(bank
        .get_frame(SoundBankIndex {
            source_index: 0,
            frame_index: 4
        })
        .is_none());
    assert!(matches!(
        SoundBank::new((0..=MAX_SOUNDS).map(|_| constant_sound(1)).collect()),
        Err(SoundBankError::Full)
    ));
}

#[test]
fn disconnected_sound_upload_does_not_publish_metadata() {
    let (mut controller, bank) = SoundBank::new(vec![]).unwrap();
    drop(bank);
    assert_eq!(
        controller.add_sound(constant_sound(4)),
        Err(SoundBankError::EngineDisconnected)
    );
    assert!(controller.get(0).is_none());
}

fn wav_bytes(channels: u16, bits_per_sample: u16, sample_format: hound::SampleFormat) -> Vec<u8> {
    let mut bytes = Cursor::new(Vec::new());
    {
        let mut writer = hound::WavWriter::new(
            &mut bytes,
            hound::WavSpec {
                channels,
                sample_rate: 48_000,
                bits_per_sample,
                sample_format,
            },
        )
        .unwrap();
        for sample in [0, 8192, -16384, 4096] {
            match sample_format {
                hound::SampleFormat::Int => writer.write_sample(sample).unwrap(),
                hound::SampleFormat::Float => writer.write_sample(sample as f32 / 32768.0).unwrap(),
            }
        }
        writer.finalize().unwrap();
    }
    bytes.into_inner()
}

fn decode(bytes: Vec<u8>, target_rate: usize) -> Result<Sound<f32>, SoundLoadError> {
    Sound::from_wav_reader(
        hound::WavReader::new(Cursor::new(bytes))?,
        "fixture",
        target_rate,
    )
}

#[test]
fn mono_pcm_and_float_import_preserve_amplitude_and_duplicate_channels() {
    for (bits, format) in [
        (16, hound::SampleFormat::Int),
        (32, hound::SampleFormat::Float),
    ] {
        let sound = decode(wav_bytes(1, bits, format), 48_000).unwrap();
        assert_eq!(sound.metadata.length, 4);
        for (frame, expected) in sound.data.iter().zip([0.0, 0.25, -0.5, 0.125]) {
            assert_eq!((frame.left(), frame.right()), (expected, expected));
        }
    }
}

#[test]
fn unsupported_wav_formats_are_explicit_errors() {
    for (channels, bits) in [(2, 16), (1, 24), (1, 32)] {
        assert!(matches!(
            decode(wav_bytes(channels, bits, hound::SampleFormat::Int), 48_000),
            Err(SoundLoadError::UnsupportedFormat(_))
        ));
    }
}

#[test]
fn malformed_truncated_and_invalid_rate_inputs_are_errors() {
    assert!(decode(b"not a WAV".to_vec(), 48_000).is_err());
    let bytes = wav_bytes(1, 16, hound::SampleFormat::Int);
    assert!(matches!(
        decode(bytes.clone(), 0),
        Err(SoundLoadError::InvalidSampleRate)
    ));
    let mut truncated = bytes;
    truncated.truncate(truncated.len() - 3);
    assert!(decode(truncated, 24_000).is_err());
    let missing =
        std::env::temp_dir().join(format!("state-machine-missing-{}.wav", std::process::id()));
    assert!(Sound::<f32>::from_wav_file(missing, 48_000).is_err());
}

#[test]
fn non_finite_float_audio_is_rejected() {
    let mut bytes = wav_bytes(1, 32, hound::SampleFormat::Float);
    let last_sample = bytes.len() - 4;
    bytes[last_sample..].copy_from_slice(&f32::NAN.to_le_bytes());
    assert!(matches!(
        decode(bytes, 48_000),
        Err(SoundLoadError::NonFiniteSample)
    ));
}

#[test]
fn bundled_demo_samples_still_load() {
    for name in ["kick", "snare", "hihat", "flute"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("assets/samples/{name}.wav"));
        let sound = Sound::<f32>::from_wav_file(path, 48_000).unwrap();
        assert_eq!(sound.metadata.name, name);
        assert_eq!(sound.metadata.length, sound.data.len());
        assert!(!sound.data.is_empty());
    }
}

#[test]
fn downsampling_advances_through_every_skipped_source_frame() {
    let source = (0..8).map(|i| i as f32 / 10.0);
    let result: Vec<_> = LinearInterpolator::new(source, 0.5).unwrap().collect();
    assert_eq!(result, [0.0, 0.2, 0.4, 0.6]);
}
