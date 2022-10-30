use std::sync::Arc;

use crossbeam_channel::Sender;
use eframe::egui;

use crate::app::*;
use crate::playback::PlaybackControlMessage;


pub struct UI {
    app_state: Arc<State>,
    playback_control: Sender<PlaybackControlMessage>
}

impl UI {
    pub fn new(app_state: Arc<State>, playback_control: Sender<PlaybackControlMessage>) -> Self {
        Self {
            app_state,
            playback_control
        }
    }

    pub fn run(self) {
        let options = eframe::NativeOptions {
            // Hide the OS-specific "chrome" around the window:
            decorated: false,
            // To have rounded corners we need transparency:
            //transparent: true,
            min_window_size: Some(egui::vec2(320.0, 100.0)),
            ..Default::default()
        };
        eframe::run_native(
            "State Machine",
            options,
            Box::new(|_cc| Box::new(self)),
        );
    }
}

impl eframe::App for UI {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        use PlaybackControlMessage::*;

        egui::CentralPanel::default().show(ctx, |ui| {
            use eframe::egui::*;

            //let text_color = ctx.style().visuals.text_color();
            let height = 28.0;
            let rect = ui.max_rect();

            // Interact with the title bar (drag to move window):
            let title_bar_rect = {
                let mut rect = rect;
                rect.max.y = rect.min.y + height;
                rect
            };
            let title_bar_response =
                ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click());
            if title_bar_response.is_pointer_button_down_on() {
                frame.drag_window();
            }

            ui.heading("Samples");
            for (id, sound) in self.app_state.sounds.read().iter() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("({})", id));
                        ui.monospace(&sound.name);
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Trigger").clicked() {
                            self.playback_control.send(Play(*id)).unwrap();
                            self.playback_control.send(Pause).unwrap();
                        }
                        if ui.button("Play").clicked() {
                            self.playback_control.send(Play(*id)).unwrap();
                        }
                        if ui.button("Pause").clicked() {
                            self.playback_control.send(Pause).unwrap();
                        }
                    });
                });
            }

            ui.group(|ui| {
                if ui.button("Add Sample").clicked() {
                    if let Some(filename) = rfd::FileDialog::new().pick_file() {
                        App::add_sound_to_state(self.app_state.clone(), filename.display().to_string());
                    }
                }
            });
            
        });
    }
}