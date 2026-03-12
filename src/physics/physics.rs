use std::f32;
use std::ops::Deref;

use crate::Connection;
use crate::ConnectionData;
use crate::Force;
use crate::Member;
use crate::Truss;
use egui::Pos2;
use egui::emath::Numeric;
use nalgebra::DMatrix;
use nalgebra::Matrix1x4;
use nalgebra::Matrix4x1;
use nalgebra::linalg::FullPivLU;
use nalgebra_sparse::CooMatrix;
pub fn calculate_member_stress(truss: &mut Truss) -> DMatrix<f32> {
    let size = 2 * truss.points.len();
    let mut reactions = 0;
    let mut matrix = DMatrix::<f32>::zeros(size, size);

    let mut zeros = DMatrix::<f32>::zeros(size, 1);
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
                matrix[(1, 2)] = 1.0;
                // matrix[(id * 2, halfsize)] = 1.;

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
        println!("forcing{}", zeros);
        println!("matrix after placing coefficeints: {}", matrix);
        let decomp = FullPivLU::new(matrix);

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
        decomp.solve_mut(&mut zeros);
        println!("sol? :{}", zeros);
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
use nalgebra::base::Matrix4;
use nalgebra_sparse::CscMatrix;
use nalgebra_sparse::CsrMatrix;
use nalgebra_sparse::csc;
use nalgebra_sparse::factorization::CscCholesky;
use nalgebra_sparse_linalg::na_sparse::na::Matrix1;
use nalgebra_sparse_linalg::na_sparse::na::MatrixXx1;

fn construct_stiffness_matrix(member: &Member, points: &Vec<Pos2>) -> Matrix4<f32> {
    let p1 = points[member.p1];

    let p2 = points[member.p2];
    let deltay = p2.y - p1.y;
    let deltax = p2.x - p1.x;
    let theta = f32::atan2(deltay, deltax);
    let c = f32::cos(theta);
    let length = p1.distance(p2);
    let A_s = 2.0;
    let E = 69e9;

    let s = f32::sin(theta);
    let stiffness = Matrix4::new(
        c * c,
        c * s,
        -c * c,
        -c * s,
        c * s,
        s * s,
        -c * s,
        -s * s,
        -c * c,
        -c * s,
        c * c,
        c * s,
        -c * s,
        -s * s,
        c * s,
        s * s,
    );
    stiffness * (A_s * E / length)
}

fn construct_global_stiffness(truss: &Truss) -> CooMatrix<f32> {
    let n = truss.points.len();
    let mut k_global = CooMatrix::<f32>::zeros(2 * n, 2 * n);
    for m in &truss.edges {
        let k_local = construct_stiffness_matrix(m, &truss.points);
        let map = [2 * m.p1, 2 * m.p1 + 1, 2 * m.p2, 2 * m.p2 + 1];

        for i in 0..4 {
            for j in 0..4 {
                k_global.push(map[i], map[j], k_local[(i, j)]);
            }
        }
    }
    k_global
}
fn construct_force_matrix(truss: &Truss) -> DMatrix<f32> {
    let mut forces = DMatrix::zeros(2 * truss.points.len(), 1);
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
        forces[(id * 2 + 1, 0)] = force.mag * angley;
        forces[(force.p1 * 2, 0)] = force.mag * anglex;
    }
    print!("{}", forces);
    forces
}
fn dofpenalty(truss: &Truss, mut stiffness: CooMatrix<f32>, mut f: DMatrix<f32>) -> DMatrix<f32> {
    let penalty = 1e8
        * stiffness
            .values()
            .iter()
            .fold(0.0_f32, |a, &b| a.max(b.abs()));
    for connection in &truss.connections {
        match connection {
            ConnectionData::Pin(idx) => {
                let x = 2 * idx;
                let y = 2 * idx + 1;

                stiffness.push(x, x, penalty);
                stiffness.push(y, y, penalty);
                f[x] = 0.0;
                f[y] = 0.0;
            }
            ConnectionData::Roller(idx) => {
                let y = 2 * idx + 1;
                stiffness.push(y, y, penalty);
                f[y] = 0.0;
            }
        }
    }
    let k_global: CscMatrix<f32> = CscMatrix::from(&stiffness);
    println!("{:?}", k_global);
    let den = DMatrix::from(&k_global);
    // let eig = den.symetriceigen();
    let cscC = CscCholesky::factor(&k_global).unwrap();

    let fa = cscC.solve(&f);
    print!("stiffness{}", fa);
    fa
}
pub fn solve_stiff(truss: &mut Truss) {
    let k_global = construct_global_stiffness(truss);
    let f = construct_force_matrix(truss);
    let awns = dofpenalty(truss, k_global, f);
    for member in &truss.edges {
        let points = &truss.points;
        let p1 = points[member.p1];

        let p2 = points[member.p2];
        let deltay = p2.y - p1.y;
        let deltax = p2.x - p1.x;
        let theta = f32::atan2(deltay, deltax);
        let c = f32::cos(theta);
        let length = p1.distance(p2);
        let A_s = 20.0;
        let E = 69e9;

        let s = f32::sin(theta);
        let transform = Matrix1x4::new(-c, -s, c, s);
        let disp = Matrix4x1::new(
            awns[2 * member.p1],
            awns[2 * member.p1 + 1],
            awns[2 * member.p2],
            awns[2 * member.p2 + 1],
        );
        let forcesvec = transform * disp * (A_s * E / length);
        print!("forces member {}-{}, {}", member.p1, member.p2, forcesvec);
    }
}
