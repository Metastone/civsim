use crate::components::body_component::BodyComponent;
use crate::constants::*;
use crate::ecs::{EntityId, RESERVED_ENTITY_ID};
use std::cell::RefCell;

/* Collision computation grid.
 *
 * For a given entity, only the surronding cells will be checked for collision.
 * Hence, to be sure to not miss a collision, the cells must be at least as big
 * as the biggest entity.
 *
 * For quick reads I use vecs for storage. Deleted bodies are recognized with w = 0 && h == 0,
 * and they are deleted later during the next periodic grid cleanup.
 *
 * The grid starts with a certain size and is automatically resized when required
 * (when a body position is out of the grid)
 */

thread_local! {
    static BODY_GRID: RefCell<BodyGrid> = RefCell::new(
        BodyGrid::new(
            - (SCREEN_WIDTH as f64) / 2.0,
            SCREEN_WIDTH as f64 / 2.0,
            - (SCREEN_HEIGHT as f64) / 2.0,
            SCREEN_HEIGHT as f64 / 2.0,
            CREATURE_SIZE as f64
        )
    )
}

enum Direction {
    Up,
    Right,
    Down,
    Left,
}

pub struct ClosestEntityIterator {
    // Constants
    entity: EntityId,
    body: BodyComponent,
    start_cell_x: isize,
    start_cell_y: isize,
    cell_x: isize,
    cell_y: isize,
    max_taxicab_distance: f64,

    // nt-th closest in current body-grid cell (from 0)
    nth_closest: usize,

    // Clock-wise spiral variables
    direction: Direction,
    countdown_next_turn: usize,
    countdown_spiral_increase: usize,
    spiral_side_length: usize,
}

impl ClosestEntityIterator {
    fn new(
        body_grid: &mut BodyGrid,
        entity: EntityId,
        body: &BodyComponent,
        max_taxicab_distance: f64,
    ) -> Self {
        let (start_cell_x, start_cell_y) = body_grid.get_cell_coords(body.x(), body.y());
        ClosestEntityIterator {
            entity,
            body: *body,
            start_cell_x: start_cell_x as isize,
            start_cell_y: start_cell_y as isize,
            cell_x: start_cell_x as isize,
            cell_y: start_cell_y as isize,
            max_taxicab_distance,
            nth_closest: 0,
            direction: Direction::Up,
            countdown_next_turn: 0,
            countdown_spiral_increase: 2,
            spiral_side_length: 1,
        }
    }
}

impl Iterator for ClosestEntityIterator {
    // (<entity found>, <euclidian distance squared>)
    type Item = (EntityId, f64);

    fn next(&mut self) -> Option<Self::Item> {
        BODY_GRID.with_borrow(|grid| grid.next_closest_entity(self))
    }
}

enum GetCoordsResult {
    Ok(usize, usize),
    GridResized(usize, usize),
}

pub struct BodyGrid {
    // Coordinates of the space occupied by all entities,
    // relative to simulation coordinates
    x: f64,
    y: f64,
    w: f64,
    h: f64,

    cell_size: f64,

    nb_cells_x: usize,
    nb_cells_y: usize,

    grid: Vec<Vec<(EntityId, BodyComponent)>>,
}

