use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::Window;

use crate::World;
use crate::components::agent_component::AgentComponent;
use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::components::move_to_target_component::MoveToTargetComponent;
use crate::configuration::Config;
use crate::ecs::{Ecs, iter_components};
use crate::ecs::{EntityInfo, iter_entities, to_ctype};
use crate::gui::text_renderer::TextRenderer;
use crate::shared_data::biome::humidity;
use crate::shared_data::body_grid;
use std::any::TypeId;
use std::f64::consts::PI;

pub struct Renderer<'ttf> {
    canvas: Canvas<Window>,
    text_renderer: TextRenderer<'ttf>,
    debug_mode: usize,
    window_width: u32,
    window_height: u32,
    camera_offset_x: isize,
    camera_offset_y: isize,
    zoom: f64,
    selected_agent: Option<usize>,
    selected_agent_description: Option<Vec<String>>,
}

impl<'ttf> Renderer<'ttf> {
    pub fn new(config: &Config, canvas: Canvas<Window>, ttf_context: &'ttf Sdl2TtfContext) -> Self {
        Renderer {
            canvas,
            text_renderer: TextRenderer::new(ttf_context),
            debug_mode: 0,
            window_width: config.renderer.screen_width,
            window_height: config.renderer.screen_height,
            camera_offset_x: 0,
            camera_offset_y: 0,
            zoom: config.renderer.initial_zoom,
            selected_agent: None,
            selected_agent_description: None,
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

    pub fn zoom_in(&mut self, config: &Config) {
        self.zoom(config.renderer.zoom_factor);
    }

    pub fn zoom_out(&mut self, config: &Config) {
        self.zoom(1.0 / config.renderer.zoom_factor);
    }

    fn zoom(&mut self, zoom_factor: f64) {
        self.camera_offset_x = (self.camera_offset_x as f64 * zoom_factor) as isize;
        self.camera_offset_y = (self.camera_offset_y as f64 * zoom_factor) as isize;
        self.zoom *= zoom_factor;
    }

    pub fn move_camera_up(&mut self, config: &Config) {
        self.move_camera(0, config.renderer.move_camera_offset);
    }

    pub fn move_camera_down(&mut self, config: &Config) {
        self.move_camera(0, -config.renderer.move_camera_offset);
    }

    pub fn move_camera_left(&mut self, config: &Config) {
        self.move_camera(config.renderer.move_camera_offset, 0);
    }

    pub fn move_camera_right(&mut self, config: &Config) {
        self.move_camera(-config.renderer.move_camera_offset, 0);
    }

    fn move_camera(&mut self, offset_x: isize, offset_y: isize) {
        self.camera_offset_x += offset_x;
        self.camera_offset_y += offset_y;
    }

    pub fn select_agent_by_click(&mut self, ecs: &mut Ecs, p_x: i32, p_y: i32) {
        let (x, y) = self.pixel_to_simu_coords(p_x, p_y);
        for (body, info) in iter_components!(ecs, (AgentComponent), (BodyComponent)) {
            if body.collides_point(x, y) {
                self.selected_agent = Some(info.entity);
                self.selected_agent_description = None;
                break;
            }
        }
    }

    pub fn to_color(c: &[u8]) -> Color {
        Color::RGBA(c[0], c[1], c[2], c[3])
    }

    pub fn draw(&mut self, world: &mut World, config: &Config) {
        self.build_selected_agent_description(world);

        let ecs = &mut world.ecs;

        // Default background
        self.canvas
            .set_draw_color(Self::to_color(&config.renderer.color.background_color));
        self.canvas.clear();

        match self.debug_mode {
            1 => {
                self.draw_perlin_noise();
            }
            2 => {
                self.draw_body_grid(config);
                self.draw_graph(ecs, config);
                self.draw_path(ecs, config);
            }
            _ => {}
        }

        let colors = &config.renderer.color;

        // Draw corpses
        for (body, _) in iter_components!(ecs, (CorpseComponent, BodyComponent), (BodyComponent)) {
            self.draw_square(body, &colors.corpse_color, config.creature.size);
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
                self.draw_square(body, color, config.creature.size);
            } else {
                continue;
            }

            if let Some(creature) = ecs.component::<CreatureComponent>(&info) {
                // Draw health bar
                self.draw_rec(
                    (
                        pos.x(),
                        pos.y()
                            - config.creature.size / 2.0
                            - config.renderer.bar_height / 2.0
                            - 5.0,
                    ),
                    &colors.health_color,
                    (
                        config.renderer.bar_width * creature.health as f64
                            / config.creature.max_health as f64,
                        config.renderer.bar_height,
                    ),
                );

                // Draw energy bar
                self.draw_rec(
                    (
                        pos.x(),
                        pos.y()
                            - config.creature.size / 2.0
                            - config.renderer.bar_height * 1.5
                            - 5.0 * 2.0,
                    ),
                    &colors.energy_color,
                    (
                        config.renderer.bar_width * creature.energy as f64
                            / config.creature.max_energy as f64,
                        config.renderer.bar_height,
                    ),
                );
            }
        }

        // Draw obstacles
        for info in iter_entities!(ecs, ObstacleComponent, BodyComponent) {
            if let Some(body) = ecs.component::<BodyComponent>(&info) {
                self.draw_square(body, &colors.obstacle_color, config.obstacle_size);
            }
        }

        // Draw seeds & plants
        for (plant, body, _) in iter_components!(ecs, (), (PlantComponent, BodyComponent)) {
            if plant.is_seed {
                self.draw_square(body, &colors.seed_color, config.seed.renderer_size);
                continue;
            }

            self.draw_square(body, &colors.plant_color, plant.size);

            // Draw the plant's seeds
            let arc = 2.0 * PI / (plant.nb_seeds as f64);
            let mut a: f64 = 0.0;
            for _ in 0..plant.nb_seeds {
                let x = a.cos() * plant.size / 2.0;
                let y = a.sin() * plant.size / 2.0;
                let seed_body =
                    BodyComponent::new_traversable(body.x() + x, body.y() + y, 0.0, 0.0);
                self.draw_square(&seed_body, &colors.seed_color, config.seed.renderer_size);
                a += arc;
            }
        }

        if let Some(text) = &self.selected_agent_description {
            self.text_renderer
                .draw_multi_line(text, 0, 0, &mut self.canvas, config);
        }

        self.canvas.present();
    }

    fn build_selected_agent_description(&mut self, world: &mut World) {
        // Borrow the ecs (from world) and copy the agent information
        let agent_opt = if let Some(agent_entity) = self.selected_agent
            && self.selected_agent_description.is_none()
            && let Some(info) = world.ecs.get_entity_info(agent_entity)
        {
            Some(
                world
                    .ecs
                    .component::<AgentComponent>(&info)
                    .unwrap()
                    .clone(),
            )
        } else {
            None
        };

        // Borrow the goap (from world) to get the description
        if let Some(agent) = agent_opt
            && let Some(goap) = world.agent_system().map(|a| a.goap())
        {
            self.selected_agent_description = Some(agent.description(goap));
        }
    }

    fn draw_perlin_noise(&mut self) {
        // Perlin noise visualisation for tests
        let texture_creator = self.canvas.texture_creator();
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
        self.canvas.copy(&texture, None, None).unwrap();
    }

    fn draw_body_grid(&mut self, config: &Config) {
        let (g_x, g_y, g_w, g_h, g_cell_size, _, _) = body_grid::coords();

        let mut j = 0;
        loop {
            let y = g_y + g_cell_size * (j as f64);
            if y - config.renderer.grid_line_wideness / 2.0 > g_y + g_h {
                break;
            }
            self.draw_rec(
                (g_x + g_w / 2.0, y),
                &config.renderer.color.grid_color,
                (g_w, config.renderer.grid_line_wideness),
            );
            j += 1;
        }

        // Draw body grid columns
        let mut i = 0;
        loop {
            let x = g_x + g_cell_size * (i as f64);
            if x - config.renderer.grid_line_wideness / 2.0 > g_x + g_w {
                break;
            }
            self.draw_rec(
                (x, g_y + g_h / 2.0),
                &config.renderer.color.grid_color,
                (config.renderer.grid_line_wideness, g_h),
            );
            i += 1;
        }
    }

    fn draw_path(&mut self, ecs: &mut Ecs, config: &Config) {
        for (move_to_target_component, ..) in iter_components!(ecs, (), (MoveToTargetComponent)) {
            for i in 0..(move_to_target_component.path().len() as isize - 1) {
                let waypoint = move_to_target_component.path()[i as usize].clone();
                let next_waypoint = move_to_target_component.path()[i as usize + 1].clone();
                self.draw_line(
                    (waypoint.x(), waypoint.y()),
                    (next_waypoint.x(), next_waypoint.y()),
                    if waypoint.reached() {
                        &config.renderer.color.waypoint_reached_color
                    } else {
                        &config.renderer.color.waypoint_color
                    },
                );
            }
        }
    }

    fn draw_graph(&mut self, ecs: &mut Ecs, config: &Config) {
        for (move_to_target_component, ..) in iter_components!(ecs, (), (MoveToTargetComponent)) {
            for (node, neighbours) in move_to_target_component.graph().neighbours().iter() {
                for nb_node in neighbours {
                    self.draw_line(
                        (node.x(), node.y()),
                        (nb_node.x(), nb_node.y()),
                        &config.renderer.color.graph_color,
                    );
                }
            }
        }
    }

    fn draw_square(&mut self, body: &BodyComponent, color: &[u8], size: f64) {
        self.draw_rec((body.x(), body.y()), color, (size, size));
    }

    fn draw_line(&mut self, (ax, ay): (f64, f64), (bx, by): (f64, f64), color: &[u8]) {
        let (p_ax, p_ay) = self.simu_to_pixel_coords(ax, ay);
        let (p_bx, p_by) = self.simu_to_pixel_coords(bx, by);
        let start = Point::new(p_ax, p_ay);
        let end = Point::new(p_bx, p_by);
        self.canvas.set_draw_color(Self::to_color(color));
        let _ = self.canvas.draw_line(start, end);
    }

    pub fn draw_rec(&mut self, (x, y): (f64, f64), color: &[u8], (w, h): (f64, f64)) {
        let p_w = if w == 0.0 { 0 } else { (w * self.zoom) as u32 };
        let p_h = if h == 0.0 { 0 } else { (h * self.zoom) as u32 };
        let (mut p_x, mut p_y) = self.simu_to_pixel_coords(x, y);
        p_x -= (p_w / 2) as i32;
        p_y -= (p_h / 2) as i32;
        self.canvas.set_draw_color(Self::to_color(color));
        let _ = self.canvas.fill_rect(Rect::new(p_x, p_y, p_w, p_h));
    }

    fn simu_to_pixel_coords(&self, x: f64, y: f64) -> (i32, i32) {
        // The simulation coordinates origin should be in the center of the window
        let p_x =
            (x * self.zoom + (self.window_width as f64) / 2.0) as i32 + self.camera_offset_x as i32;
        let p_y = (y * self.zoom + (self.window_height as f64) / 2.0) as i32
            + self.camera_offset_y as i32;
        (p_x, p_y)
    }

    fn pixel_to_simu_coords(&self, p_x: i32, p_y: i32) -> (f64, f64) {
        // The simulation coordinates origin should be in the center of the window
        let x = (p_x as f64 - self.camera_offset_x as f64 - (self.window_width as f64 / 2.0))
            / self.zoom;
        let y = (p_y as f64 - self.camera_offset_y as f64 - (self.window_height as f64 / 2.0))
            / self.zoom;
        (x, y)
    }
}
