#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self};
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
enum FileNode {
    File {
        name: String,
        path: PathBuf,
    },
    Folder {
        name: String,
        contents: Vec<FileNode>,
    },
}

fn visit_dirs(dir: &Path) -> Vec<FileNode> {
    let mut contents = Vec::new();

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();

            if path.is_dir() {
                let sub_contents = visit_dirs(&path);
                contents.push(FileNode::Folder {
                    name,
                    contents: sub_contents,
                });
            } else {
                contents.push(FileNode::File { name, path });
            }
        }
    }
    contents
}

fn main() -> eframe::Result {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([640.0, 480.0]),
        ..Default::default()
    };
    eframe::run_native(
        "My Text Editor",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::<MyApp>::default())
        }),
    )
}

struct MyApp {
    contents: String,
    viewed_file: i32,
    files_open: [String; 32],
    directory_open: String,
    save_to_file: String,
    directory_layout: Vec<FileNode>,
}

impl Default for MyApp {
    fn default() -> Self {
        let array_placeholder: [String; 32] = std::array::from_fn(|_i| String::new());

        let layout: Vec<FileNode> = vec![];

        Self {
            contents: "".to_string(),
            viewed_file: 0,
            files_open: array_placeholder,
            directory_open: ".".to_string(),
            save_to_file: "Untitled.txt".to_string(),
            directory_layout: layout,
        }
    }
}

impl eframe::App for MyApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Directory file list
        egui::Window::new(self.directory_open.to_string()).show(ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                for file_node in self.directory_layout.clone() {
                    self.render_file_node(ui, &file_node);
                }
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            // Files open
            egui::ScrollArea::horizontal()
                .id_salt(0)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.style_mut().spacing.item_spacing = egui::vec2(0.0, 0.0);
                    ui.style_mut().spacing.button_padding = egui::vec2(8.0, 4.0);

                    ui.style_mut().visuals.widgets.active.corner_radius = egui::CornerRadius::ZERO;
                    ui.style_mut().visuals.widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
                    ui.style_mut().visuals.widgets.inactive.corner_radius =
                        egui::CornerRadius::ZERO;

                    ui.horizontal(|ui| {
                        // Check through all the name slots
                        for i in 0..32 {
                            if !self.files_open[i as usize].is_empty()
                                && ui
                                    .add(egui::Button::new(self.files_open[i as usize].to_string()))
                                    .clicked()
                            {
                                self.viewed_file = i;
                                self.save_to_file =
                                    self.files_open[self.viewed_file as usize].to_string();

                                let path = Path::new(&self.files_open[self.viewed_file as usize]);
                                let display = path.display();

                                let mut file = match fs::File::open(path) {
                                    Err(why) => panic!("couldn't open {}: {}", display, why),
                                    Ok(file) => file,
                                };

                                let mut s = String::new();
                                match file.read_to_string(&mut s) {
                                    Err(why) => panic!("couldn't read {}: {}", display, why),
                                    Ok(_) => self.contents = s,
                                }
                            }
                        }
                    })
                });

            // Save and where to save to
            egui::ScrollArea::vertical().id_salt(1).show(ui, |ui| {
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("Save File")).clicked() {
                        if self.save_to_file != self.files_open[self.viewed_file as usize]
                            && self.viewed_file < 32
                        {
                            self.viewed_file += 1;
                            self.files_open[self.viewed_file as usize] =
                                self.save_to_file.to_string();
                        }

                        let path = Path::new(&self.files_open[self.viewed_file as usize]);
                        let display = path.display();

                        let mut file = match fs::File::create(path) {
                            Err(why) => panic!("couldn't create {}: {}", display, why),
                            Ok(file) => file,
                        };

                        match file.write_all(self.contents.as_bytes()) {
                            Err(why) => panic!("couldn't write to {}: {}", display, why),
                            Ok(_) => println!("successfully wrote to {}", display),
                        }
                    }
                    ui.add(egui::TextEdit::singleline(&mut self.save_to_file));
                });
            });

            // Text editor area
            egui::ScrollArea::vertical().id_salt(2).show(ui, |ui| {
                let available_size = ui.available_size();
                ui.add(egui::TextEdit::multiline(&mut self.contents).min_size(available_size));
            });
        });
    }
}

impl MyApp {
    fn render_file_node(&mut self, ui: &mut egui::Ui, node: &FileNode) {
        match node {
            FileNode::File { name, path } => {
                if ui.add(egui::Button::new(name)).clicked() {
                    self.save_to_file = path.to_string_lossy().to_string();

                    match fs::read_to_string(path) {
                        Ok(text) => {
                            self.contents = text;
                        }
                        Err(why) => {
                            eprintln!("Couldn't open {}: {}", path.display(), why);
                        }
                    }
                }
            }
            FileNode::Folder { name, contents, .. } => {
                ui.collapsing(name, |ui| {
                    for child in contents {
                        self.render_file_node(ui, child);
                    }
                });
            }
        }
    }
}