impl BodyGrid {
    fn next_closest_entity(&self, it: &mut ClosestEntityIterator) -> Option<(EntityId, f64)> {
        let mut found_entity = RESERVED_ENTITY_ID;
        let mut squared_euclidian_distance = f64::MAX;

        while found_entity == RESERVED_ENTITY_ID {
            // If we got too far away from the starting cell, stop the search, end iterator
            let taxicab_distance = ((it.cell_x - it.start_cell_x).abs()
                + (it.cell_y - it.start_cell_y).abs()) as f64
                * self.cell_size;
            if taxicab_distance > it.max_taxicab_distance {
                return None;
            }

            // Look for the closest entity in the current cell
            if it.cell_x >= 0
                && it.cell_x < self.nb_cells_x as isize
                && it.cell_y >= 0
                && it.cell_y < self.nb_cells_y as isize
            {
                let mut entities_found = Vec::new();
                for (e, b) in
                    self.grid[it.cell_y as usize * self.nb_cells_x + it.cell_x as usize].iter()
                {
                    if *e != it.entity {
                        let d = (b.x() - it.body.x()).powi(2) + (b.y() - it.body.y()).powi(2);
                        entities_found.push((*e, d));
                    }
                }
                entities_found.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

                if it.nth_closest < entities_found.len() {
                    found_entity = entities_found[it.nth_closest].0;
                    squared_euclidian_distance = entities_found[it.nth_closest].1;
                    it.nth_closest += 1;

                    // We found an entity so exit, to advance to the next cell only when all
                    // entities in the current  cell are iterated on.
                    break;
                } else {
                    it.nth_closest = 0;
                }
            }

            // Handle direction change to follow a clock-wise spiral around the starting cell
            if it.countdown_next_turn == 0 {
                match it.direction {
                    Direction::Up => {
                        it.direction = Direction::Right;
                    }
                    Direction::Right => {
                        it.direction = Direction::Down;
                    }
                    Direction::Down => {
                        it.direction = Direction::Left;
                    }
                    Direction::Left => {
                        it.direction = Direction::Up;
                    }
                }
                if it.countdown_spiral_increase == 0 {
                    it.countdown_spiral_increase = 2;
                    it.spiral_side_length += 1;
                }
                it.countdown_spiral_increase -= 1;
                it.countdown_next_turn = it.spiral_side_length;
            }
            it.countdown_next_turn -= 1;

            // Advance one cell in the current direction
            match it.direction {
                Direction::Up => {
                    it.cell_y -= 1;
                }
                Direction::Right => {
                    it.cell_x += 1;
                }
                Direction::Down => {
                    it.cell_y += 1;
                }
                Direction::Left => {
                    it.cell_x -= 1;
                }
            }
        }

        // If an entity was found, return it to the iterator caller
        if found_entity != RESERVED_ENTITY_ID {
            return Some((found_entity, squared_euclidian_distance));
        }
        None
    }

    pub fn new(
        mut min_x: f64,
        mut max_x: f64,
        mut min_y: f64,
        mut max_y: f64,
        max_entity_size: f64,
    ) -> Self {
        min_x -= max_entity_size / 2.0;
        max_x += max_entity_size / 2.0;
        min_y -= max_entity_size / 2.0;
        max_y += max_entity_size / 2.0;
        let max_entity_size = CREATURE_SIZE as f64;
        let cell_size = max_entity_size * CELL_SIZE_FACTOR;

        // Compute the minimal size for the grid (float)
        let mut w = (max_x - min_x).abs();
        let mut h = (max_y - min_y).abs();

        // Deduce the number of cells (integer)
        let nb_cells_x = (w / cell_size) as usize + 1;
        let nb_cells_y = (h / cell_size) as usize + 1;

        // Re-compute the grid size (float),
        // so that it corresponds exactly to the number of cells
        w = cell_size * nb_cells_x as f64;
        h = cell_size * nb_cells_y as f64;

        BodyGrid {
            x: min_x,
            y: min_y,
            w,
            h,
            cell_size,
            nb_cells_x,
            nb_cells_y,
            grid: vec![Vec::new(); nb_cells_x * nb_cells_y],
        }
    }

    fn get_cell_coords(&mut self, x: f64, y: f64) -> (usize, usize) {
        match self.get_cell_coords_impl(x, y) {
            GetCoordsResult::Ok(cell_x, cell_y) => (cell_x, cell_y),
            GetCoordsResult::GridResized(cell_x, cell_y) => (cell_x, cell_y),
        }
    }

    fn get_cell_coords_impl(&mut self, x: f64, y: f64) -> GetCoordsResult {
        // Get coordinates relative to the grid
        let x_in_grid = x - self.x;
        let y_in_grid = y - self.y;

        /* Check if a grid resize is required on x and y axis independently.
         * If a resize is required, the grid will be made at least twice bigger,
         * or even more if the new body to accomodate is even further away.
         */
        let (resize_x, offset_cell_x, new_nb_cells_x): (bool, usize, usize) =
            self.check_resize(x_in_grid, self.w, self.nb_cells_x);
        let (resize_y, offset_cell_y, new_nb_cells_y): (bool, usize, usize) =
            self.check_resize(y_in_grid, self.h, self.nb_cells_y);

        // Resize the grid on x or y or both
        if resize_x || resize_y {
            let nb_cells_x = if resize_x {
                new_nb_cells_x
            } else {
                self.nb_cells_x
            };
            let nb_cells_y = if resize_y {
                new_nb_cells_y
            } else {
                self.nb_cells_y
            };

            // Allocate a new grid, copy content into it in the right location
            let mut grid = vec![Vec::new(); nb_cells_x * nb_cells_y];
            for x in 0..self.nb_cells_x {
                for y in 0..self.nb_cells_y {
                    let old_index = y * self.nb_cells_x + x;
                    let new_index = (y + offset_cell_y) * nb_cells_x + (x + offset_cell_x);
                    grid[new_index] = self.grid[old_index].clone();
                }
            }
            self.grid = grid;

            self.x -= offset_cell_x as f64 * self.cell_size;
            self.y -= offset_cell_y as f64 * self.cell_size;
            self.nb_cells_x = nb_cells_x;
            self.nb_cells_y = nb_cells_y;
            self.w = self.cell_size * nb_cells_x as f64;
            self.h = self.cell_size * nb_cells_y as f64;

            return GetCoordsResult::GridResized(
                ((x - self.x) / self.cell_size) as usize,
                ((y - self.y) / self.cell_size) as usize,
            );
        }

        // No grid resize required
        GetCoordsResult::Ok(
            (x_in_grid / self.cell_size) as usize,
            (y_in_grid / self.cell_size) as usize,
        )
    }

