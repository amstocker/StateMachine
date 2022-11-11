use std::sync::atomic::Ordering::SeqCst;

use assert_no_alloc::*;
use iced::widget::{button, column, text, Column, Text, Row, Button, Container, Space};
use iced::{Alignment, Element, Sandbox, Settings, Length, alignment::Vertical};
use cpal::{SampleRate, StreamConfig, Stream};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::sequencer::Node;
use crate::sound::*;
use crate::output::stereo_to_output_frame;
use crate::fonts::*;

use crate::{
    sequencer::{Sequencer, SequencerParameters, SequencerControlMessage, GRID_SIZE, GRID_SIZE_ROOT},
    sound::{SoundBankMeta, MAX_SOUNDS, Sound},
    output::{OutputSample, OutputFormat}
};


#[derive(Debug, Clone, Copy)]
pub enum Message {
    Sequencer(SequencerControlMessage)
}

pub struct Application<S> where S: OutputSample {
    sound_bank: SoundBankMeta<S>,
    sequencer_params: SequencerParameters,
    output_format: OutputFormat,
    output_stream: Stream
}

impl<S> Application<S> where S: OutputSample {
    pub fn node_view<'a>(&'a self, index: usize, node: &'a Node) -> Element<'a, Message> {
        use SequencerControlMessage::*;

        let sound_index = node.sound_index.load(SeqCst);
        let sound_meta = self.sound_bank.get_sound_meta(sound_index).unwrap();
        
        let mut column = Column::new();
        let mut sound_info = Row::new();
        sound_info = sound_info.push(
            Text::new(&sound_meta.name)
                .font(JETBRAINS_MONO_BOLD)
        );
        sound_info = sound_info.push(
            Space::with_width(Length::Fill)
        );
        sound_info = sound_info.push(
            Button::new(
                Text::new("-")
                    .font(JETBRAINS_MONO)
            ).on_press(Message::Sequencer(DecrSoundIndex(index)))
        );
        sound_info = sound_info.push(
            Button::new(
                Text::new("+")
                    .font(JETBRAINS_MONO)
            ).on_press(Message::Sequencer(IncrSoundIndex(index)))
        );
        sound_info = sound_info.push(
            Space::with_width(Length::Units(3))
        );
        sound_info = sound_info.push(
            Button::new(
                Text::new("Play")
                    .font(JETBRAINS_MONO)
            ).on_press(Message::Sequencer(PlaySound(index)))
        );
        column = column.push(sound_info);
        column = column.push(Space::with_height(Length::Units(3)));
    
        let mut enable = Row::new();
        enable = enable.push(Space::with_width(Length::Fill));
        enable = enable.push(
            if node.enabled.load(SeqCst) {
                Button::new(
                    Text::new("Disable")
                        .font(JETBRAINS_MONO)
                ).on_press(Message::Sequencer(DisableSound(index)))
            } else {
                Button::new(
                    Text::new("Enable")
                        .font(JETBRAINS_MONO)
                ).on_press(Message::Sequencer(EnableSound(index)))
            }
        );
        column = column.push(enable);

        Container::new(column)
            .width(Length::Units(150))
            .padding(5)
            .into()
    }
}

impl<S> Sandbox for Application<S> where S: 'static + OutputSample {
    type Message = Message;

    fn new() -> Self {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();

        let supported_config = device.default_output_config().unwrap();
        let sample_format = supported_config.sample_format();
        let config: StreamConfig = supported_config.into();

        let num_channels = config.channels as usize;
        let SampleRate(sample_rate) = config.sample_rate;
        let format = OutputFormat {
            channels: num_channels,
            sample_rate: sample_rate as f32,
            sample_format
        };

        let (mut sound_bank_meta, sound_bank) = SoundBank::new();
    
        // Pre-load with some drum sounds...
        sound_bank_meta.add_sound(Sound::from_wav_file("assets/samples/kick.wav", &format)).unwrap();
        sound_bank_meta.add_sound(Sound::from_wav_file("assets/samples/snare.wav", &format)).unwrap();
        sound_bank_meta.add_sound(Sound::from_wav_file("assets/samples/hihat.wav", &format)).unwrap();

        let (controller, mut sequencer) = Sequencer::new(sound_bank);

        let stream = device.build_output_stream(
            &config,
            move |data: &mut [S], _| {
                assert_no_alloc(|| {
                    for out_frame in data.chunks_mut(format.channels) {
                        stereo_to_output_frame(out_frame, sequencer.next_frame(), num_channels, (0, 1));
                    }
                });
            },
            move |err| {
                println!{"{}", err};
            },
        ).unwrap();
        stream.play().unwrap();

        Self { 
            sound_bank: sound_bank_meta,
            sequencer_params: controller,
            output_format: format,
            output_stream: stream
        }
    }

    fn title(&self) -> String {
        String::from("State Machine")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::Sequencer(control) => self.sequencer_params.handle_sequencer_control(control)
        }
    }

    fn view(&self) -> Element<Message> {
        let mut column = Column::new();
        for j in 0..GRID_SIZE_ROOT {
            let mut row = Row::new();
            for i in 0..GRID_SIZE_ROOT {
                let index = j * GRID_SIZE_ROOT + i;
                let node = self.sequencer_params.nodes.get(index).unwrap();
                row = row.push(self.node_view(index, node));
            }
            column = column.push(row);
        }
        Container::new(column).into()
    }
}