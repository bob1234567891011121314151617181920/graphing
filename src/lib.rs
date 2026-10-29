pub mod app;
pub mod data;
pub mod graph;
pub mod node;
mod short_numbers;

use crate::app::App;

pub fn run() {
    eframe::run_native(
        "graphing",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
    .unwrap();
}
