use eframe::egui;

use crate::app::*;
use crate::sound::*;


// how to properly abstract UI?

impl eframe::App for App {
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
                ui.interact(title_bar_rect, Id::new("title_bar"), Sense::click());
            if title_bar_response.is_pointer_button_down_on() {
                frame.drag_window();
            }

            ui.heading("Samples");
            for (id, sound) in self.shared.sounds.read().iter() {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("({})", id));
                        ui.monospace(&sound.filename);
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        ui.label("Triggers:");
                        ui.vertical(|ui| {
                            if let Some(triggers) = self.trigger_map.get(&id) {
                                for trigger in triggers {
                                    ui.horizontal(|ui| {
                                        // TODO: change trigger and delay to editable fields
                                        ui.label("To:");
                                        ui.label(format!("({})", trigger.target));
                                        ui.label("Delay:");
                                        match trigger.delay {
                                            Delay::Milliseconds(ms) => {
                                                ui.label(format!("{} ms", ms));
                                            },
                                            Delay::Tempo { count, division, swing } => {
    
                                            }
                                        }
                                    });
                                }
                            }
                            if ui.button("Add").clicked() {
                                //self.add_connection(id, connection)
                            }
                        });
                    });
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("Play").clicked() {
                            self.sender.send(Message::PlaySound(sound.id)).unwrap();
                        }
                        if ui.button("Remove").clicked() {
                            self.queue_for_remove.push(*id);
                        }
                        
                    });
                });
            }
            while !self.queue_for_remove.is_empty() {
                if let Some(id) = self.queue_for_remove.pop() {
                    self.shared.sounds.write().remove(&id);
                }
            }

            ui.group(|ui| {
                if ui.button("Add Sample").clicked() {
                    if let Some(filename) = rfd::FileDialog::new().pick_file() {
                        self.add_sound(filename.display().to_string());
                    }
                }
            });
            
        });
    }
}