    // Check if a grid resize is required on one axis (x or y).
    // Return tuple (resize_required, offset_cells, nb_new_cells)
    fn check_resize(&self, coord_in_grid: f64, size: f64, nb_cells: usize) -> (bool, usize, usize) {
        if coord_in_grid < 0.0 {
            (
                true,
                nb_cells,
                usize::max(
                    nb_cells * 2,
                    (coord_in_grid.abs() / self.cell_size) as usize + 1 + nb_cells,
                ),
            )
        } else if coord_in_grid >= size {
            (
                true,
                0,
                usize::max(nb_cells * 2, (coord_in_grid / self.cell_size) as usize + 1),
            )
        } else {
            (false, 0, 0)
        }
    }

    /// Return true if the body was translated successfully (no collision).
    /// otherwise, return false and do nothing.
    fn try_translate(
        &mut self,
        entity: EntityId,
        target_entity: EntityId,
        body: &BodyComponent,
        offset_x: f64,
        offset_y: f64,
    ) -> bool {
        let mut translated_body = *body;
        translated_body.translate(offset_x, offset_y);

        if !self.collides_except_target(entity, target_entity, &translated_body) {
            self.translate(entity, body, translated_body);
            return true;
        }
        false
    }

    fn collides(&mut self, entity: EntityId, body: &BodyComponent) -> bool {
        self.collides_impl(entity, RESERVED_ENTITY_ID, body)
    }

    fn collides_except_target(
        &mut self,
        entity: EntityId,
        target_entity: EntityId,
        body: &BodyComponent,
    ) -> bool {
        self.collides_impl(entity, target_entity, body)
    }

