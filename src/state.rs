use crate::colors::ColorPallet;
use crate::connections::Connection;
use crate::egui::{Pos2, Rect, Vec2};
use crate::parts::{PORT_SIZE, Part, Port};
use crate::saveload::Config;
use crate::tools::Tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

//operation being completed, ie box selecting, resizing, etc
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InteractionState {
    Idle,
    Panning,
    BoxSelecting,
    Dragging,
    Connecting,
}

impl Default for InteractionState {
    fn default() -> Self {
        InteractionState::Idle
    }
}

#[derive(Clone, Deserialize, Serialize)]
pub struct CanvasSnapshot {
    pub parts: HashMap<u64, Part>,
    pub connections: Vec<Connection>,
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
    pub selection: Vec<u64>,
    pub box_select_start: Option<Pos2>,
    pub connect_start: Option<Port>,
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
                connections: Vec::new(),
                next_id: 0,
            },
            selection: Vec::new(),
            box_select_start: None,
            connect_start: None,
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

    pub fn port_at_pos(&self, world_pos: Pos2) -> Option<Port> {
        for part in self.canvas_snapshot.parts.values() {
            for (port_pos, input, port_id) in part.connections_pos_with_id() {
                if world_pos.distance_sq(port_pos) <= PORT_SIZE * PORT_SIZE {
                    return Some(Port {
                        part: part.id,
                        input: input,
                        port_id: port_id,
                    });
                }
            }
        }
        None
    }

    pub fn part_at_pos(&self, world_pos: Pos2) -> Option<&Part> {
        for part in self.canvas_snapshot.parts.values() {
            let size = part.part_data.size();

            let rect = Rect::from_min_size(part.pos, size);

            if rect.contains(world_pos) {
                return Some(part);
            }
        }
        None
    }

    pub fn parts_in_rect(&self, rect: Rect) -> Vec<u64> {
        self.canvas_snapshot
            .parts
            .values()
            .filter(|part| {
                let size = part.part_data.size();
                Rect::from_min_size(part.pos, size).intersects(rect)
            })
            .map(|part| part.id)
            .collect()
    }
}
