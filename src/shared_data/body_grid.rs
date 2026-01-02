use crate::components::body_component::BodyComponent;
use crate::constants::*;
use crate::ecs::EntityId;
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
        let w = (max_x - min_x).abs();
        let h = (max_y - min_y).abs();
        let nb_cells_x = (w / cell_size + 1.0) as usize;
        let nb_cells_y = (h / cell_size + 1.0) as usize;
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

    fn get_cell_coords_with_resize(&mut self, body: &BodyComponent) -> GetCoordsResult {
        // Get body coordinates relative to the grid
        let x_in_grid = body.get_x() - self.x;
        let y_in_grid = body.get_y() - self.y;

        // Check if a grid resize is required on x axis
        let (offset_cell_x, resize_x): (usize, bool) = if x_in_grid < 0.0 {
            (self.nb_cells_x, true)
        } else if x_in_grid >= self.w {
            (0, true)
        } else {
            (0, false)
        };

        // Check if a grid resize is required on y axis
        let (offset_cell_y, resize_y): (usize, bool) = if y_in_grid < 0.0 {
            (self.nb_cells_y, true)
        } else if y_in_grid >= self.h {
            (0, true)
        } else {
            (0, false)
        };

        // Resize the grid (double its size on the appropriate side on x or y or both)
        if resize_x || resize_y {
            let nb_cells_x = if resize_x {
                self.nb_cells_x * 2
            } else {
                self.nb_cells_x
            };
            let nb_cells_y = if resize_y {
                self.nb_cells_y * 2
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
                (body.get_x() - self.x / self.cell_size) as usize,
                (body.get_y() - self.y / self.cell_size) as usize,
            );
        }

        // No grid resize required
        GetCoordsResult::Ok(
            (x_in_grid / self.cell_size) as usize,
            (y_in_grid / self.cell_size) as usize,
        )
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

    fn collides_in_surronding_cells(
        &mut self,
        entity: EntityId,
        translated_body: &BodyComponent,
    ) -> bool {
        let (t_body_cell_x, t_body_cell_y) = match self.get_cell_coords_with_resize(translated_body)
        {
            GetCoordsResult::Ok(x, y) => (x, y),
            GetCoordsResult::GridResized(x, y) => (x, y),
        };

        for i in -1..2 {
            let cell_x = t_body_cell_x as i64 + i;
            if cell_x < 0 || cell_x >= self.nb_cells_x as i64 {
                continue;
            }
            for j in -1..2 {
                let cell_y = t_body_cell_y as i64 + j;
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
                    if translated_body.collides(b) {
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
            self.get_cell_coords_with_resize(body),
            self.get_cell_coords_with_resize(&translated_body),
        ) {
            (GetCoordsResult::Ok(x, y), GetCoordsResult::Ok(tx, ty)) => (x, y, tx, ty),
            _ => {
                // The grid was resized, re-compute the coords to be sure they are both valid
                let (x, y) = match self.get_cell_coords_with_resize(body) {
                    GetCoordsResult::Ok(x, y) => (x, y),
                    GetCoordsResult::GridResized(x, y) => (x, y),
                };
                let (tx, ty) = match self.get_cell_coords_with_resize(&translated_body) {
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
        let (cell_x, cell_y) = match self.get_cell_coords_with_resize(body) {
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
        let (cell_x, cell_y) = match self.get_cell_coords_with_resize(body) {
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

pub fn try_translate(entity: EntityId, body: &BodyComponent, offset_x: f64, offset_y: f64) -> bool {
    BODY_GRID.with_borrow_mut(|grid| grid.try_translate(entity, body, offset_x, offset_y))
}

pub fn collides_in_surronding_cells(entity: EntityId, translated_body: &BodyComponent) -> bool {
    BODY_GRID.with_borrow_mut(|grid| grid.collides_in_surronding_cells(entity, translated_body))
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

pub fn get_coords() -> (f64, f64, f64, f64, f64) {
    BODY_GRID.with_borrow(|grid| (grid.x, grid.y, grid.w, grid.h, grid.cell_size))
}
