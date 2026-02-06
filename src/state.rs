use crate::colors::ColorPallet;
use crate::config::Config;
use crate::egui::{Pos2, Vec2};
use crate::parts::Part;
use crate::tools::Tool;
use std::collections::HashMap;

//operation being completed, ie box selecting, resizing, etc
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InteractionState {
    Idle,
    Panning,
}

impl Default for InteractionState {
    fn default() -> Self {
        InteractionState::Idle
    }
}

#[derive(Clone)]
pub struct CanvasSnapshot {
    pub parts: HashMap<u64, Part>,
    pub next_id: u64,
}

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub interaction_state: InteractionState,
    pub active_tool: Option<Tool>,
    pub settings_open: bool,
    // other live info
    pub pan_offset: Vec2,
    pub zoom: f32,
    pub canvas_snapshot: CanvasSnapshot,
    pub fps_idle: bool,
    // settings
    pub show_arrows: bool,
    pub show_grid: bool,
    pub show_connection_count: bool,
    pub snap_to_grid: bool,
    pub show_fps: bool,
    pub color_pallet: ColorPallet,
}

impl AppState {
    pub fn new(ctx: &egui::Context) -> Self {
        let config = Config::load();
        let result = Self {
            interaction_state: InteractionState::Idle,
            active_tool: None,
            settings_open: false,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            canvas_snapshot: CanvasSnapshot {
                parts: HashMap::new(),
                next_id: 0,
            },
            fps_idle: false,
            show_arrows: config.show_arrows,
            show_grid: config.show_grid,
            snap_to_grid: config.snap_to_grid,
            show_connection_count: config.show_connection_count,
            show_fps: config.show_fps,
            color_pallet: config.color_pallet.clone(),
            config: config,
        };
        result.color_pallet.apply_theme(ctx);
        result
    }

    pub fn screen_to_world(&self, screen_pos: Pos2) -> Pos2 {
        Pos2::new(
            screen_pos.x / self.zoom + self.pan_offset.x,
            screen_pos.y / self.zoom + self.pan_offset.y,
        )
    }

    pub fn world_to_screen(&self, world_pos: Pos2) -> Pos2 {
        Pos2::new(
            (world_pos.x - self.pan_offset.x) * self.zoom,
            (world_pos.y - self.pan_offset.y) * self.zoom,
        )
    }
}
