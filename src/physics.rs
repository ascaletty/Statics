use std::{f32::consts::PI, num::NonZeroUsize};

use faer::linalg::solvers::{PartialPivLu, SolveCore};

use bevy::{
    math::{Vec2, ops::atan},
    platform::collections::HashSet,
    tasks::futures_lite::stream::iter,
};
use faer::linalg::solvers::Qr;
use faer::{
    dyn_stack::{MemStack, mem},
    linalg::{cholesky::llt::solve::solve_in_place, solvers},
    mat::Mat,
    prelude::*,
};
use rayon::ThreadPoolBuilder;
use truss::structs::*;

pub fn calculate_member_stress(truss: &mut Truss) -> Mat<f32> {
    let size = 2 * truss.nodes.len();
    let mut reactions = 0;
    let mut matrix = Mat::<f32>::zeros(size, size);

    let mut zeros = Mat::<f32>::zeros(size, 1);
    print!("matrix, {:?}", matrix);
    for node in &truss.nodes {
        println!("Node: {:?}", node);
    }
    for member in &truss.edges {
        println!("Member {:?}", member);
    }
    // puts reactions at the node id for x and the node id +1 for y eqautions
    // the second half(left to right) of the matrix should be reactions, whereas the first half
    // should be the forces in all the members.
    let mut halfsize = size / 2;

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
