use egui::{Key, Pos2, debug_text::print};
#[derive(Debug, Default)]
pub enum MessageType {
    #[default]
    Forcemsg,
    Command,
}
#[derive(Default, Debug)]
pub struct Truss {
    pub edges: Vec<Member>,
    pub points: Vec<Pos2>,
    pub connections: Vec<ConnectionData>,
    pub last_node: Option<usize>,
    pub mode: Mode,
    pub messagetyp: MessageType,
    pub force: Vec<Force>,
    pub input_buf: String,
}
#[derive(Default, Debug)]
pub struct Force {
    pub p1: usize,
    pub p2: Pos2,
    pub mag: f32,
}

#[derive(Default, Debug)]
pub struct Member {
    pub p1: usize,
    pub p2: usize,
}
#[derive(Default, Debug)]
pub enum Mode {
    Command,
    #[default]
    Insert,
    TextEdit,
    Edit,
    Solve,
}
#[derive(Debug)]
pub enum Connection {
    Pin,
    Roller,
    Joint,
}
#[derive(Debug)]
pub enum ConnectionData {
    Roller(usize),
    Pin(usize),
}

use std::{f32::consts::PI, num::NonZeroUsize, process::id};

use faer::{
    dyn_stack::{MemStack, mem},
    linalg::solvers::PartialPivLu,
    linalg::solvers::SolveCore,
    linalg::{cholesky::llt::solve::solve_in_place, solvers},
    mat::Mat,
    prelude::*,
};

pub fn calculate_member_stress(truss: &mut Truss) -> Mat<f32> {
    let size = 2 * truss.points.len();
    let mut reactions = 0;
    let mut matrix = Mat::<f32>::zeros(size, size);

    let mut zeros = Mat::<f32>::zeros(size, 1);
    print!("matrix, {:?}", matrix);
    for node in &truss.points {
        println!("Node: {:?}", node);
    }
    for member in &truss.edges {
        println!("Member {:?}", member);
    }
    // puts reactions at the node id for x and the node id +1 for y eqautions
    // the second half(left to right) of the matrix should be reactions, whereas the first half
    // should be the forces in all the members.
    let mut halfsize = truss.edges.len();
    println!("halfsize{}", halfsize);

    for connection in &truss.connections {
        halfsize += match connection {
            ConnectionData::Pin(id) => {
                reactions += 2;

                matrix[(id * 2, halfsize)] = 1.;

                println!("put pin at {},{}", id, halfsize);
                matrix[(id * 2 + 1, halfsize + 1)] = 1.;

                println!("put pin at {},{}", id, halfsize + 1);
                2
            }
            ConnectionData::Roller(id) => {
                reactions += 1;
                matrix[(id * 2 + 1, halfsize)] = 1.;

                1
            }
        };
        println!("halfsize count {halfsize}");
    }
    for force in &truss.force {
        print!("force id{}", force.p1);
        matrix[(force.p1, halfsize - 1)];
    }
    if size == truss.edges.len() + reactions {
        println!("Solvable!");
        let mut i = 0;
        for member in &truss.edges {
            let start = truss.points[member.p1];
            let end = truss.points[member.p2];
            let dx = end.x - start.x;
            let dy = end.y - start.y;
            let length = (dx * dx + dy * dy).sqrt();

            let col = i; // member force column

            let row_x_start = 2 * member.p1;
            let row_y_start = 2 * member.p1 + 1;
            let row_x_end = 2 * member.p2;
            let row_y_end = 2 * member.p2 + 1;

            i += 1;
            // Start node
            // println!("placing {row_x_start}, {col}");
            //
            // println!("placing {row_y_start}, {col}");
            //
            // println!("placing {row_x_end}, {col}");
            //
            // println!("placing {row_y_end}, {col}");
            matrix[(row_x_start, col)] = dx / length;
            matrix[(row_y_start, col)] = dy / length;
            // End node (negative)
            matrix[(row_x_end, col)] = -dx / length;
            matrix[(row_y_end, col)] = -dy / length;
        }
        for force in &truss.force {
            let start = truss.points[force.p1];
            let end = force.p2;
            let diff = start - end;
            print!("diff{}", diff);
            let anglex = diff.x / diff.length();
            let angley = diff.y / diff.length();
            println!("angle_x, {}", anglex);

            println!("angle_y, {}", angley);
            let id = force.p1;
            zeros[(id * 2 + 1, 0)] = force.mag * angley;
            zeros[(force.p1 * 2, 0)] = force.mag * anglex;
        }
        println!("forcing{:?}", zeros);
        println!("matrix after placing coefficeints: {:?}", matrix);
        let decomp = PartialPivLu::new(matrix.as_ref());

        // let u = decomp.U();
        //
        // let tol = 1e-9;
        // let mut singular = false;
        //
        // for i in 0..u.nrows().min(u.ncols()) {
        //     if u[(i, i)].abs() < tol {
        //         singular = true;
        //         break;
        //     }
        // // }
        // if singular {
        //     panic!("matrix is singular and cannot be solved")
        // } else {
        // println!("matrix is not singular and is  invertable");
        decomp.solve_in_place_with_conj(faer::Conj::No, zeros.as_mut());
        println!("matrix {:?}", decomp);
        println!("sol? :{:?}", zeros);
        // }
    } else if truss.points.len() + reactions > size {
        println!("overdefined")
    } else if truss.points.len() + reactions < size {
        print!(
            "truss points + reactions{}",
            (truss.points.len() + reactions)
        );
        print!("size{}", size);
        let morereactions = size - reactions;
        println!("need {} more reactions", morereactions);
        println!("reactions: {}", reactions);
    }
    zeros
}

