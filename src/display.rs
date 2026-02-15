use crate::components::all::*;
use crate::components::body_component::BodyComponent;
use crate::components::move_to_target_component::MoveToTargetComponent;
use crate::constants::*;
use crate::ecs::Ecs;
use crate::ecs::EntityInfo;
use crate::shared_data::body_grid;
use std::any::TypeId;

pub struct Display {
    debug_mode: bool,
}

impl Display {
    pub fn new() -> Self {
        Display { debug_mode: true }
    }

    pub fn toogle_debug_mode(&mut self) {
        self.debug_mode = !self.debug_mode;
    }

    pub fn draw(&self, ecs: &mut Ecs, pixels: &mut [u8], window_width: u32, window_height: u32) {
        // Background
        for pixel in pixels.chunks_exact_mut(4) {
            pixel.copy_from_slice(&[0xcc, 0xcc, 0xcc, 0xff]);
        }

        if self.debug_mode {
            draw_body_grid(pixels, window_width, window_height);
            draw_graph(ecs, pixels, window_width, window_height);
            draw_path(ecs, pixels, window_width, window_height);
        }

        // Draw corpses
        for (body, _) in iter_components!(ecs, (CorpseComponent, BodyComponent), (BodyComponent)) {
            draw_square(
                body,
                CORPSE_COLOR,
                CREATURE_PIXEL_SIZE,
                pixels,
                window_width,
                window_height,
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
                HERBIVOROUS_COLOR
            } else {
                CARNIVOROUS_COLOR
            };
            let pos;
            if let Some(body) = ecs.get_component::<BodyComponent>(&info) {
                pos = *body;
                draw_square(
                    body,
                    color,
                    CREATURE_PIXEL_SIZE,
                    pixels,
                    window_width,
                    window_height,
                );
            } else {
                continue;
            }

            if let Some(creature) = ecs.get_component::<CreatureComponent>(&info) {
                // Draw health bar
                draw_rec(
                    (
                        pos.get_x(),
                        pos.get_y()
                            - CREATURE_PIXEL_SIZE as f64 / 2.0
                            - BAR_HEIGHT as f64 / 2.0
                            - 5.0,
                    ),
                    HEALTH_COLOR,
                    (
                        (BAR_WIDTH as f32 * creature.health / MAX_HEALTH) as u32,
                        BAR_HEIGHT,
                    ),
                    pixels,
                    window_width,
                    window_height,
                );

                // Draw energy bar
                draw_rec(
                    (
                        pos.get_x(),
                        pos.get_y()
                            - CREATURE_PIXEL_SIZE as f64 / 2.0
                            - BAR_HEIGHT as f64 * 1.5
                            - 5.0 * 2.0,
                    ),
                    ENERGY_COLOR,
                    (
                        (BAR_WIDTH as f32 * creature.energy / MAX_ENERGY) as u32,
                        BAR_HEIGHT,
                    ),
                    pixels,
                    window_width,
                    window_height,
                );
            }
        }

        // Draw obstacles
        for info in iter_entities!(ecs, ObstacleComponent, BodyComponent) {
            if let Some(body) = ecs.get_component::<BodyComponent>(&info) {
                draw_square(
                    body,
                    OBSTACLE_COLOR,
                    OBSTACLE_PIXEL_SIZE,
                    pixels,
                    window_width,
                    window_height,
                );
            }
        }

        // Draw food
        for info in iter_entities!(ecs, FoodComponent, BodyComponent) {
            if let Some(body) = ecs.get_component::<BodyComponent>(&info) {
                draw_square(
                    body,
                    FOOD_COLOR,
                    FOOD_PIXEL_SIZE,
                    pixels,
                    window_width,
                    window_height,
                );
            }
        }
    }
}

