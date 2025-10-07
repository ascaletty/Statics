use std::f32::consts::PI;

use bevy::{
    math::{Vec2, ops::atan},
    platform::collections::HashSet,
    tasks::futures_lite::stream::iter,
};
use nalgebra::{
    Const, DMatrix, DimAdd, Matrix1xX, Matrix2xX, MatrixXx1, MatrixXx2, VecStorage, Vector, VectorN,
};
use truss::structs::*;

pub fn calculate_member_stress(truss: &mut Truss) -> ResultMatrix {
    let size = 2 * truss.edges.len();
    let mut reactions = 0;
    let mut matrix = DMatrix::zeros(size, size);

    let mut finals = nalgebra::MatrixXx1::from_element(size, 0.);

    let mut zeros = nalgebra::MatrixXx1::zeros(size);
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
                matrix[(matching_node.id, halfsize)] = 1.;

                println!("put pin at {},{}", matching_node.id, halfsize);
                matrix[(matching_node.id + 1, halfsize + 1)] = 1.;

                println!("put pin at {},{}", matching_node.pos, halfsize + 1);
                2
            }
            Connection::Roller(pos, _id) => {
                reactions += 1;
                let matching_node = truss
                    .nodes
                    .iter()
                    .find(|x| x.pos == *pos)
                    .expect("need roller to be at nodes");

                println!("node id found{} for roller", matching_node.id);

                //only doing the y reactions rn
                //will need to add support for rollers with x reactions
                //
                matrix[(matching_node.id + 1, halfsize)] = 1.;

                println!("put rollecr at {},{}", matching_node.id + 1, halfsize);
                1
            }
            Connection::Force(_force) => 0,
        };
        println!("halfsize count {halfsize}");
    }

    println!("Matrix after this connection:\n{}", matrix);
    print!("num of members {}", truss.edges.len());
    print!("num of reactions {}", reactions);
    let sanity_check = truss.edges.len() + reactions;
    if sanity_check >= size {
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
            println!("placing {row_x_start}, {col}");

            println!("placing {row_y_start}, {col}");

            println!("placing {row_x_end}, {col}");

            println!("placing {row_y_end}, {col}");
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
        let tol = 1e-2;
        let inverse = matrix.clone().pseudo_inverse(tol).unwrap();
        print!("zeros: {}", zeros);
        // 6x6 * 6x1= 6x1
        inverse.mul_to(&zeros, &mut finals);
        println!("matrix {}", matrix);
        println!("final {:?}", finals);
        for val in finals.as_mut_slice() {
            if (*val).abs() < 0.00001 {
                *val = 0.;
            } else {
                *val = (*val * 1000.0).round() / (1000.0)
            }
        }
    } else {
        println!("need more reactions")
    }
    ResultMatrix {
        matrix: matrix,
        result: finals,
        forcing: zeros,
    }
}
