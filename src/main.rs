use eframe::{
    egui::{self, Color32, Pos2, Stroke},
    epaint::EllipseShape,
};
use egui::Painter;
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
}
#[derive(Default)]
struct Member {
    p1: usize,
    p2: usize,
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
            if let Some(pos2, keydown) = ctx.input(|i| i.key_pressed(egui::Key::Space)) {
                self.edges.push(Member {
                    p1: self.points.len(),
                    p2: self.points.len(),
                });
                self.edges.push(Member {
                    p1: self.points.len(),
                    p2: self.points.len() + 1,
                });

                self.points.push(pos);
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
        });
    }
}