// UI
fn hit_test(points: &[Pos2], pos: Pos2) -> Option<usize> {
    points.iter().position(|p| p.distance(pos) < 8.0)
}

impl eframe::App for Truss {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_mode(ctx);

        egui::CentralPanel::default().show(ctx, |ui| {
            self.draw_scene(ui, ctx);
        });
        self.draw_command_bar(ctx);
    }
}

impl Truss {
    fn handle_mode(&mut self, ctx: &egui::Context) {
        match self.mode {
            Mode::Insert => self.handle_insert(ctx),
            Mode::Command => self.handle_command(ctx),
            Mode::TextEdit => self.handle_text_edit(ctx),
            Mode::Edit => {}
            Mode::Solve => {}
        }
    }

    fn handle_insert(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.key_pressed(egui::Key::Space)) {
            if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                self.handle_insert_click(pos, Connection::Joint);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::P)) {
            if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                self.handle_insert_click(pos, Connection::Pin);
            }
        }

        if ctx.input(|i| i.key_pressed(egui::Key::R)) {
            if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                self.handle_insert_click(pos, Connection::Roller);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F)) {
            self.mode = Mode::TextEdit;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.mode = Mode::Command;
        }
    }

    fn handle_insert_click(&mut self, pos: egui::Pos2, jointtype: Connection) {
        match jointtype {
            Connection::Joint => {
                if let Some(idx) = hit_test(&self.points, pos) {
                    self.edges.push(Member {
                        p1: self.last_node.unwrap_or(self.points.len() - 1),
                        p2: idx,
                    });
                    self.last_node = Some(idx);
                } else {
                    if !self.points.is_empty() {
                        self.edges.push(Member {
                            p1: self.last_node.unwrap_or(self.points.len() - 1),
                            p2: self.points.len(),
                        });
                        self.last_node = None;
                    }
                    self.points.push(pos);
                }
            }
            Connection::Roller => {
                if let Some(idx) = hit_test(&self.points, pos) {
                    self.connections.push(ConnectionData::Roller(idx));
                    self.last_node = Some(idx);
                } else {
                    panic!("Connections need to be at nodes")
                }
            }
            Connection::Pin => {
                if let Some(idx) = hit_test(&self.points, pos) {
                    self.connections.push(ConnectionData::Pin(idx));
                    self.last_node = Some(idx);
                } else {
                    panic!("Connection must be at nodes")
                }
            }
        }
    }

    fn handle_command(&mut self, ctx: &egui::Context) {
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
        if ctx.input(|i| i.key_pressed(Key::Colon)) {
            self.mode = Mode::TextEdit;
            self.messagetyp = MessageType::Command;
        }
    }

    fn handle_text_edit(&mut self, ctx: &egui::Context) {
        use egui::*;
        match &self.messagetyp {
            MessageType::Forcemsg => {
                TopBottomPanel::bottom("nvim_command_bar")
                    .exact_height(28.0)
                    .show(ctx, |ui| {
                        Frame::new().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(":").color(Color32::LIGHT_GREEN).monospace(),
                                );

                                let text_edit = TextEdit::singleline(&mut self.input_buf)
                                    .font(TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .frame(false);

                                let response = ui.add(text_edit);

                                // Auto-focus when entering mode
                                if ui.memory(|m| !m.has_focus(response.id)) {
                                    response.request_focus();
                                }

                                // Enter submits
                                if ui.input(|i| i.key_pressed(Key::Enter)) {
                                    self.submit_force(ctx);
                                    response.highlight();
                                }

                                // Esc cancels
                                if ui.input(|i| i.key_pressed(Key::Escape)) {
                                    self.input_buf.clear();
                                    self.mode = Mode::Insert;
                                }
                            });
                        });
                    });
            }
            MessageType::Command => {
                TopBottomPanel::bottom("nvim_command_bar")
                    .exact_height(28.0)
                    .show(ctx, |ui| {
                        Frame::new().show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(":").color(Color32::LIGHT_GREEN).monospace(),
                                );

                                let text_edit = TextEdit::singleline(&mut self.input_buf)
                                    .font(TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .frame(false);

                                let response = ui.add(text_edit);

                                if ui.memory(|m| !m.has_focus(response.id)) {
                                    response.request_focus();
                                }

                                // Enter submits
                                if ui.input(|i| i.key_pressed(Key::Enter)) {
                                    self.submit_command(ctx);
                                    response.highlight();
                                }

                                // Esc cancels
                                if ui.input(|i| i.key_pressed(Key::Escape)) {
                                    self.input_buf.clear();
                                    self.mode = Mode::Insert;
                                }
                            });
                        });
                    });
            }
        }
    }

    fn submit_force(&mut self, ctx: &egui::Context) {
        if let Ok(mag) = self.input_buf.parse() {
            if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                self.force.push(Force {
                    p1: self.last_node.unwrap_or(self.points.len() - 1),
                    p2: pos,
                    mag,
                });
                println!("Creating force");
            }
        }

        self.input_buf.clear();
        self.mode = Mode::Insert;
    }

    fn submit_command(&mut self, ctx: &egui::Context) {
        let input = self.input_buf.as_str();
        match input {
            "solve" => {
                println!("solving beep boop");
                calculate_member_stress(self);
            }
            _ => {}
        }

        self.input_buf.clear();
        self.mode = Mode::Insert;
    }

    fn draw_scene(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let painter = ui.painter();

        // Preview line
        if let Mode::Insert = self.mode {
            let pos = ctx
                .input(|i| i.pointer.hover_pos())
                .unwrap_or(Pos2::new(0., 0.));
            if !self.points.is_empty() {
                painter.line_segment(
                    [
                        self.points[self.last_node.unwrap_or(self.points.len() - 1)],
                        pos,
                    ],
                    egui::Stroke::new(1.0, egui::Color32::WHITE),
                );
            }
        }

        // Draw points
        for point in &self.points {
            painter.circle_stroke(*point, 3.0, egui::Stroke::new(1.0, egui::Color32::WHITE));
        }
        for connection in &self.connections {
            match connection {
                ConnectionData::Roller(idx) => {
                    painter.circle(
                        self.points[*idx],
                        6.0,
                        egui::Color32::GREEN,
                        egui::Stroke::new(1.0, egui::Color32::GREEN),
                    );
                }
                ConnectionData::Pin(idx) => {
                    painter.circle(
                        self.points[*idx],
                        6.0,
                        egui::Color32::ORANGE,
                        egui::Stroke::new(1.0, egui::Color32::ORANGE),
                    );
                }
            }
        }

        // Draw members
        for member in &self.edges {
            painter.line_segment(
                [self.points[member.p1], self.points[member.p2]],
                egui::Stroke::new(2.0, egui::Color32::RED),
            );
        }
        for force in &self.force {
            painter.line_segment(
                [self.points[force.p1], force.p2],
                egui::Stroke::new(3.0, egui::Color32::GREEN),
            );
        }
    }

    fn draw_command_bar(&self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("command bar").show(ctx, |ui| {
            let mode_str = match self.mode {
                Mode::Edit => "Edit",
                Mode::TextEdit => "Input",
                Mode::Command => "Command",
                Mode::Solve => "Solve",
                Mode::Insert => "Insert",
            };

            ui.label(mode_str);
        });
    }

    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}
