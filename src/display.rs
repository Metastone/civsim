use crate::algorithms::perlin_noise::perlin_noise;
use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::components::move_to_target_component::MoveToTargetComponent;
use crate::constants::*;
use crate::ecs::Ecs;
use crate::ecs::EntityInfo;
use crate::shared_data::body_grid;
use std::any::TypeId;

pub struct Display {
    is_initialized: bool,
    debug_mode: bool,
    window_width: u32,
    window_height: u32,
    camera_offset_x: isize,
    camera_offset_y: isize,
    zoom: f64,
}

impl Display {
    pub fn new() -> Self {
        Display {
            is_initialized: false,
            debug_mode: false,
            window_width: 0,
            window_height: 0,
            camera_offset_x: 0,
            camera_offset_y: 0,
            zoom: INITIAL_ZOOM,
        }
    }

    pub fn resize(&mut self, window_width: u32, window_height: u32) {
        self.window_width = window_width;
        self.window_height = window_height;
        self.is_initialized = true;
    }

    pub fn toogle_debug_mode(&mut self) {
        self.debug_mode = !self.debug_mode;
    }

    pub fn zoom_in(&mut self) {
        self.zoom(ZOOM_FACTOR);
    }

    pub fn zoom_out(&mut self) {
        self.zoom(1.0 / ZOOM_FACTOR);
    }

    fn zoom(&mut self, zoom_factor: f64) {
        self.camera_offset_x = (self.camera_offset_x as f64 * zoom_factor) as isize;
        self.camera_offset_y = (self.camera_offset_y as f64 * zoom_factor) as isize;
        self.zoom *= zoom_factor;
    }

    pub fn move_camera_up(&mut self) {
        self.move_camera(0, MOVE_CAMERA_OFFSET);
    }

    pub fn move_camera_down(&mut self) {
        self.move_camera(0, -MOVE_CAMERA_OFFSET);
    }

    pub fn move_camera_left(&mut self) {
        self.move_camera(MOVE_CAMERA_OFFSET, 0);
    }

    pub fn move_camera_right(&mut self) {
        self.move_camera(-MOVE_CAMERA_OFFSET, 0);
    }

    fn move_camera(&mut self, offset_x: isize, offset_y: isize) {
        self.camera_offset_x += offset_x;
        self.camera_offset_y += offset_y;
    }

