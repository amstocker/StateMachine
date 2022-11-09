use std::sync::atomic::Ordering::SeqCst;

use eframe::egui;

use crate::{sequencer::{SequencerParameters, GRID_SIZE, GRID_SIZE_ROOT}, sound::{SoundBankMeta, MAX_SOUNDS}, output::OutputSample};


pub struct UI<S> where S: OutputSample {
    sound_bank: SoundBankMeta<S>,
    controller: SequencerParameters
}

impl<S> UI<S> where S: OutputSample + 'static {
    pub fn new(sound_bank: SoundBankMeta<S>, controller: SequencerParameters) -> Self {
        Self {
            sound_bank,
            controller
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

impl<S> eframe::App for UI<S> where S: OutputSample {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
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
                ui.interact(title_bar_rect, Id::new("state_machine"), Sense::click());
            if title_bar_response.is_pointer_button_down_on() {
                frame.drag_window();
            }

            ui.heading("Grid");
            for j in 0..GRID_SIZE_ROOT {
                ui.horizontal(|ui| {
                    for i in 0..GRID_SIZE_ROOT {
                        let index = j * GRID_SIZE_ROOT + i;
                        let node = self.controller.nodes.get(index).unwrap();
                        let sound_index = node.sound_index.load(SeqCst);
                        let sound_meta = self.sound_bank.get_sound_meta(sound_index).unwrap();
                        let progress = node.current_frame_index.load(SeqCst) as f32 / sound_meta.length as f32;
                        ui.group(|ui| {
                            ui.vertical(|ui| {
                                ui.label(format!("Sound: {}", sound_meta.name));
                                ui.horizontal(|ui| {
                                    if ui.button("Prev").clicked() && sound_index > 0 {
                                        node.sound_index.fetch_sub(1, SeqCst);
                                    }
                                    if ui.button("Next").clicked() && sound_index < MAX_SOUNDS {
                                        node.sound_index.fetch_add(1, SeqCst);
                                    }
                                });
                                //ui.label(format!("Progress: {:.2}", progress));
                                ui.horizontal(|ui| {
                                    if node.enabled.load(SeqCst) {
                                        if ui.button("Disable").clicked() {
                                            node.enabled.store(false, SeqCst);
                                        }
                                    } else {
                                        if ui.button("Enable").clicked() {
                                            node.enabled.store(true, SeqCst);
                                        }
                                    }
                                    if ui.button("Play").clicked() {
                                        node.current_frame_index.store(0, SeqCst);
                                        node.is_playing.store(true, SeqCst);
                                    }
                                });
                            });
                        });
                    }
                });
            }

            // for (id, sound) in self.app_state.sounds.read().iter() {
            //     ui.group(|ui| {
            //         ui.horizontal(|ui| {
            //             ui.label(format!("({})", id));
            //             ui.monospace(&sound.name);
            //         });
            //         ui.separator();
            //         ui.horizontal(|ui| {
            //             if ui.button("Trigger").clicked() {
            //             }
            //             if ui.button("Play").clicked() {
            //             }
            //             if ui.button("Pause").clicked() {
            //             }
            //         });
            //     });
            // }

            // ui.group(|ui| {
            //     if ui.button("Add Sample").clicked() {
            //         if let Some(filename) = rfd::FileDialog::new().pick_file() {
            //             App::add_sound_to_state(self.app_state.clone(), filename.display().to_string());
            //         }
            //     }
            // });
            
        });

        // necessary because egui cannot tell if atomic variables have been updated
        // TODO: better way?
        ctx.request_repaint();
    }
}