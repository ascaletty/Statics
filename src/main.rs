use truss::Truss;
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let native_options = eframe::NativeOptions::default();
    println!("PROGRAM STARTED");
    eframe::run_native(
        "My egui App",
        native_options,
        Box::new(|cc| Ok(Box::new(Truss::new(cc)))),
    )
    .unwrap();
}
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(Truss::new(cc)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;
    use egui::debug_text::print;
    use faer::Mat;
    use faer::prelude::*;
    use faer::traits::num_traits::ToPrimitive;
    fn zeros(mut matrix: Mat<f32>) -> Mat<f32> {
        for i in 0..matrix.nrows() {
            for j in 0..matrix.ncols() {
                if matrix[(i, j)].abs() < 1e-4 {
                    matrix[(i, j)] = 0.;
                }
            }
        }
        matrix
    }

    use egui::Pos2;
    use rayon::str::SplitInclusive;
    use std::fs;
    use std::path::Path;
    use truss::ConnectionData;
    use truss::Force;
    use truss::Member;
    use truss::Truss;

    #[derive(Debug, serde::Deserialize)]
    struct RawTruss {
        nodes: Vec<String>,
        members: Vec<String>,
        supports: serde_json::Map<String, serde_json::Value>,
        forces: Vec<String>,
        workspace: serde_json::Value,
        awnsers: Vec<f32>,
    }

    pub fn load_trusses_from_folder<P: AsRef<Path>>(folder: P) -> Vec<bool> {
        let mut vecbool = Vec::new();

        for entry in fs::read_dir(folder).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let file_name = path.file_name().unwrap().to_str().unwrap();

            if file_name.starts_with("truss") {
                let data = fs::read_to_string(&path).unwrap();
                let raw: RawTruss = serde_json::from_str(&data).unwrap();

                // Parse nodes/
                let nodes: Vec<Pos2> = raw
                    .nodes
                    .iter()
                    .map(|(s)| {
                        let parts: Vec<f32> = s
                            .split(',')
                            .map(|x| x.trim().parse::<f32>().unwrap())
                            .collect();
                        Pos2 {
                            x: parts[0],
                            y: parts[1],
                        }
                    })
                    .collect();

                // Parse members into your Member struct
                let edges: Vec<Member> = raw
                    .members
                    .iter()
                    .enumerate()
                    .map(|(i, s)| {
                        let parts: Vec<usize> = s
                            .split(',')
                            .map(|x| x.trim().parse::<usize>().unwrap())
                            .collect();
                        let start = parts[0];
                        let end = parts[1];
                        Member { p1: start, p2: end }
                    })
                    .collect();

                // TODO add connections
                let connections: Vec<ConnectionData> = raw
                    .supports
                    .iter()
                    .map(|(idx, typ)| {
                        if typ == "P" {
                            println!("Pin at {}", idx);
                            ConnectionData::Pin(idx.parse::<usize>().unwrap())
                        } else {
                            println!("Roller at {}", idx);
                            ConnectionData::Roller(idx.parse::<usize>().unwrap())
                        }
                    })
                    .collect();
                let forces = raw
                    .forces
                    .iter()
                    .map(|s| {
                        let parts: Vec<f32> = s
                            .split(',')
                            .map(|x| x.trim().parse::<f32>().unwrap())
                            .collect();
                        let p1x = nodes[parts[0].to_usize().unwrap()].x;

                        let p1y = nodes[parts[0].to_usize().unwrap()].y;
                        let p2 = Pos2::new((parts[1] + p1x), (parts[2] + p1y));
                        let mag = Pos2::new(p1x, p1y).distance(p2);
                        println!("mag{}, p2{}", mag, p2);
                        Force {
                            p1: parts[0].to_usize().unwrap(),
                            p2: p2,
                            mag: mag,
                        }
                    })
                    .collect();
                let mut truss = Truss {
                    force: forces,
                    last_node: None,
                    mode: truss::Mode::Insert,
                    input_buf: String::new(),
                    messagetyp: truss::MessageType::Forcemsg,
                    points: nodes,

                    edges,
                    connections: connections,
                };
                let mut expirimental = truss::calculate_member_stress(&mut truss);
                let mut expirimentalr = expirimental.map(|x| *x as i64);
                let awnser_mat =
                    Mat::from_fn(raw.awnsers.len(), 1, |i, j| raw.awnsers[i]).map(|x| *x as i64);
                if awnser_mat == expirimentalr {
                    vecbool.push(true);
                } else {
                    vecbool.push(false);
                }
            }
        }
        vecbool
    }
    #[derive(serde::Deserialize)]
    struct MatrixWrapper {
        matrix: Vec<Vec<f32>>,
    }
    //
    // fn test_triangle() -> Mat<f32> {
    //     let mut truss = Truss {
    //         force: vec![],
    //         last_node: None,
    //         mode: truss::Mode::Insert,
    //         input_buf: String::new(),
    //         messagetyp: truss::MessageType::Forcemsg,
    //         points: vec![],
    //
    //         edges: vec![],
    //         connections: vec![],
    //     };
    //     let one = Pos2::new(0., 0.);
    //     let two = Pos2::new(1., 0.);
    //     let three = Pos2::new(0., 1.);
    //     let force_end = Pos2::new(0., -1.);
    //     let n0 = Node { pos: one, id: 0 };
    //     let n1 = Node { pos: two, id: 1 };
    //     let n2 = Node { pos: three, id: 2 };
    //     let m0 = Member {
    //         start: n0.clone(),
    //         end: n1.clone(),
    //         id: 0,
    //     };
    //     let m1 = Member {
    //         start: n1.clone(),
    //         end: n2.clone(),
    //         id: 1,
    //     };
    //     let m2 = Member {
    //         start: n2.clone(),
    //         end: n0.clone(),
    //         id: 2,
    //     };
    //
    //     let p1 = Connection::Pin(one, 1);
    //     let r1 = Connection::Roller(two, 2);
    //     let f1 = Connection::Force(Force {
    //         start: one,
    //         end: force_end,
    //         id: 3,
    //         magnitude: 2.,
    //     });
    //     truss
    //         .nodes
    //         .append(&mut vec![n0.clone(), n1.clone(), n2.clone()]);
    //     truss
    //         .edges
    //         .append(&mut vec![m0, m1, m2.clone()]);
    //     truss
    //         .connections
    //         .append(&mut vec![p1.clone(), r1.clone(), f1.clone()]);
    //     truss::calculate_member_stress(&mut truss)
    // }

    // fn basic_triangle() -> Mat<f32> {
    //     mat![[0.], [0.], [0.], [0.], [2.], [0.],]
    // }

    #[test]
    fn test_batch() {
        let vecbool = load_trusses_from_folder("test_trusses");
        for bools in vecbool {
            assert!(bools, "the  truss had an error");
        }
    }

    // fn forcing_matrix__match() {
    //     let true_output = basic_triangle();
    //     let output = test_triangle();
    //
    //     assert_eq!(output.forcing, true_output.forcing);
    // }
    //
    // #[test]
    // fn val_matrix_match() {
    //     let true_output = basic_triangle();
    //
    //     let output = test_triangle();
    //     assert_eq!(output.matrix, true_output.matrix);
    // }
    //
}
