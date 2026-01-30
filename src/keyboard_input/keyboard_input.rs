//     pub fn handle_insert(
//         commands: Commands,
//         keys: Res<ButtonInput<KeyCode>>,
//         mut mode: ResMut<Mode>,
//         cursor: Res<CursorLocation>,
//         mut truss: ResMut<Truss>,
//         mut size: ResMut<MatSize>,
//     ) {
//         let cursorloc = cursor.world_position().unwrap_or(Vec2::ZERO);
//         if keys.just_pressed(KeyCode::Space) {
//             size.0 += 1;
//             if size.0 == 1 {
//                 truss.mat = Vec::new();
//             }
//             if size.0 > 1 {
//                 let posmatch = truss.pos.iter().position(|x| *x == cursorloc);
//                 match posmatch {
//                     Some(idx) => truss.mat.insert(idx, true),
//                     None => {
//                         truss.mat.push(true);
//                         truss.pos.push(cursorloc)
//                     }
//                 }
//             }
//         }
//         if keys.just_pressed(KeyCode::Escape) {
//             *mode = Mode::Command;
//         }
//         if keys.just_pressed(KeyCode::KeyR) {}
//         if keys.just_pressed(KeyCode::KeyP) {}
//         if keys.just_pressed(KeyCode::KeyF) {
//             println!("enter magnitude");
//             *mode = Mode::InsertText;
//         }
//         // we can check multiple at once with `.any_*`
//         if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) {
//             // Either the left or right shift are being held down
//         }
//         if keys.any_just_pressed([KeyCode::Delete, KeyCode::Backspace]) {
//             // Either delete or backspace was just pressed
//         }
//     }
