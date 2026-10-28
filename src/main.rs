use state_machine::app::App;
use state_machine::config::AppConfig;
use state_machine::sound::Sound;
use state_machine::ui::Application;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut config = AppConfig::default();

    let files = [
        "assets/samples/kick.wav",
        "assets/samples/snare.wav",
        "assets/samples/hihat.wav",
        "assets/samples/flute.wav",
    ];
    config.sounds = files
        .into_iter()
        .map(|path| {
            Sound::from_wav_file(path, config.output.sample_rate)
                .map_err(|error| format!("{path}: {error}"))
        })
        .collect::<Result<_, _>>()?;

    App::run(config);
    Ok(())
}
