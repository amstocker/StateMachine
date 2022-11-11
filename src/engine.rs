use cpal::{Stream, SampleFormat};
use cpal::traits::{DeviceTrait, StreamTrait};
use assert_no_alloc::*;
use dasp::Sample;

use crate::{
    output::{OutputConfig, StereoFrameGenerator, OutputSample},
    application::Float
};

#[cfg(debug_assertions)]
#[global_allocator]
static A: AllocDisabler = AllocDisabler;


pub struct Engine {
    output_config: OutputConfig,
    output_stream: Option<Stream>
}

impl Engine {
    pub fn new(output_config: OutputConfig) -> Self {
        Self {
            output_config,
            output_stream: None
        }
    }

    pub fn run<T: 'static + StereoFrameGenerator<Float> + Send>(&mut self, generator: T) {
        use SampleFormat::*;
        let stream = match self.output_config.sample_format {
            I16 => {
                self.build_stream::<i16, _>(generator)
            }
            U16 => {
                self.build_stream::<u16, _>(generator)
            },
            F32 => {
                self.build_stream::<f32, _>(generator)
            }
        };
        stream.play().unwrap();

        self.output_stream = Some(stream);
    }

    pub fn build_stream<S: OutputSample, T: 'static + StereoFrameGenerator<Float> + Send>(&self, mut generator: T) -> Stream {
        let channels = self.output_config.channels;
        let output_channels = self.output_config.output_channels;

        self.output_config.device.build_output_stream(
            &self.output_config.stream_config,
            move |data: &mut [S], _| {
                assert_no_alloc(|| {
                    for out_frame in data.chunks_mut(channels) {
                        let frame = generator.next_frame();
                        for (i, out_sample) in out_frame.iter_mut().enumerate() {
                            if i == output_channels.0 % channels {
                                *out_sample = frame.left().to_sample::<S>();
                            } else if i == output_channels.1 % channels {
                                *out_sample = frame.right().to_sample::<S>();
                            } else {
                                *out_sample = S::EQUILIBRIUM;
                            }
                        } 
                    }
                });
            },
            move |err| {
                println!{"{}", err};
            },
        ).unwrap()
    }
}
