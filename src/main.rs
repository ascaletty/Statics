use truss::Truss;

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