    pub fn draw(&self, ecs: &mut Ecs, pixels: &mut [u8]) {
        if !self.is_initialized {
            return;
        }

        if self.debug_mode {
            // Background
            for p_x in 0..self.window_width {
                for p_y in 0..self.window_height {
                    let x = (p_x as f64
                        - (self.window_width as f64) / 2.0
                        - self.camera_offset_x as f64)
                        / self.zoom;
                    let y = (p_y as f64
                        - (self.window_height as f64) / 2.0
                        - self.camera_offset_y as f64)
                        / self.zoom;

                    let index =
                        ((p_y as usize) * (self.window_width as usize) + (p_x as usize)) * 4;

                    // TODO understand: my perlin noise seems to return values in [-1.0; 1.0]
                    let n = perlin_noise(x, y);
                    let b: u8 = ((n + 1.0) * 255.0 / 2.0) as u8;
                    let r: u8 = 255 - b;
                    pixels[index..(index + 4)].copy_from_slice(&[r, 0x00, b, 0xff]);
                }
            }

            self.draw_body_grid(pixels);
            self.draw_graph(ecs, pixels);
            self.draw_path(ecs, pixels);
        } else {
            // Background
            for pixel in pixels.chunks_exact_mut(4) {
                pixel.copy_from_slice(&[0xcc, 0xcc, 0xcc, 0xff]);
            }
        }

        // Draw corpses
        for (body, _) in iter_components!(ecs, (CorpseComponent, BodyComponent), (BodyComponent)) {
            self.draw_square(body, CORPSE_COLOR, CREATURE_SIZE, pixels);
        }

        // Get all creatures in the order of their entity ID.
        // The point is to always draw them in the same order, to avoid an ugly "flickering" effect
        // (they change archetype from one iteration to the next)
        let mut creature_infos: Vec<EntityInfo> =
            iter_entities!(ecs, CreatureComponent, BodyComponent).collect();
        creature_infos.sort_by(|a, b| a.entity.cmp(&b.entity));

        // Draw creatures
        for info in creature_infos {
            // Check what kind of creature this is
            let color = if ecs.has_component(info.arch_index, &to_ctype!(HerbivorousComponent)) {
                HERBIVOROUS_COLOR
            } else {
                CARNIVOROUS_COLOR
            };
            let pos;
            if let Some(body) = ecs.component::<BodyComponent>(&info) {
                pos = *body;
                self.draw_square(body, color, CREATURE_SIZE, pixels);
            } else {
                continue;
            }

            if let Some(creature) = ecs.component::<CreatureComponent>(&info) {
                // Draw health bar
                self.draw_rec(
                    (
                        pos.x(),
                        pos.y() - CREATURE_SIZE as f64 / 2.0 - BAR_HEIGHT as f64 / 2.0 - 5.0,
                    ),
                    HEALTH_COLOR,
                    (
                        (BAR_WIDTH as f32 * creature.health / MAX_HEALTH) as u32,
                        BAR_HEIGHT,
                    ),
                    pixels,
                );

                // Draw energy bar
                self.draw_rec(
                    (
                        pos.x(),
                        pos.y() - CREATURE_SIZE as f64 / 2.0 - BAR_HEIGHT as f64 * 1.5 - 5.0 * 2.0,
                    ),
                    ENERGY_COLOR,
                    (
                        (BAR_WIDTH as f32 * creature.energy / MAX_ENERGY) as u32,
                        BAR_HEIGHT,
                    ),
                    pixels,
                );
            }
        }

        // Draw obstacles
        for info in iter_entities!(ecs, ObstacleComponent, BodyComponent) {
            if let Some(body) = ecs.component::<BodyComponent>(&info) {
                self.draw_square(body, OBSTACLE_COLOR, OBSTACLE_SIZE, pixels);
            }
        }

        // Draw food
        for info in iter_entities!(ecs, FoodComponent, BodyComponent) {
            if let Some(body) = ecs.component::<BodyComponent>(&info) {
                self.draw_square(body, FOOD_COLOR, FOOD_SIZE, pixels);
            }
        }
    }

    fn draw_body_grid(&self, pixels: &mut [u8]) {
        let (g_x, g_y, g_w, g_h, g_cell_size, _, _) = body_grid::coords();

        let mut j = 0;
        loop {
            let y = g_y + g_cell_size * (j as f64);
            if y - GRID_LINE_WIDENESS as f64 / 2.0 > g_y + g_h {
                break;
            }
            self.draw_rec(
                (g_x + g_w / 2.0, y),
                GRID_COLOR,
                (g_w as u32, GRID_LINE_WIDENESS),
                pixels,
            );
            j += 1;
        }

        // Draw body grid columns
        let mut i = 0;
        loop {
            let x = g_x + g_cell_size * (i as f64);
            if x - GRID_LINE_WIDENESS as f64 / 2.0 > g_x + g_w {
                break;
            }
            self.draw_rec(
                (x, g_y + g_h / 2.0),
                GRID_COLOR,
                (GRID_LINE_WIDENESS, g_h as u32),
                pixels,
            );
            i += 1;
        }
    }

    fn draw_path(&self, ecs: &mut Ecs, pixels: &mut [u8]) {
        for (move_to_target_component, ..) in iter_components!(ecs, (), (MoveToTargetComponent)) {
            for i in 0..(move_to_target_component.path().len() - 1) {
                let waypoint = move_to_target_component.path()[i].clone();
                let next_waypoint = move_to_target_component.path()[i + 1].clone();
                self.draw_edge(
                    (waypoint.x(), waypoint.y()),
                    (next_waypoint.x(), next_waypoint.y()),
                    if waypoint.reached() {
                        WAYPOINT_REACHED_COLOR
                    } else {
                        WAYPOINT_COLOR
                    },
                    pixels,
                );
            }
        }
    }

