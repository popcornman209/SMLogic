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
use std::time::Instant;

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
        if self.last_project_reload.elapsed().as_secs() >= 3 {
            self.reload_project_folder();
            self.last_project_reload = Instant::now();
        }

        if ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Copy))) {
            if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                let world_pos = self.screen_to_world(pos);
                self.to_clipboard(world_pos);
            }
        } else if ctx.input(|i| i.events.iter().any(|e| matches!(e, egui::Event::Paste(_)))) {
            if let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) {
                let world_pos = self.screen_to_world(pos);
                self.load_clipboard(world_pos);
            }
        }

        self.draw_sidebar(ctx);
        self.draw_settings(ctx);
        self.draw_footer(ctx);
        egui::CentralPanel::default().show(ctx, |ui| {
            let (response, painter) = self.draw_canvas(ui, ctx);
            self.handle_input(ctx, &painter, &response);
        });
    }
}
