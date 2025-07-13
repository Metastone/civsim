use crate::components::*;
use crate::constants::*;
use crate::ecs::{ArchetypeManager, EntityId};
use std::any::TypeId;

pub fn draw(
    archetype_manager: &ArchetypeManager,
    pixels: &mut [u8],
    window_width: u32,
    window_height: u32,
) {
    // Background
    for pixel in pixels.chunks_exact_mut(4) {
        pixel.copy_from_slice(&[0xcc, 0xcc, 0xcc, 0xff]);
    }

    // Draw corpses
    for (arch_index, entity_index, _) in
        iter_entities_with!(archetype_manager, CorpseComponent, PositionComponent)
    {
        if let Some(position) =
            archetype_manager.get_component::<PositionComponent>(arch_index, entity_index)
        {
            draw_square(
                position,
                CORPSE_COLOR,
                CREATURE_PIXEL_SIZE,
                pixels,
                window_width,
                window_height,
            );
        }
    }

    // Get all creatures in the order of their entity ID.
    // The point is to always draw them in the same order, to avoid an ugly "flickering" effect
    // (they change archetype from one iteration to the next)
    let mut creature_indexes: Vec<(usize, usize, EntityId)> =
        iter_entities_with!(archetype_manager, CreatureComponent, PositionComponent).collect();
    creature_indexes.sort_by(|a, b| a.2.cmp(&b.2));

    // Draw creatures
    for (arch_index, entity_index, _) in creature_indexes {
        // Check what kind of creature this is
        let color = if archetype_manager.has_component(arch_index, &to_ctype!(HerbivorousComponent))
        {
            HERBIVOROUS_COLOR
        } else {
            CARNIVOROUS_COLOR
        };
        let pos;
        if let Some(position) =
            archetype_manager.get_component::<PositionComponent>(arch_index, entity_index)
        {
            pos = *position;
            draw_square(
                position,
                color,
                CREATURE_PIXEL_SIZE,
                pixels,
                window_width,
                window_height,
            );
        } else {
            continue;
        }

        if let Some(creature) =
            archetype_manager.get_component::<CreatureComponent>(arch_index, entity_index)
        {
            // Draw health bar
            draw_rec(
                (
                    pos.x,
                    pos.y + CREATURE_PIXEL_SIZE as f64 / 2.0 + BAR_HEIGHT as f64 / 2.0 + 5.0,
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
                    pos.x,
                    pos.y + CREATURE_PIXEL_SIZE as f64 / 2.0 + BAR_HEIGHT as f64 * 1.5 + 5.0 * 2.0,
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

    // Draw food
    for (arch_index, entity_index, _) in
        iter_entities_with!(archetype_manager, FoodComponent, PositionComponent)
    {
        if let Some(position) =
            archetype_manager.get_component::<PositionComponent>(arch_index, entity_index)
        {
            draw_square(
                position,
                FOOD_COLOR,
                FOOD_PIXEL_SIZE,
                pixels,
                window_width,
                window_height,
            );
        }
    }
}

fn draw_square(
    position: &PositionComponent,
    color: &[u8],
    size: u32,
    pixels: &mut [u8],
    window_width: u32,
    window_height: u32,
) {
    draw_rec(
        (position.x, position.y),
        color,
        (size, size),
        pixels,
        window_width,
        window_height,
    );
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
        -y + (window_height as f64) / 2.0,
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