    fn draw_graph(&self, ecs: &mut Ecs, pixels: &mut [u8]) {
        for (move_to_target_component, ..) in iter_components!(ecs, (), (MoveToTargetComponent)) {
            for (node, neighbours) in move_to_target_component.graph().neighbours().iter() {
                for nb_node in neighbours {
                    self.draw_edge(
                        (node.x(), node.y()),
                        (nb_node.x(), nb_node.y()),
                        GRAPH_COLOR,
                        pixels,
                    );
                }
            }
        }
    }

    fn draw_square(&self, body: &BodyComponent, color: &[u8], size: u32, pixels: &mut [u8]) {
        self.draw_rec((body.x(), body.y()), color, (size, size), pixels);
    }

    // This is probably horribly inefficient, just use it for debugging display
    fn draw_edge(
        &self,
        (ax, ay): (f64, f64),
        (bx, by): (f64, f64),
        color: &[u8],
        pixels: &mut [u8],
    ) {
        let dx = bx - ax;
        let dy = by - ay;
        let m1 = dy / dx;
        let m2 = dx / dy;

        let x_min = ax * self.zoom + (self.window_width as f64) / 2.0 + self.camera_offset_x as f64;
        let x_max = bx * self.zoom + (self.window_width as f64) / 2.0 + self.camera_offset_x as f64;
        let y_min =
            ay * self.zoom + (self.window_height as f64) / 2.0 + self.camera_offset_y as f64;
        let y_max =
            by * self.zoom + (self.window_height as f64) / 2.0 + self.camera_offset_y as f64;

        let thickness = GRAPH_EDGE_THICKNESS as isize;
        let mut x = x_min;
        while x <= x_max {
            let pix_x = x as isize;
            for j in (-thickness / 2)..(thickness / 2) {
                let pix_y = (m1 * (x - x_min) + y_min) as isize + j;
                if pix_x >= 0
                    && pix_x < self.window_width as isize
                    && pix_y >= 0
                    && pix_y < self.window_height as isize
                {
                    let index =
                        ((pix_y as usize) * (self.window_width as usize) + (pix_x as usize)) * 4;
                    pixels[index..(index + 4)].copy_from_slice(color);
                }
            }

            x += 1.0;
        }

        // Now do the same thing with inverted axis.
        // Otherwise the edges with slopes too steep do not appear continuous
        let mut y = y_min;
        while y <= y_max {
            let pix_y = y as isize;
            for i in (-thickness / 2)..(thickness / 2) {
                let pix_x = (m2 * (y - y_min) + x_min) as isize + i;
                if pix_x >= 0
                    && pix_x < self.window_width as isize
                    && pix_y >= 0
                    && pix_y < self.window_height as isize
                {
                    let index =
                        ((pix_y as usize) * (self.window_width as usize) + (pix_x as usize)) * 4;
                    pixels[index..(index + 4)].copy_from_slice(color);
                }
            }

            y += 1.0;
        }
    }

    fn draw_rec(
        &self,
        (x, y): (f64, f64),
        color: &[u8],
        (rec_width, rec_height): (u32, u32),
        pixels: &mut [u8],
    ) {
        // The simulation coordinates origin should be in the center of the window
        let rec_center_pos = (
            (x * self.zoom + (self.window_width as f64) / 2.0) as isize + self.camera_offset_x,
            (y * self.zoom + (self.window_height as f64) / 2.0) as isize + self.camera_offset_y,
        );

        let w = (rec_width as f64 * self.zoom / 2.0) as isize;
        let h = (rec_height as f64 * self.zoom / 2.0) as isize;
        for i in -w..w {
            for j in -h..h {
                let pixel_pos = (rec_center_pos.0 + i, rec_center_pos.1 + j);
                if pixel_pos.0 >= 0
                    && pixel_pos.0 < self.window_width as isize
                    && pixel_pos.1 >= 0
                    && pixel_pos.1 < self.window_height as isize
                {
                    let index = ((pixel_pos.1 as usize) * (self.window_width as usize)
                        + (pixel_pos.0 as usize))
                        * 4;
                    pixels[index..(index + 4)].copy_from_slice(color);
                }
            }
        }
    }
}
