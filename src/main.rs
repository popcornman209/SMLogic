mod canvas;
mod colors;
mod connections;
mod interaction;
mod part_render;
mod parts;
mod saveload;
mod state;
mod tools;

use eframe::egui::{self};
use std::path::PathBuf;
use std::str::FromStr;

use state::AppState;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_title("Scrap Mechanic Logic Creator"),
        ..Default::default()
    };
    eframe::run_native(
        "Scrap Mechanic Logic Creator",
        options,
        Box::new(|cc| Ok(Box::new(AppState::new(&cc.egui_ctx)))),
    )
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.draw_sidebar(ctx);
        self.draw_settings(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = self.draw_canvas(ui, ctx);
            self.handle_input(ctx, &painter, &response);
            self.canvas_snapshot
                .save(PathBuf::from_str("/home/leo/test.json").expect("reg"));
        });
    }
}
