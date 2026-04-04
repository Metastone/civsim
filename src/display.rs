use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::components::move_to_target_component::MoveToTargetComponent;
use crate::configuration::Config;
use crate::ecs::Ecs;
use crate::ecs::EntityInfo;
use crate::shared_data::biome::humidity;
use crate::shared_data::body_grid;
use std::any::TypeId;
use std::f64::consts::PI;

pub struct Display {
    debug_mode: usize,
    window_width: u32,
    window_height: u32,
    camera_offset_x: isize,
    camera_offset_y: isize,
    zoom: f64,
    config: Config,
}

impl Display {
    pub fn new(config: &Config) -> Self {
        Display {
            debug_mode: 0,
            window_width: config.display.screen_width,
            window_height: config.display.screen_height,
            camera_offset_x: 0,
            camera_offset_y: 0,
            zoom: config.display.initial_zoom,
            config: *config,
        }
    }

    pub fn resize(&mut self, window_width: u32, window_height: u32) {
        self.window_width = window_width;
        self.window_height = window_height;
    }

    pub fn toogle_debug_mode(&mut self) {
        self.debug_mode += 1;
        self.debug_mode %= 3;
    }

    pub fn zoom_in(&mut self) {
        self.zoom(self.config.display.zoom_factor);
    }

    pub fn zoom_out(&mut self) {
        self.zoom(1.0 / self.config.display.zoom_factor);
    }

    fn zoom(&mut self, zoom_factor: f64) {
        self.camera_offset_x = (self.camera_offset_x as f64 * zoom_factor) as isize;
        self.camera_offset_y = (self.camera_offset_y as f64 * zoom_factor) as isize;
        self.zoom *= zoom_factor;
    }

    pub fn move_camera_up(&mut self) {
        self.move_camera(0, self.config.display.move_camera_offset);
    }

    pub fn move_camera_down(&mut self) {
        self.move_camera(0, -self.config.display.move_camera_offset);
    }

    pub fn move_camera_left(&mut self) {
        self.move_camera(self.config.display.move_camera_offset, 0);
    }

    pub fn move_camera_right(&mut self) {
        self.move_camera(-self.config.display.move_camera_offset, 0);
    }

    fn move_camera(&mut self, offset_x: isize, offset_y: isize) {
        self.camera_offset_x += offset_x;
        self.camera_offset_y += offset_y;
    }

    fn to_color(c: &[u8]) -> Color {
        Color::RGBA(c[0], c[1], c[2], c[3])
    }

    pub fn draw(&self, ecs: &mut Ecs, canvas: &mut Canvas<Window>) {
        // Default background
        canvas.set_draw_color(Self::to_color(&self.config.display.color.background_color));
        canvas.clear();

        match self.debug_mode {
            1 => {
                self.draw_perlin_noise(canvas);
            }
            2 => {
                self.draw_body_grid(canvas);
                self.draw_graph(ecs, canvas);
                self.draw_path(ecs, canvas);
            }
            _ => {}
        }

        let colors = &self.config.display.color;

        // Draw corpses
        for (body, _) in iter_components!(ecs, (CorpseComponent, BodyComponent), (BodyComponent)) {
            self.draw_square(
                body,
                &colors.corpse_color,
                self.config.creature.size,
                canvas,
            );
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
                &colors.herbivorous_color
            } else {
                &colors.carnivorous_color
            };
            let pos;
            if let Some(body) = ecs.component::<BodyComponent>(&info) {
                pos = *body;
                self.draw_square(body, color, self.config.creature.size, canvas);
            } else {
                continue;
            }

            if let Some(creature) = ecs.component::<CreatureComponent>(&info) {
                // Draw health bar
                self.draw_rec(
                    (
                        pos.x(),
                        pos.y()
                            - self.config.creature.size / 2.0
                            - self.config.display.bar_height / 2.0
                            - 5.0,
                    ),
                    &colors.health_color,
                    (
                        self.config.display.bar_width * creature.health as f64
                            / self.config.creature.max_health as f64,
                        self.config.display.bar_height,
                    ),
                    canvas,
                );

                // Draw energy bar
                self.draw_rec(
                    (
                        pos.x(),
                        pos.y()
                            - self.config.creature.size / 2.0
                            - self.config.display.bar_height * 1.5
                            - 5.0 * 2.0,
                    ),
                    &colors.energy_color,
                    (
                        self.config.display.bar_width * creature.energy as f64
                            / self.config.creature.max_energy as f64,
                        self.config.display.bar_height,
                    ),
                    canvas,
                );
            }
        }

        // Draw obstacles
        for info in iter_entities!(ecs, ObstacleComponent, BodyComponent) {
            if let Some(body) = ecs.component::<BodyComponent>(&info) {
                self.draw_square(
                    body,
                    &colors.obstacle_color,
                    self.config.obstacle_size,
                    canvas,
                );
            }
        }