fn draw_body_grid(pixels: &mut [u8], window_width: u32, window_height: u32) {
    let (g_x, g_y, g_w, g_h, g_cell_size, _, _) = body_grid::get_coords();

    let mut j = 0;
    loop {
        let y = g_y + g_cell_size * (j as f64);
        if y - GRID_LINE_WIDENESS as f64 / 2.0 > g_y + g_h {
            break;
        }
        draw_rec(
            (g_x + g_w / 2.0, y),
            GRID_COLOR,
            (g_w as u32, GRID_LINE_WIDENESS),
            pixels,
            window_width,
            window_height,
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
        draw_rec(
            (x, g_y + g_h / 2.0),
            GRID_COLOR,
            (GRID_LINE_WIDENESS, g_h as u32),
            pixels,
            window_width,
            window_height,
        );
        i += 1;
    }
}

fn draw_path(ecs: &mut Ecs, pixels: &mut [u8], window_width: u32, window_height: u32) {
    for (move_to_target_component, ..) in iter_components!(ecs, (), (MoveToTargetComponent)) {
        for i in 0..(move_to_target_component.path().len() - 1) {
            let waypoint = move_to_target_component.path()[i].clone();
            let next_waypoint = move_to_target_component.path()[i + 1].clone();
            draw_edge(
                (waypoint.x(), waypoint.y()),
                (next_waypoint.x(), next_waypoint.y()),
                if waypoint.reached() {
                    WAYPOINT_REACHED_COLOR
                } else {
                    WAYPOINT_COLOR
                },
                pixels,
                window_width,
                window_height,
            );
        }
    }
}

fn draw_graph(ecs: &mut Ecs, pixels: &mut [u8], window_width: u32, window_height: u32) {
    for (move_to_target_component, ..) in iter_components!(ecs, (), (MoveToTargetComponent)) {
        for (node, neighbours) in move_to_target_component.graph().neighbours().iter() {
            for nb_node in neighbours {
                draw_edge(
                    (node.get_x(), node.get_y()),
                    (nb_node.get_x(), nb_node.get_y()),
                    GRAPH_COLOR,
                    pixels,
                    window_width,
                    window_height,
                );
            }
        }
    }
}

fn draw_square(
    body: &BodyComponent,
    color: &[u8],
    size: u32,
    pixels: &mut [u8],
    window_width: u32,
    window_height: u32,
) {
    draw_rec(
        (body.get_x(), body.get_y()),
        color,
        (size, size),
        pixels,
        window_width,
        window_height,
    );
}

// This is probably horribly inefficient, just use it for debugging display
fn draw_edge(
    (ax, ay): (f64, f64),
    (bx, by): (f64, f64),
    color: &[u8],
    pixels: &mut [u8],
    window_width: u32,
    window_height: u32,
) {
    let dx = bx - ax;
    let dy = by - ay;
    let m1 = dy / dx;
    let m2 = dx / dy;

    let x_min = ax + (window_width as f64) / 2.0;
    let x_max = bx + (window_width as f64) / 2.0;
    let y_min = ay + (window_height as f64) / 2.0;
    let y_max = by + (window_height as f64) / 2.0;

    let thickness = GRAPH_EDGE_THICKNESS as isize;
    let mut x = x_min;
    while x <= x_max {
        let pix_x = x as isize;
        for j in (-thickness / 2)..(thickness / 2) {
            let pix_y = (m1 * (x - x_min) + y_min) as isize + j;
            if pix_x >= 0
                && pix_x < window_width as isize
                && pix_y >= 0
                && pix_y < window_height as isize
            {
                let index = ((pix_y as usize) * (window_width as usize) + (pix_x as usize)) * 4;
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
                && pix_x < window_width as isize
                && pix_y >= 0
                && pix_y < window_height as isize
            {
                let index = ((pix_y as usize) * (window_width as usize) + (pix_x as usize)) * 4;
                pixels[index..(index + 4)].copy_from_slice(color);
            }
        }

        y += 1.0;
    }
}

fn draw_rec(
    (x, y): (f64, f64),
    color: &[u8],
    (width, height): (u32, u32),
    pixels: &mut [u8],
    window_width: u32,
    window_height: u32,
) {
    let pos_in_window = (
        x + (window_width as f64) / 2.0,
        y + (window_height as f64) / 2.0,
    );
    let w = width as i64 / 2;
    let h = height as i64 / 2;
    for i in -w..w {
        for j in -h..h {
            let pixel_pos = (pos_in_window.0 as i64 + i, pos_in_window.1 as i64 + j);
            if pixel_pos.0 >= 0
                && pixel_pos.0 < window_width as i64
                && pixel_pos.1 >= 0
                && pixel_pos.1 < window_height as i64
            {
                let index =
                    ((pixel_pos.1 as usize) * (window_width as usize) + (pixel_pos.0 as usize)) * 4;
                pixels[index..(index + 4)].copy_from_slice(color);
            }
        }
    }
}
