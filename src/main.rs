use std::process::id;

use eframe::{
    egui::{self, Color32, Pos2, Stroke},
    epaint::EllipseShape,
};
use egui::{InputOptions, Painter};
use nalgebra::{DMatrix, point};

fn main() {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(Truss::new(cc)))),
    );
}

#[derive(Default)]
struct Truss {
    edges: Vec<Member>,
    points: Vec<Pos2>,
    last_node: Option<usize>,
    mode: Mode,
    force: Vec<Force>,
    input_buf: String,
}
#[derive(Default)]
struct Force {
    p1: usize,
    p2: Pos2,
    mag: u32,
}

#[derive(Default)]
struct Member {
    p1: usize,
    p2: usize,
}
#[derive(Default)]
enum Mode {
    Command,
    #[default]
    Insert,
    TextEdit,
    Edit,
    Solve,
}

impl Truss {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

fn hit_test(points: &[Pos2], pos: Pos2) -> Option<usize> {
    points.iter().position(|p| p.distance(pos) < 8.0)
}

impl eframe::App for Truss {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            match self.mode {
                Mode::Insert => {
                    if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                        if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                            if let Some(idx) = hit_test(&self.points, pos) {
                                println!("HIT{idx}");
                                self.edges.push(Member {
                                    p1: self.last_node.unwrap_or(self.points.len() - 1),
                                    p2: idx,
                                });
                                self.last_node = Some(idx);
                            } else {
                                if self.points.len() >= 1 {
                                    self.edges.push(Member {
                                        p1: self.last_node.unwrap_or(self.points.len() - 1),
                                        p2: self.points.len(),
                                    });
                                    self.last_node = None;
                                }
                                self.points.push(pos);
                            }
                        }
                    }

                    if ctx.input(|i| i.key_pressed(egui::Key::F)) {
                        println!("Force");
                        self.mode = Mode::TextEdit
                    }
                    if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                        if self.points.len() >= 1 {
                            ui.painter().line_segment(
                                [
                                    self.points[self.last_node.unwrap_or(self.points.len() - 1)],
                                    pos,
                                ],
                                Stroke::new(1.0, egui::Color32::WHITE),
                            );
                        }
                    }
                    if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
                        self.mode = Mode::Command;
                    }
                }
                Mode::Command => {
                    if ctx.input(|i| i.key_pressed(egui::Key::I)) {
                        self.mode = Mode::Insert;
                    }

                    if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                        if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                            if let Some(idx) = hit_test(&self.points, pos) {
                                self.last_node = Some(idx);
                            }
                        }
                    }
                }
                Mode::Edit => {}
                Mode::Solve => {}
                Mode::TextEdit => {
                    let pos = ctx
                        .input(|i| i.pointer.hover_pos())
                        .unwrap_or(Pos2 { x: 0., y: 0. });
                    let response = ui.add(egui::TextEdit::singleline(&mut self.input_buf));

                    if !response.has_focus() {
                        response.request_focus();
                    }
                    if self.points.len() >= 1 {
                        ui.painter().line_segment(
                            [
                                self.points[self.last_node.unwrap_or(self.points.len() - 1)],
                                pos,
                            ],
                            Stroke::new(1.0, egui::Color32::WHITE),
                        );
                    }
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        self.force.push(Force {
                            p1: self.last_node.unwrap_or(self.points.len() - 1),
                            p2: pos,
                            mag: self.input_buf.parse().unwrap(),
                        });
                        self.mode = Mode::Insert;
                        println!("{}", self.input_buf);
                    }
                }
            }
            for point in &self.points {
                ui.painter()
                    .circle_stroke(*point, 3.0, Stroke::new(1.0, egui::Color32::WHITE));
            }
            for member in &self.edges {
                ui.painter().line_segment(
                    [self.points[member.p1], self.points[member.p2]],
                    Stroke::new(2.0, Color32::RED),
                );
            }
            egui::TopBottomPanel::bottom("command bar").show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let mod_str = match self.mode {
                        Mode::Edit => "Edit",
                        Mode::TextEdit => "Input",
                        Mode::Command => "Command",
                        Mode::Solve => "Solve",
                        Mode::Insert => "Insert",
                    };
                    ui.label(mod_str);
                })
            })
        });
    }
}
