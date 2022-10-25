#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::{fs::File, thread};
use std::io::BufReader;

use rodio::{Decoder, OutputStream, source::Source};
use eframe::egui;
use crossbeam_channel::{unbounded, Receiver, Sender};

fn main() {
    let (sender, receiver) = unbounded();

    thread::spawn(move || { playback(receiver) });

    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "State Machine",
        options,
        Box::new(|_cc| Box::new(
            State::new(
                sender,
                vec![
                    "samples/kick.wav".into(),
                    "samples/snare.wav".into(),
                    "samples/hihat.wav".into()
                ]
            )
        )),
    );
}

enum Message {
    Play(String),
    LoadSample
}

fn playback(receiver: Receiver<Message>) {
    // TICK = 1 ms (or e.g. 500 ms for 120 bpm)
    //      (collect some data on timing)
    // (1) use priority queue to play sounds with tick = 0
    // (2) queue next sounds based on routing configuration
    // (3) iterate over rest to decrease ticks for all queued sounds.
    // (4) use Instand::now() to sleep thread for time delta until next tick
    //      (or just sleep until next sample scheduled)
    
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    loop {
        if let Ok(Message::Play(path)) = receiver.recv() {
            let file = BufReader::new(File::open(path).unwrap());
            let source = Decoder::new(file).unwrap();
            let duration = source.total_duration();
    
            stream_handle.play_raw(source.convert_samples()).unwrap();
        }
    }   
    
}

struct State {
    sender: Sender<Message>,
    sample_filenames: Vec<String>
}

impl State {
    fn new(sender: Sender<Message>, sample_filenames: Vec<String>) -> Self {
        Self {
            sender: sender,
            sample_filenames: sample_filenames
        }
    }
}

impl eframe::App for State {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Samples:");
            for filename in &self.sample_filenames {
                ui.horizontal(|ui| {
                    ui.label(filename);
                    if ui.button("Play").clicked() {
                        self.sender.send(Message::Play(filename.to_string())).unwrap();
                    }
                });
            }
        });
    }
}
