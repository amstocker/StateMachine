mod sound;
mod sequencer;
mod ui;
mod config;

use ui::EventSender;

use crate::ui::{run, Application};
use crate::config::Config;
use crate::sound::Sound;


enum Event {

}

struct Instrument {

}

impl Instrument {
    fn new(config: Config) -> Self {
        Self {

        }
    }
}

impl Application for Instrument {
    type Event = Event;

    fn init(&mut self, event_sender: EventSender<Self::Event>) {
        todo!()
    }

    fn update(&mut self) {
        todo!()
    }

    fn draw(&self) {
        todo!()
    }

    fn handle(&mut self, event: Self::Event) {
        todo!()
    }
}


fn main() {
    let mut config = Config::default();

    let files = [
        "assets/samples/kick.wav",
        "assets/samples/snare.wav",
        "assets/samples/hihat.wav"
    ];
    for path in files {
        config.sounds.push(Sound::from_wav_file(path, &config.output));
    }

    let app = Instrument::new(config);

    run(app);
}