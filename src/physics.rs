use bevy::math::Vec2;
use nalgebra::{DMatrix, VecStorage, Vector, VectorN};
use truss::structs::*;

pub fn calculate_member_stress(truss: &mut Truss) {
    let size = truss.edges.len();
    let mut reactions = 0;
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
    let sanity_check = truss.edges.len() + reactions;
    if sanity_check > 2 * truss.nodes.len() {
        println!("Solvable!")
    } else {
        println!("need more reactions")
    }
}
