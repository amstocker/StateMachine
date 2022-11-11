use std::sync::atomic::Ordering::SeqCst;

use assert_no_alloc::*;
use iced::widget::{button, column, text, Column, Text, Row, Button, Container};
use iced::{Alignment, Element, Sandbox, Settings};
use cpal::{SampleRate, StreamConfig, Stream};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

use crate::sequencer::Node;
use crate::sound::*;
use crate::output::stereo_to_output_frame;

use crate::{
    sequencer::{Sequencer, SequencerParameters, GRID_SIZE, GRID_SIZE_ROOT},
    sound::{SoundBankMeta, MAX_SOUNDS, Sound},
    output::{OutputSample, OutputFormat}
};


#[derive(Debug, Clone, Copy)]
pub enum Message {
    EnableSound(usize),
    PlaySound(usize),
    DisableSound(usize)
}

pub struct Application<S> where S: OutputSample {
    value: usize,
    sound_bank: SoundBankMeta<S>,
    sequencer_params: SequencerParameters,
    output_format: OutputFormat,
    output_stream: Stream
}

impl<S> Application<S> where S: OutputSample {
    pub fn node_view<'a>(&'a self, node: &'a Node) -> Column<'a, Message> {
        let sound_index = node.sound_index.load(SeqCst);
        let sound_meta = self.sound_bank.get_sound_meta(sound_index).unwrap();
        
        let mut column = Column::new();
        column = column.push(Text::new(&sound_meta.name));
    
        let mut button_row = Row::new();
        let enable_button = if node.enabled.load(SeqCst) {
            button("Disable").on_press(Message::DisableSound(sound_index))
        } else {
            button("Enable").on_press(Message::EnableSound(sound_index))
        };
        let play_button = button("Play").on_press(Message::PlaySound(sound_index));
        button_row = button_row.push(enable_button);
        button_row = button_row.push(play_button);
    
        column = column.push(button_row);
        column
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
        sound_bank_meta.add_sound(Sound::from_wav_file("samples/kick.wav", &format)).unwrap();
        sound_bank_meta.add_sound(Sound::from_wav_file("samples/snare.wav", &format)).unwrap();
        sound_bank_meta.add_sound(Sound::from_wav_file("samples/hihat.wav", &format)).unwrap();

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
            value: 0,
            sound_bank: sound_bank_meta,
            sequencer_params: controller,
            output_format: format,
            output_stream: stream
        }
    }

    fn title(&self) -> String {
        String::from("Counter - Iced")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::PlaySound(index) => {
                let node = self.sequencer_params.nodes.get(index).unwrap();
                node.current_frame_index.store(0, SeqCst);
                node.is_playing.store(true, SeqCst);
            },
            Message::EnableSound(index) => {
                let node = self.sequencer_params.nodes.get(index).unwrap();
                node.enabled.store(true, SeqCst);
            },
            Message::DisableSound(index) => {
                let node = self.sequencer_params.nodes.get(index).unwrap();
                node.enabled.store(false, SeqCst);
            } 
        }
    }

    fn view(&self) -> Element<Message> {
        let mut column = Column::new();
        for j in 0..GRID_SIZE_ROOT {
            let mut row = Row::new();
            for i in 0..GRID_SIZE_ROOT {
                let index = j * GRID_SIZE_ROOT + i;
                let node = self.sequencer_params.nodes.get(index).unwrap();
                row = row.push(self.node_view(node));
            }
            column = column.push(row);
        }
        Container::new(column).into()
    }
}