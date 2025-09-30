use bevy::{
    math::{Vec2, ops::atan},
    tasks::futures_lite::stream::iter,
};
use nalgebra::{
    Const, DMatrix, DimAdd, Matrix1xX, Matrix2xX, MatrixXx2, VecStorage, Vector, VectorN,
};
use truss::structs::*;
#[derive(Debug)]
struct PointToPoint {
    memberid: usize,
    start: Node,
    end: Node,
}
pub fn calculate_member_stress(truss: &mut Truss) {
    let size = 2 * truss.edges.len();
    let mut reactions = 0;
    let mut matrix = DMatrix::zeros(size, size);
    print!("matrix, {:?}", matrix);
    for connection in &truss.connections {
        println!("Connection: {:?}", connection);
        match connection {
            Connection::Pin(_pos, _id) => reactions += 2,
            Connection::Roller(_pos, _id) => reactions += 1,
            Connection::Force(force) => {
                let diff = force.end - force.start;
                //note this only works with downwards forces need to find a better way
                if diff.angle_to(Vec2::new(1.0, 0.)) == 0.
                    || diff.angle_to(Vec2::new(0., -1.0)) == 0.
                {
                    reactions += 1;
                } else {
                    reactions += 2;
                }
            }
        }
    }

    for node in &truss.nodes {
        println!("Node: {:?}", node);
    }
    for member in &truss.edges {
        println!("Member {:?}", member);
    }
    let sanity_check = truss.edges.len() + reactions;
    if sanity_check > 2 * truss.nodes.len() {
        println!("Solvable!");
        let mut anglevec: Vec<PointToPoint> = vec![];
        for member in &truss.edges {
            let start_node = truss.nodes.iter().find(|x| x.0 == member.start).unwrap();
            let end_node = truss.nodes.iter().find(|x| x.0 == member.end).unwrap();
            anglevec.push(PointToPoint {
                memberid: member.id,
                start: start_node.clone(),
                end: end_node.clone(),
            });
        }
        println!("anglevec {:?}", anglevec);
        let mut row_count = 0;

        for node in &truss.nodes {
            let mem_for_node: Vec<&PointToPoint> = anglevec
                .iter()
                .filter(|x| x.start.0 == node.0 || x.end.0 == node.0)
                .collect();
            let first = mem_for_node[0];
            let second = mem_for_node[1];
            let diff1 = first.start.0 - first.end.0;
            let diff2 = second.start.0 - second.end.0;
            let d1 = first.start.0.distance(first.end.0);
            let d2 = second.start.0.distance(second.end.0);
            let angle = atan(diff1.dot(diff2) / d1 * d2);
            matrix[(row_count, node.1)] = angle.cos();
            matrix[(row_count, node.1 + 1)] = angle.sin();
            row_count += 1;
        }
        print!("matrix{:?}", matrix);
        let mut zeros = nalgebra::MatrixXx1::zeros(size);
        for force in &truss.connections {
            match force {
                Connection::Force(force) => {
                    let nodes_with_force = truss.nodes.iter().filter(|x| x.0 == force.start);
                    for node in nodes_with_force {
                        print!("node with force {:?}", node);
                        zeros[(node.1, 0)] = force.magnitude;
                    }
                }

                _ => {}
            }
        }

        let inverse = matrix.clone().pseudo_inverse(0.).unwrap();
        print!("zeros: {:?}", zeros);
        let mut finals = nalgebra::MatrixXx1::from_element(size, 0.);
        // 6x6 * 6x1= 6x1
        inverse.mul_to(&zeros, &mut finals);
        println!("hi");
        println!("final {:?}", finals);
    } else {
        println!("need more reactions")
    }
}
