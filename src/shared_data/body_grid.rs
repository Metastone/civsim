use crate::components::body_component::BodyComponent;
use crate::constants::*;
use crate::ecs::EntityId;
use std::cell::RefCell;
use std::cmp;

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
            CREATURE_PIXEL_SIZE as f64
        )
    )
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
        let max_entity_size = CREATURE_PIXEL_SIZE as f64;
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
                cmp::max(
                    nb_cells * 2,
                    (coord_in_grid.abs() / self.cell_size) as usize + 1 + nb_cells,
                ),
            )
        } else if coord_in_grid >= size {
            (
                true,
                0,
                cmp::max(nb_cells * 2, (coord_in_grid / self.cell_size) as usize + 1),
            )
        } else {
            (false, 0, 0)
        }
    }

    /* Return true if the body was translated successfully (no collision)
     * otherwise, return false and do nothing
     */
    fn try_translate(
        &mut self,
        entity: EntityId,
        body: &BodyComponent,
        offset_x: f64,
        offset_y: f64,
    ) -> bool {
        let translated_body = BodyComponent::new_traversable(
            body.get_x() + offset_x,
            body.get_y() + offset_y,
            body.get_w(),
            body.get_h(),
        );

        if !self.collides_in_surronding_cells(entity, &translated_body) {
            self.translate(entity, body, translated_body);
            return true;
        }
        false
    }

    fn collides_in_surronding_cells(&mut self, entity: EntityId, body: &BodyComponent) -> bool {
        let (body_cell_x, body_cell_y) = match self.get_cell_coords_impl(body.get_x(), body.get_y())
        {
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
                    let is_deleted_body = b.get_w() == 0.0 && b.get_h() == 0.0;
                    let is_itself = *e == entity;
                    if is_deleted_body || is_itself {
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

    // a, b are 2D points.
    fn edge_collides(
        &mut self,
        a: (f64, f64),
        b: (f64, f64),
        entity: EntityId,
        margin: (f64, f64),
    ) -> bool {
        let (a_cx, a_cy) = self.get_cell_coords(a.0, a.1);
        let (b_cx, b_cy) = self.get_cell_coords(b.0, b.1);

        // Get the bounds of a rectangle that contains the 2D points A and B (vertices of the
        // edge), with a margin
        let margin_cx = (margin.0 / self.cell_size) as isize + 1;
        let margin_cy = (margin.1 / self.cell_size) as isize + 1;
        let min_x = cmp::max(0, cmp::min(a_cx as isize, b_cx as isize) - margin_cx);
        let max_x = cmp::min(
            self.nb_cells_x as isize - 1,
            cmp::max(a_cx as isize, b_cx as isize) + margin_cx,
        );
        let min_y = cmp::max(0, cmp::min(a_cy as isize, b_cy as isize) - margin_cy);
        let max_y = cmp::min(
            self.nb_cells_y as isize - 1,
            cmp::max(a_cy as isize, b_cy as isize) + margin_cy,
        );

        for cx in min_x..(max_x + 1) {
            for cy in min_y..(max_y + 1) {
                for (e, body) in self.grid[cy as usize * self.nb_cells_x + cx as usize].iter() {
                    let is_deleted_body = body.get_w() == 0.0 && body.get_h() == 0.0;
                    let is_itself = *e == entity;
                    if is_deleted_body || is_itself {
                        continue;
                    }
                    let inflated_body = BodyComponent::new_traversable(
                        body.get_x(),
                        body.get_y(),
                        body.get_w() + margin.0,
                        body.get_h() + margin.1,
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
            self.get_cell_coords_impl(body.get_x(), body.get_y()),
            self.get_cell_coords_impl(translated_body.get_x(), translated_body.get_y()),
        ) {
            (GetCoordsResult::Ok(x, y), GetCoordsResult::Ok(tx, ty)) => (x, y, tx, ty),
            _ => {
                // The grid was resized, re-compute the coords to be sure they are both valid
                let (x, y) = match self.get_cell_coords_impl(body.get_x(), body.get_y()) {
                    GetCoordsResult::Ok(x, y) => (x, y),
                    GetCoordsResult::GridResized(x, y) => (x, y),
                };
                let (tx, ty) = match self
                    .get_cell_coords_impl(translated_body.get_x(), translated_body.get_y())
                {
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
        let (cell_x, cell_y) = match self.get_cell_coords_impl(body.get_x(), body.get_y()) {
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
        let (cell_x, cell_y) = match self.get_cell_coords_impl(body.get_x(), body.get_y()) {
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
            bodies.retain(|(_, b)| b.get_w() != 0.0 || b.get_h() != 0.0);
        }
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
    let m = (body.get_x(), body.get_y());
    let n = (body.get_x() + body.get_w(), body.get_y());
    let o = (body.get_x(), body.get_y() + body.get_h());
    let p = (body.get_x() + body.get_w(), body.get_y() + body.get_h());

    edge_collides_edge(a, b, m, n)
        || edge_collides_edge(a, b, n, p)
        || edge_collides_edge(a, b, p, o)
        || edge_collides_edge(a, b, o, m)
}

// a, b, c, d are points. First edge is [a, b], second edge is [c, d]
fn edge_collides_edge(a: (f64, f64), b: (f64, f64), c: (f64, f64), d: (f64, f64)) -> bool {
    // Lines equations (A, B, C, D, U are 2D points, u1 and u2 are scalar):
    // line 1 : U = A + u1 * (B - A)
    // line 2 : U = C + u2 * (D - C)

    let u1 = ((d.0 - c.0) * (a.1 - c.1) - (d.1 - c.1) * (a.0 - c.0))
        / ((d.1 - c.1) * (b.0 - a.0) - (d.0 - c.0) * (b.1 - a.1));
    let u2 = ((b.0 - a.0) * (a.1 - c.1) - (b.1 - a.1) * (a.0 - c.0))
        / ((d.1 - c.1) * (b.0 - a.0) - (d.0 - c.0) * (b.1 - a.1));

    (0.0..=1.0).contains(&u1) || (0.0..=1.0).contains(&u2)
}

pub fn try_translate(entity: EntityId, body: &BodyComponent, offset_x: f64, offset_y: f64) -> bool {
    BODY_GRID.with_borrow_mut(|grid| grid.try_translate(entity, body, offset_x, offset_y))
}

pub fn collides(entity: EntityId, body: &BodyComponent) -> bool {
    BODY_GRID.with_borrow_mut(|grid| grid.collides_in_surronding_cells(entity, body))
}

pub fn edge_collides(
    position_a: (f64, f64),
    position_b: (f64, f64),
    entity: EntityId,
    margin: (f64, f64),
) -> bool {
    BODY_GRID.with_borrow_mut(|grid| grid.edge_collides(position_a, position_b, entity, margin))
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

pub fn get_coords() -> (f64, f64, f64, f64, f64, usize, usize) {
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
