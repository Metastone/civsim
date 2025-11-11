use crate::components::body_component::BodyComponent;
use crate::constants::*;
use std::cell::RefCell;

/* Collision computation grid.
 *
 * For a given entity, only the surronding cells will be checked for collision.
 * Hence, to be sure to not miss a collision, the cells must be at least as big
 * as the biggest entity.
 *
 * By construction bodies never collide, so when checking collision for a body within a cell,
 * we can identify the body by its position which is necessarily unique.
 *
 * For quick access I use vecs for storage. Deleted bodies are recognized with w = 0 && h == 0,
 * and they are deleted later during the next periodic grid cleanup.
 */

// TODO make it private
thread_local! {
    pub static BODY_GRID: RefCell<BodyGrid> = RefCell::new(
        BodyGrid::new(
            - (SCREEN_WIDTH as f64) / 2.0,
            SCREEN_WIDTH as f64 / 2.0,
            - (SCREEN_HEIGHT as f64) / 2.0,
            SCREEN_HEIGHT as f64 / 2.0,
            CREATURE_PIXEL_SIZE as f64
        )
    )
}

pub struct BodyGrid {
    // Coordinates of the space occupied by all entities,
    // relative to simulation coordinates
    x: f64,
    y: f64,

    cell_size: f64,

    nb_cells_x: usize,
    nb_cells_y: usize,

    grid: Vec<Vec<BodyComponent>>,
}

impl BodyGrid {
    // TODO update grid size on body translation

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
            cell_size,
            nb_cells_x,
            nb_cells_y,
            grid: vec![Vec::new(); nb_cells_x * nb_cells_y],
        }
    }

    fn to_cell_coords(&self, body: &BodyComponent) -> (usize, usize) {
        (
            ((body.get_x() - self.x) / self.cell_size) as usize,
            ((body.get_y() - self.y) / self.cell_size) as usize,
        )
    }

    /* Return true if the body was translated successfully (no collision)
     * otherwise, return false and do nothing
     */
    pub fn try_translate(&mut self, body: &BodyComponent, offset_x: f64, offset_y: f64) -> bool {
        let translated_body = BodyComponent::from(
            body.get_x() + offset_x,
            body.get_y() + offset_y,
            body.get_w(),
            body.get_h(),
        );

        if !self.collides_in_surronding_cells(body, &translated_body) {
            self.translate(body, translated_body);
            return true;
        }
        false
    }

    pub fn collides_in_surronding_cells(
        &self,
        body: &BodyComponent,
        translated_body: &BodyComponent,
    ) -> bool {
        let (t_body_cell_x, t_body_cell_y) = self.to_cell_coords(translated_body);
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
                for b in self.grid[cell_y as usize * self.nb_cells_x + cell_x as usize].iter() {
                    let is_deleted_body = b.get_w() == 0.0 && b.get_h() == 0.0;
                    let is_itself = b.get_x() == body.get_x() && b.get_y() == body.get_y();
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

    pub fn translate(&mut self, body: &BodyComponent, translated_body: BodyComponent) {
        let (cell_x, cell_y) = self.to_cell_coords(body);
        let (t_cell_x, t_cell_y) = self.to_cell_coords(&translated_body);
        if cell_x == t_cell_x && cell_y == t_cell_y {
            self.update(body, translated_body, cell_x, cell_y);
        } else {
            self.delete_from_cell(body, cell_x, cell_y);
            self.add_to_cell(translated_body, t_cell_x, t_cell_y);
        }
    }

    fn update(
        &mut self,
        body: &BodyComponent,
        translated_body: BodyComponent,
        cell_x: usize,
        cell_y: usize,
    ) {
        for b in self.grid[cell_y * self.nb_cells_x + cell_x].iter_mut() {
            if b.get_x() == body.get_x() && b.get_y() == body.get_y() {
                *b = translated_body;
                break;
            }
        }
    }

    pub fn delete(&mut self, body: &BodyComponent) {
        let (cell_x, cell_y) = self.to_cell_coords(body);
        self.grid[cell_y * self.nb_cells_x + cell_x]
            .retain(|b| b.get_x() != body.get_x() || b.get_y() != body.get_y());
    }

    fn delete_from_cell(&mut self, body: &BodyComponent, cell_x: usize, cell_y: usize) {
        for b in self.grid[cell_y * self.nb_cells_x + cell_x].iter_mut() {
            if b.get_x() == body.get_x() && b.get_y() == body.get_y() {
                *b = BodyComponent::from(0.0, 0.0, 0.0, 0.0);
            }
        }
    }

    pub fn add(&mut self, body: &BodyComponent) {
        let (cell_x, cell_y) = self.to_cell_coords(body);
        self.add_to_cell(*body, cell_x, cell_y);
    }

    fn add_to_cell(&mut self, body: BodyComponent, cell_x: usize, cell_y: usize) {
        /* I don't check yet if there is already a body exactly at the same position.
         * It is improbable but possible and could lead to bugs.
         *
         * TODO fix this loophole
         */
        self.grid[cell_y * self.nb_cells_x + cell_x].push(body);
    }

    fn purge_deleted_bodies(&mut self) {
        for bodies in self.grid.iter_mut() {
            bodies.retain(|b| b.get_w() != 0.0 || b.get_h() != 0.0);
        }
    }
}

pub fn purge_deleted_bodies() {
    BODY_GRID.with_borrow_mut(|grid| grid.purge_deleted_bodies());
}