        // Draw seeds & plants
        for (plant, body, _) in iter_components!(ecs, (), (PlantComponent, BodyComponent)) {
            if plant.is_seed {
                self.draw_square(
                    body,
                    &colors.seed_color,
                    self.config.seed.display_size,
                    canvas,
                );
                continue;
            }

            self.draw_square(body, &colors.plant_color, plant.size, canvas);

            // Draw the plant's seeds
            let arc = 2.0 * PI / (plant.nb_seeds as f64);
            let mut a: f64 = 0.0;
            for _ in 0..plant.nb_seeds {
                let x = a.cos() * plant.size / 2.0;
                let y = a.sin() * plant.size / 2.0;
                let seed_body =
                    BodyComponent::new_traversable(body.x() + x, body.y() + y, 0.0, 0.0);
                self.draw_square(
                    &seed_body,
                    &colors.seed_color,
                    self.config.seed.display_size,
                    canvas,
                );
                a += arc;
            }
        }
    }

    fn draw_perlin_noise(&self, canvas: &mut Canvas<Window>) {
        // Perlin noise visualisation for tests
        let texture_creator = canvas.texture_creator();
        let mut texture = texture_creator
            .create_texture_streaming(
                PixelFormatEnum::RGB24,
                self.window_width,
                self.window_height,
            )
            .unwrap();
        texture
            .with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for p_x in 0..self.window_width {
                    for p_y in 0..self.window_height {
                        let offset = p_y as usize * pitch + p_x as usize * 3;

                        let x = (p_x as f64
                            - (self.window_width as f64) / 2.0
                            - self.camera_offset_x as f64)
                            / self.zoom;
                        let y = (p_y as f64
                            - (self.window_height as f64) / 2.0
                            - self.camera_offset_y as f64)
                            / self.zoom;

                        let h = humidity(x, y);
                        let l = (h * 255.0) as u8;

                        buffer[offset] = l;
                        buffer[offset + 1] = l;
                        buffer[offset + 2] = l;
                    }
                }
            })
            .unwrap();
        canvas.copy(&texture, None, None).unwrap();
    }

    fn draw_body_grid(&self, canvas: &mut Canvas<Window>) {
        let (g_x, g_y, g_w, g_h, g_cell_size, _, _) = body_grid::coords();

        let mut j = 0;
        loop {
            let y = g_y + g_cell_size * (j as f64);
            if y - self.config.display.grid_line_wideness / 2.0 > g_y + g_h {
                break;
            }
            self.draw_rec(
                (g_x + g_w / 2.0, y),
                &self.config.display.color.grid_color,
                (g_w, self.config.display.grid_line_wideness),
                canvas,
            );
            j += 1;
        }

        // Draw body grid columns
        let mut i = 0;
        loop {
            let x = g_x + g_cell_size * (i as f64);
            if x - self.config.display.grid_line_wideness / 2.0 > g_x + g_w {
                break;
            }
            self.draw_rec(
                (x, g_y + g_h / 2.0),
                &self.config.display.color.grid_color,
                (self.config.display.grid_line_wideness, g_h),
                canvas,
            );
            i += 1;
        }
    }

    fn draw_path(&self, ecs: &mut Ecs, canvas: &mut Canvas<Window>) {
        for (move_to_target_component, ..) in iter_components!(ecs, (), (MoveToTargetComponent)) {
            for i in 0..(move_to_target_component.path().len() - 1) {
                let waypoint = move_to_target_component.path()[i].clone();
                let next_waypoint = move_to_target_component.path()[i + 1].clone();
                self.draw_line(
                    (waypoint.x(), waypoint.y()),
                    (next_waypoint.x(), next_waypoint.y()),
                    if waypoint.reached() {
                        &self.config.display.color.waypoint_reached_color
                    } else {
                        &self.config.display.color.waypoint_color
                    },
                    canvas,
                );
            }
        }
    }

    fn draw_graph(&self, ecs: &mut Ecs, canvas: &mut Canvas<Window>) {
        for (move_to_target_component, ..) in iter_components!(ecs, (), (MoveToTargetComponent)) {
            for (node, neighbours) in move_to_target_component.graph().neighbours().iter() {
                for nb_node in neighbours {
                    self.draw_line(
                        (node.x(), node.y()),
                        (nb_node.x(), nb_node.y()),
                        &self.config.display.color.graph_color,
                        canvas,
                    );
                }
            }
        }
    }

    fn draw_square(
        &self,
        body: &BodyComponent,
        color: &[u8],
        size: f64,
        canvas: &mut Canvas<Window>,
    ) {
        self.draw_rec((body.x(), body.y()), color, (size, size), canvas);
    }

    fn draw_line(
        &self,
        (ax, ay): (f64, f64),
        (bx, by): (f64, f64),
        color: &[u8],
        canvas: &mut Canvas<Window>,
    ) {
        let (p_ax, p_ay) = self.simu_to_pixel_coords(ax, ay);
        let (p_bx, p_by) = self.simu_to_pixel_coords(bx, by);
        let start = Point::new(p_ax, p_ay);
        let end = Point::new(p_bx, p_by);
        canvas.set_draw_color(Self::to_color(color));
        let _ = canvas.draw_line(start, end);
    }

    pub fn draw_rec(
        &self,
        (x, y): (f64, f64),
        color: &[u8],
        (w, h): (f64, f64),
        canvas: &mut Canvas<Window>,
    ) {
        let p_w = if w == 0.0 { 0 } else { (w * self.zoom) as u32 };
        let p_h = if h == 0.0 { 0 } else { (h * self.zoom) as u32 };
        let (mut p_x, mut p_y) = self.simu_to_pixel_coords(x, y);
        p_x -= (p_w / 2) as i32;
        p_y -= (p_h / 2) as i32;
        canvas.set_draw_color(Self::to_color(color));
        let _ = canvas.fill_rect(Rect::new(p_x, p_y, p_w, p_h));
    }

    fn simu_to_pixel_coords(&self, x: f64, y: f64) -> (i32, i32) {
        // The simulation coordinates origin should be in the center of the window
        let p_x =
            (x * self.zoom + (self.window_width as f64) / 2.0) as i32 + self.camera_offset_x as i32;
        let p_y = (y * self.zoom + (self.window_height as f64) / 2.0) as i32
            + self.camera_offset_y as i32;
        (p_x, p_y)
    }
}
