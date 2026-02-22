use egui::{Key, Pos2};
#[derive(Debug, Default)]
pub enum MessageType {
    #[default]
    Forcemsg,
    Command,
}
#[derive(Default, Debug)]
pub struct Truss {
    edges: Vec<Member>,
    points: Vec<Pos2>,
    last_node: Option<usize>,
    mode: Mode,
    messagetyp: MessageType,
    force: Vec<Force>,
    input_buf: String,
}
#[derive(Default, Debug)]
pub struct Force {
    p1: usize,
    p2: Pos2,
    mag: u32,
}

#[derive(Default, Debug)]
pub struct Member {
    p1: usize,
    p2: usize,
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

use std::{f32::consts::PI, num::NonZeroUsize};

use faer::{
    dyn_stack::{MemStack, mem},
    linalg::{cholesky::llt::solve::solve_in_place, solvers},
    mat::Mat,
    prelude::*,
};
use rayon::ThreadPoolBuilder;

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

    for connection in &truss.connections {
        println!("Connection: {:?}", connection);

        halfsize += match connection {
            Connection::Pin(pos, _id) => {
                reactions += 2;
                let matching_node = truss
                    .nodes
                    .iter()
                    .find(|x| x.pos == *pos)
                    .expect("need connections to be at nodes");
                println!("node id found{} for pin", matching_node.id);
                matrix[(matching_node.id * 2, halfsize)] = 1.;

                println!("put pin at {},{}", matching_node.id, halfsize);
                matrix[(matching_node.id * 2 + 1, halfsize + 1)] = 1.;

                println!("put pin at {},{}", matching_node.id, halfsize + 1);
                2
            }
            Connection::Roller(pos, _id) => {
                reactions += 1;
                let matching_node = truss
                    .nodes
                    .iter()
                    .filter(|x| x.pos == *pos)
                    .min_by_key(|x| x.pos.distance(*pos) < 0.0001)
                    .expect("need roller to be at nodes");

                println!("node id found{} for roller", matching_node.id);

                //only doing the y reactions rn
                //will need to add support for rollers with x reactions
                //
                matrix[(matching_node.id * 2 + 1, halfsize)] = 1.;

                println!("put rollecr at {},{}", matching_node.id + 1, halfsize);
                1
            }
            Connection::Force(_force) => 0,
        };
        println!("halfsize count {halfsize}");
    }

    print!("num of members {}", truss.edges.len());
    print!("number of node{}", truss.nodes.len());
    print!("num of reactions {}", reactions);
    println!("matrix after placing reactions: {:?}", matrix);
    if size == truss.edges.len() + reactions {
        println!("Solvable!");

        for member in truss.edges.clone() {
            let start = member.start;
            let end = member.end;
            let dx = end.pos.x - start.pos.x;
            let dy = end.pos.y - start.pos.y;
            let length = (dx * dx + dy * dy).sqrt();
            println!("member id{}", member.id);
            println!("start.id {}", start.id);
            println!("end.id{}", end.id);

            let col = member.id; // member force column

            let row_x_start = 2 * start.id;
            let row_y_start = 2 * start.id + 1;
            let row_x_end = 2 * end.id;
            let row_y_end = 2 * end.id + 1;

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
        for force in &truss.connections {
            if let Connection::Force(field) = force {
                let nodes_with_force = truss.nodes.iter().filter(|x| x.pos == field.start);
                for node in nodes_with_force {
                    print!("node with force {:?}", node);
                    //this should place the force on only y rows
                    //place it at the end node of the
                    //
                    let start = field.start;
                    let end = field.end;
                    let diff = start - end;
                    let angle = diff.angle_to(Vec2::new(1.0, 0.));
                    println!("angle {}", angle);
                    println!("angle_x, {}", angle.cos());

                    println!("angle_y, {}", angle.sin());
                    let member = truss.edges.iter().find(|x| x.end.pos == node.pos).unwrap();

                    zeros[(node.id * 2 + 1, 0)] = -field.magnitude * angle.sin();
                    zeros[(node.id * 2, 0)] = field.magnitude * angle.cos();
                }
            }
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
    } else if truss.edges.len() + reactions > size {
        println!("overdefined")
    } else {
        println!("need more reactions")
    }
    zeros
}
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
                self.handle_insert_click(pos);
            }
        }
        if ctx.input(|i| i.key_pressed(egui::Key::F)) {
            self.mode = Mode::TextEdit;
        }

        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.mode = Mode::Command;
        }
    }

    fn handle_insert_click(&mut self, pos: egui::Pos2) {
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
                println!("solving beep boop")
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
