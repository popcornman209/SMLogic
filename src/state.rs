use crate::colors::ColorPallet;
use crate::connections::{
    Connection, WIRE_WIDTH, closest_point_to_rect, compute_wire_route, dist_point_to_segment,
};
use crate::egui::{Pos2, Rect, Vec2};
use crate::parts::{PORT_SIZE, Part, Port};
use crate::saveload::{ClipboardData, Config};
use crate::tools::Tool;
use egui_notify::Toasts;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Instant;

//operation being completed, ie box selecting, resizing, etc
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InteractionState {
    Idle,
    Panning,
    BoxSelecting,
    Dragging,
    Connecting,
}

#[derive(Clone, PartialEq)]
pub enum Selection {
    Part(u64),
    Connection(usize),
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

pub struct AppState {
    pub config: Config,
    pub interaction_state: InteractionState,
    pub active_tool: Option<Tool>,
    pub settings_open: bool,
    pub clipboard_data: Option<ClipboardData>,
    pub toasts: Toasts,
    //project
    pub project_folder: Option<PathBuf>,
    pub project_sub_folder: Option<PathBuf>,
    pub current_folder_files: Vec<PathBuf>,
    pub current_module_path: Option<PathBuf>,
    pub undo_stack: Vec<CanvasSnapshot>,
    pub redo_stack: Vec<CanvasSnapshot>,
    pub connection_counts: HashMap<Port, u64>,
    // other live info
    pub pan_offset: Vec2,
    pub zoom: f32,
    pub canvas_snapshot: CanvasSnapshot,
    pub selection: Vec<Selection>,
    pub box_select_start: Option<Pos2>,
    pub connect_start: Option<Port>,
    pub last_project_reload: Instant,
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
            clipboard_data: None,
            toasts: Toasts::default(),
            project_folder: None,
            project_sub_folder: None,
            current_folder_files: Vec::new(),
            current_module_path: None,
            pan_offset: Vec2::ZERO,
            zoom: 1.0,
            canvas_snapshot: CanvasSnapshot {
                parts: HashMap::new(),
                connections: Vec::new(),
                next_id: 0,
            },
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            connection_counts: HashMap::new(),
            selection: Vec::new(),
            box_select_start: None,
            connect_start: None,
            last_project_reload: Instant::now(),
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

    pub fn connection_at_pos(&self, world_pos: Pos2) -> Option<usize> {
        let half_width = WIRE_WIDTH * 0.5;

        for (i, connection) in self.canvas_snapshot.connections.iter().enumerate() {
            if let (Some(start), Some(end)) = (connection.start.pos(self), connection.end.pos(self))
            {
                let points = compute_wire_route(start, end);

                for seg in points.windows(2) {
                    if dist_point_to_segment(world_pos, seg[0], seg[1]) <= half_width {
                        return Some(i);
                    }
                }
            }
        }
        None
    }

    pub fn connections_in_rect(&self, rect: Rect) -> Vec<usize> {
        let half_width = WIRE_WIDTH / 2.0;
        let expanded = rect.expand(half_width);

        self.canvas_snapshot
            .connections
            .iter()
            .enumerate()
            .filter(|(_i, connection)| {
                if let (Some(start), Some(end)) =
                    (connection.start.pos(self), connection.end.pos(self))
                {
                    let points = compute_wire_route(start, end);

                    points.windows(2).any(|seg| {
                        let a = seg[0];
                        let b = seg[1];

                        // horizontal
                        if (a.y - b.y).abs() < f32::EPSILON {
                            let y = a.y;
                            let min_x = a.x.min(b.x);
                            let max_x = a.x.max(b.x);

                            expanded.top() <= y
                                && y <= expanded.bottom()
                                && max_x >= expanded.left()
                                && min_x <= expanded.right()
                        }
                        //vertical
                        else if (a.x - b.x).abs() < f32::EPSILON {
                            let x = a.x;
                            let min_y = a.y.min(b.y);
                            let max_y = a.y.max(b.y);

                            expanded.left() <= x
                                && x <= expanded.right()
                                && max_y >= expanded.top()
                                && min_y <= expanded.bottom()
                        } else {
                            false
                        }
                    })
                } else {
                    false
                }
            })
            .map(|(i, _)| i)
            .collect()
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

    pub fn push_undo(&mut self) {
        self.redo_stack.clear();
        self.undo_stack.push(self.canvas_snapshot.clone());
    }
    pub fn undo(&mut self) {
        if let Some(snapshot) = self.undo_stack.pop() {
            self.redo_stack.push(self.canvas_snapshot.clone());
            self.canvas_snapshot = snapshot;
            self.selection.clear();
        }
    }
    pub fn redo(&mut self) {
        if let Some(snapshot) = self.redo_stack.pop() {
            self.undo_stack.push(self.canvas_snapshot.clone());
            self.canvas_snapshot = snapshot;
            self.selection.clear();
        }
    }

    pub fn reload_connection_counts(&mut self) {
        self.connection_counts.clear();
        for connection in &self.canvas_snapshot.connections {
            let start_count = self
                .connection_counts
                .entry(connection.start.clone())
                .or_insert(0);

            *start_count += 1;

            let end_count = self
                .connection_counts
                .entry(connection.end.clone())
                .or_insert(0);

            *end_count += 1;
        }
    }
    pub fn open_file(&mut self, path: PathBuf) {
        let new_snapshot =
            CanvasSnapshot::load(path.clone(), self.project_folder.clone(), &mut self.toasts);
        match new_snapshot {
            Ok(snapshot) => {
                self.canvas_snapshot = snapshot;
                self.current_module_path = Some(path.clone());
                self.reload_connection_counts();
                self.toasts.success(format!(
                    "Opened file: {}",
                    path.file_name().unwrap().to_string_lossy()
                ));
            }
            Err(e) => {
                self.toasts.error(format!("Failed to load file: {}", e));
            }
        }
    }
}