    fn collides_impl(
        &mut self,
        entity: EntityId,
        target_entity: EntityId,
        body: &BodyComponent,
    ) -> bool {
        let (body_cell_x, body_cell_y) = match self.get_cell_coords_impl(body.x(), body.y()) {
            GetCoordsResult::Ok(x, y) => (x, y),
            GetCoordsResult::GridResized(x, y) => (x, y),
        };

        for i in -1..2 {
            let cell_x = body_cell_x as i64 + i;
            if cell_x < 0 || cell_x >= self.nb_cells_x as i64 {
                continue;
            }
            for j in -1..2 {
                let cell_y = body_cell_y as i64 + j;
                if cell_y < 0 || cell_y >= self.nb_cells_y as i64 {
                    continue;
                }
                for (e, b) in self.grid[cell_y as usize * self.nb_cells_x + cell_x as usize].iter()
                {
                    let is_deleted_body = b.w() == 0.0 && b.h() == 0.0;
                    let is_itself = *e == entity;
                    let is_target = target_entity != RESERVED_ENTITY_ID && *e == target_entity;
                    if b.is_traversable() || is_deleted_body || is_itself || is_target {
                        continue;
                    }
                    if body.collides(b) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn edge_collides(
        &mut self,
        a: (f64, f64),
        b: (f64, f64),
        entity: EntityId,
        target_entity: EntityId,
        margin: (f64, f64),
    ) -> bool {
        // a, b are 2D points.

        /* Get the grid cell coordinates for both vertices,
         * making sure that they are both valid if a grid resize occurs
         */
        self.get_cell_coords(a.0, a.1);
        let (b_cx, b_cy) = self.get_cell_coords(b.0, b.1);
        let (a_cx, a_cy) = self.get_cell_coords(a.0, a.1);

        // Get the bounds of a rectangle that contains the 2D points A and B (vertices of the
        // edge), with a margin
        let margin_cx = (margin.0 / self.cell_size) as isize + 1;
        let margin_cy = (margin.1 / self.cell_size) as isize + 1;
        let min_x = isize::max(0, isize::min(a_cx as isize, b_cx as isize) - margin_cx);
        let max_x = isize::min(
            self.nb_cells_x as isize - 1,
            isize::max(a_cx as isize, b_cx as isize) + margin_cx,
        );
        let min_y = isize::max(0, isize::min(a_cy as isize, b_cy as isize) - margin_cy);
        let max_y = isize::min(
            self.nb_cells_y as isize - 1,
            isize::max(a_cy as isize, b_cy as isize) + margin_cy,
        );

        for cx in min_x..(max_x + 1) {
            for cy in min_y..(max_y + 1) {
                for (e, body) in self.grid[cy as usize * self.nb_cells_x + cx as usize].iter() {
                    let is_deleted_body = body.w() == 0.0 && body.h() == 0.0;
                    let is_itself = *e == entity;
                    let is_target = *e == target_entity;
                    if body.is_traversable() || is_deleted_body || is_itself || is_target {
                        continue;
                    }
                    let inflated_body = BodyComponent::new_traversable(
                        body.x(),
                        body.y(),
                        body.w() + margin.0,
                        body.h() + margin.1,
                    );
                    if edge_collides_body(a, b, &inflated_body) {
                        return true;
                    }
                }
            }
        }
        false
    }

    fn translate(
        &mut self,
        entity: EntityId,
        body: &BodyComponent,
        translated_body: BodyComponent,
    ) {
        /* Get the grid cell coordinates for both bodies,
         * making sure that they are both valid if a grid resize occurs
         */
        let (cell_x, cell_y, t_cell_x, t_cell_y) = match (
            self.get_cell_coords_impl(body.x(), body.y()),
            self.get_cell_coords_impl(translated_body.x(), translated_body.y()),
        ) {
            (GetCoordsResult::Ok(x, y), GetCoordsResult::Ok(tx, ty)) => (x, y, tx, ty),
            _ => {
                // The grid was resized, re-compute the coords to be sure they are both valid
                let (x, y) = match self.get_cell_coords_impl(body.x(), body.y()) {
                    GetCoordsResult::Ok(x, y) => (x, y),
                    GetCoordsResult::GridResized(x, y) => (x, y),
                };
                let (tx, ty) =
                    match self.get_cell_coords_impl(translated_body.x(), translated_body.y()) {
                        GetCoordsResult::Ok(x, y) => (x, y),
                        GetCoordsResult::GridResized(x, y) => (x, y),
                    };
                (x, y, tx, ty)
            }
        };

        if cell_x == t_cell_x && cell_y == t_cell_y {
            self.update(entity, translated_body, cell_x, cell_y);
        } else {
            self.delete_from_cell(entity, cell_x, cell_y);
            self.add_to_cell(entity, translated_body, t_cell_x, t_cell_y);
        }
    }

    fn update(
        &mut self,
        entity: EntityId,
        translated_body: BodyComponent,
        cell_x: usize,
        cell_y: usize,
    ) {
        for (e, b) in self.grid[cell_y * self.nb_cells_x + cell_x].iter_mut() {
            if *e == entity {
                *b = translated_body;
                break;
            }
        }
    }

    fn delete(&mut self, entity: EntityId, body: &BodyComponent) {
        let (cell_x, cell_y) = match self.get_cell_coords_impl(body.x(), body.y()) {
            GetCoordsResult::Ok(x, y) => (x, y),
            GetCoordsResult::GridResized(x, y) => (x, y),
        };

        self.grid[cell_y * self.nb_cells_x + cell_x].retain(|(e, _)| *e != entity);
    }

    fn delete_from_cell(&mut self, entity: EntityId, cell_x: usize, cell_y: usize) {
        for (e, b) in self.grid[cell_y * self.nb_cells_x + cell_x].iter_mut() {
            if *e == entity {
                *b = BodyComponent::new_traversable(0.0, 0.0, 0.0, 0.0);
            }
        }
    }

    fn add(&mut self, entity: EntityId, body: &BodyComponent) {
        let (cell_x, cell_y) = match self.get_cell_coords_impl(body.x(), body.y()) {
            GetCoordsResult::Ok(x, y) => (x, y),
            GetCoordsResult::GridResized(x, y) => (x, y),
        };

        self.add_to_cell(entity, *body, cell_x, cell_y);
    }

    fn add_to_cell(&mut self, entity: EntityId, body: BodyComponent, cell_x: usize, cell_y: usize) {
        self.grid[cell_y * self.nb_cells_x + cell_x].push((entity, body));
    }

    fn purge_deleted_bodies(&mut self) {
        for bodies in self.grid.iter_mut() {
            bodies.retain(|(_, b)| b.w() != 0.0 || b.h() != 0.0);
        }
    }

    fn iter_closest(
        &mut self,
        entity: EntityId,
        body: &BodyComponent,
        max_taxicab_distance: f64,
    ) -> ClosestEntityIterator {
        // I use taxical distance because it's faster to compute than euclidian distance,
        // and an approximation is enough for now.
        ClosestEntityIterator::new(self, entity, body, max_taxicab_distance)
    }
}

// a, b are 2D points.
fn edge_collides_body(a: (f64, f64), b: (f64, f64), body: &BodyComponent) -> bool {
    /*
     *   A       M ---- N
     *    \      |      |
     *     \     |      |
     *      \    O ---- P
     *       \
     *        B
     */
    let half_w = body.w() / 2.0;
    let half_h = body.h() / 2.0;
    let m = (body.x() - half_w, body.y() - half_h);
    let n = (body.x() + half_w, body.y() - half_h);
    let o = (body.x() - half_w, body.y() + half_h);
    let p = (body.x() + half_w, body.y() + half_h);

    let edge_inside_square = f64::min(a.0, b.0) >= m.0
        && f64::max(a.0, b.0) <= n.0
        && f64::min(a.1, b.1) >= m.1
        && f64::max(a.1, b.1) <= o.1;

    edge_inside_square
        || edge_collides_edge(a, b, m, n)
        || edge_collides_edge(a, b, n, p)
        || edge_collides_edge(a, b, p, o)
        || edge_collides_edge(a, b, o, m)
}

fn edge_collides_edge(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> bool {
    /*
     * a, b, c, d are points. First edge is [a, b], second edge is [c, d]
     *
     * Lines equations :
     * line 1 : U = A + u1 * (B - A)
     * line 2 : U = C + u2 * (D - C)
     *
     * - A, B, C, D, U are 2D points, u1 and u2 are scalar
     * - U is the point of intersection
     * - The edges intersect if U is in the edges (u1 & u2 in [0.0, 1.0])
     */

    let u1 = ((d.0 - c.0) * (a.1 - c.1) - (d.1 - c.1) * (a.0 - c.0))
        / ((d.1 - c.1) * (b.0 - a.0) - (d.0 - c.0) * (b.1 - a.1));
    let u2 = ((b.0 - a.0) * (a.1 - c.1) - (b.1 - a.1) * (a.0 - c.0))
        / ((d.1 - c.1) * (b.0 - a.0) - (d.0 - c.0) * (b.1 - a.1));

    (0.0..=1.0).contains(&u1) && (0.0..=1.0).contains(&u2)
}

pub fn try_translate(
    entity: EntityId,
    target_entity: EntityId,
    body: &BodyComponent,
    offset_x: f64,
    offset_y: f64,
) -> bool {
    BODY_GRID
        .with_borrow_mut(|grid| grid.try_translate(entity, target_entity, body, offset_x, offset_y))
}

pub fn collides(entity: EntityId, body: &BodyComponent) -> bool {
    BODY_GRID.with_borrow_mut(|grid| grid.collides(entity, body))
}

pub fn collides_except_target(
    entity: EntityId,
    target_entity: EntityId,
    body: &BodyComponent,
) -> bool {
    BODY_GRID.with_borrow_mut(|grid| grid.collides_except_target(entity, target_entity, body))
}

pub fn edge_collides(
    position_a: (f64, f64),
    position_b: (f64, f64),
    entity: EntityId,
    target_entity: EntityId,
    margin: (f64, f64),
) -> bool {
    BODY_GRID.with_borrow_mut(|grid| {
        grid.edge_collides(position_a, position_b, entity, target_entity, margin)
    })
}

pub fn add(entity: EntityId, body: &BodyComponent) {
    BODY_GRID.with_borrow_mut(|grid| grid.add(entity, body));
}

pub fn delete(entity: EntityId, body: &BodyComponent) {
    BODY_GRID.with_borrow_mut(|grid| grid.delete(entity, body));
}

pub fn purge_deleted_bodies() {
    BODY_GRID.with_borrow_mut(|grid| grid.purge_deleted_bodies());
}

pub fn coords() -> (f64, f64, f64, f64, f64, usize, usize) {
    BODY_GRID.with_borrow(|grid| {
        (
            grid.x,
            grid.y,
            grid.w,
            grid.h,
            grid.cell_size,
            grid.nb_cells_x,
            grid.nb_cells_y,
        )
    })
}

pub fn get_cell_coords(x: f64, y: f64) -> (usize, usize) {
    BODY_GRID.with_borrow_mut(|grid| grid.get_cell_coords(x, y))
}

pub fn iter_closest(
    entity: EntityId,
    body: &BodyComponent,
    max_taxicab_distance: f64,
) -> ClosestEntityIterator {
    BODY_GRID.with_borrow_mut(|grid| grid.iter_closest(entity, body, max_taxicab_distance))
}
