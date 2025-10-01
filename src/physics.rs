use std::f32::consts::PI;

use bevy::{
    math::{Vec2, ops::atan},
    tasks::futures_lite::stream::iter,
};
use nalgebra::{
    Const, DMatrix, DimAdd, Matrix1xX, Matrix2xX, MatrixXx1, MatrixXx2, VecStorage, Vector, VectorN,
};
use truss::structs::*;
#[derive(Debug)]
struct PointToPoint {
    memberid: usize,
    start: Node,
    end: Node,
    angle_to_y: f32,
}

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
                    .find(|x| x.0.distance(*pos) < 0.001)
                    .expect("need connections to be at nodes");
                matrix[(matching_node.1 - 1, halfsize)] = 1.;
                println!("put pin at {},{}", matching_node.1 - 1, halfsize);
                matrix[(matching_node.1, halfsize + 1)] = 1.;

                println!("put pin at {},{}", matching_node.1, halfsize + 1);
                2
            }
            Connection::Roller(pos, _id) => {
                reactions += 1;
                let matching_node = truss
                    .nodes
                    .iter()
                    .find(|x| x.0.distance(*pos) < 0.001)
                    .expect("need connections to be at nodes");
                //only doing the y reactions rn
                //will need to add support for rollers with x reactions
                matrix[(matching_node.1 + 1, halfsize)] = -1.;

                println!("put rollecr at {},{}", matching_node.1 + 1, halfsize);
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
        let mut anglevec: Vec<PointToPoint> = vec![];
        for member in &truss.edges {
            let start_node = truss.nodes.iter().find(|x| x.0 == member.start).unwrap();
            let end_node = truss.nodes.iter().find(|x| x.0 == member.end).unwrap();

            let diff = start_node.0 - end_node.0;

            let angle_to_y1 = diff.angle_to(Vec2::new(1.0, 0.));
            matrix[m]

            anglevec.push(PointToPoint {
                memberid: member.id,
                start: start_node.clone(),
                end: end_node.clone(),
                angle_to_y: angle_to_y1,
            });
        }
        println!("anglevec {:?}", anglevec);
        let mut row_count = 0;

        for node in &truss.nodes {
            let start = anglevec.iter().find(|x| x.start.0 == node.0).unwrap();
            let end = anglevec.iter().find(|x| x.end.0 == node.0).unwrap();
            println!(
                "start member for node {} @ {} is {:?}",
                node.1, node.0, start
            );

            println!("end member for node {} @ {} is {:?}", node.1, node.0, end);
            let diff2 = end.start.0 - end.end.0;
            println!("diff1(start) {}, diff2(end) {}", diff1, diff2);
            let d1 = start.start.0.distance(start.end.0);
            let d2 = end.start.0.distance(end.end.0);
            println!("d1(start) {}, d2(end) {}", d1, d2);
            let angle_between = ((diff1.dot(diff2)) / (d1 * d2)).acos();
            let angle_to_y1 = diff1.angle_to(Vec2::new(1.0, 0.));
            let angle_to_y2 = diff2.angle_to(Vec2::new(1.0, 0.));

            println!("angle for node {} is pi/2", node.1);

            println!("angle for node {} is {}", node.1, angle_between);
            //diff1
            matrix[(node.1 - 1, node.1 - 1)] = angle_to_y1.cos();
            println!(
                "Row {} Col {} <= {}",
                node.1 - 1,
                node.1 - 1,
                matrix[(node.1 - 1, node.1 - 1)]
            );

            println!("{}", matrix);

            matrix[(node.1, node.1)] = angle_to_y1.sin();
            println!(
                "Row {} Col {} <= {}",
                node.1,
                node.1,
                matrix[(node.1, node.1)]
            );

            println!("{}", matrix);

            //diff2
            matrix[(node.1 - 1, node.1 - 1)] = angle_to_y2.cos();
            println!(
                "Row {} Col {} <= {}",
                node.1 - 1,
                node.1 - 1,
                matrix[(node.1 - 1, node.1 - 1)]
            );

            println!("{}", matrix);

            matrix[(node.1, node.1)] = angle_to_y2.sin();
            println!(
                "Row {} Col {} <= {}",
                node.1,
                node.1,
                matrix[(node.1, node.1)]
            );
            println!("{}", matrix);
        }
        for force in &truss.connections {
            match force {
                Connection::Force(force) => {
                    let nodes_with_force = truss.nodes.iter().filter(|x| x.0 == force.start);
                    for node in nodes_with_force {
                        print!("node with force {:?}", node);
                        zeros[(node.1 - 1, 0)] = force.magnitude;
                    }
                }

                _ => {}
            }
        }

        let inverse = matrix.clone().pseudo_inverse(0.).unwrap();
        print!("zeros: {}", zeros);
        // 6x6 * 6x1= 6x1
        inverse.mul_to(&zeros, &mut finals);
        println!("matrix {}", matrix);
        println!("final {}", finals);
    } else {
        println!("need more reactions")
    }
    ResultMatrix {
        matrix: matrix,
        result: finals,
        forcing: zeros,
    }
}
