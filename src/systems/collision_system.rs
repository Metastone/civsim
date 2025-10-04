use crate::components::body_component::{self, BodyComponent};
use crate::ecs::{Ecs, EntityId, System};
use std::any::TypeId;

struct Grid {
    // Coordinates of the space occupied by all entities,
    // relative to simulation coordinates
    x: f64,
    y: f64,

    nb_cells_x: usize,
    nb_cells_y: usize,

    grid: Vec<Vec<(EntityId, BodyComponent)>>,
}

impl Grid {
    fn new(min_x: f64, max_x: f64, min_y: f64, max_y: f64) -> Self {
        let w = (max_x - min_x).abs();
        let h = (max_y - min_y).abs();
        let half_size = body_component::get_max_half_body_size();
        let nb_cells_x = (w / half_size + 1.0) as usize;
        let nb_cells_y = (h / half_size + 1.0) as usize;
        Grid {
            x: min_x,
            y: min_y,
            nb_cells_x,
            nb_cells_y,
            grid: vec![Vec::new(); nb_cells_x * nb_cells_y],
        }
    }

    fn to_cell_coords(&self, body: &BodyComponent) -> (usize, usize) {
        let half_size = body_component::get_max_half_body_size();
        (
            ((body.x - self.x) / half_size) as usize,
            ((body.y - self.y) / half_size) as usize,
        )
    }

    fn add(&mut self, entity: EntityId, body: BodyComponent) {
        let (cell_x, cell_y) = self.to_cell_coords(&body);
        self.grid[cell_y * self.nb_cells_x + cell_x].push((entity, body));
    }

    fn handle_collision(&self, entity: EntityId, body: &mut BodyComponent) {
        let (body_cell_x, body_cell_y) = self.to_cell_coords(body);
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
                    if *e == entity {
                        continue;
                    }
                    body.if_colliding_step_to_the_side(b);
                }
            }
        }
    }
}

pub struct CollisionSystem;
impl System for CollisionSystem {
    fn run(&self, ecs: &mut Ecs) {
        //        let mut updates: Vec<Update> = Vec::new();

        // Find the bounds of the space occupied by all entities combined
        let mut min_x = 0.0;
        let mut max_x = 0.0;
        let mut min_y = 0.0;
        let mut max_y = 0.0;
        for (body, _) in iter_components!(ecs, BodyComponent) {
            let (w2, h2) = (body.w / 2.0, body.h / 2.0);
            if body.x - w2 < min_x {
                min_x = body.x - w2;
            }
            if body.x + w2 > max_x {
                max_x = body.x + w2;
            }
            if body.y - h2 < min_y {
                min_y = body.y - h2;
            }
            if body.y + h2 > max_y {
                max_y = body.y + h2;
            }
        }

        /* Construct a collision computation grid.
         * We assume that for a given entity, only the surronding cells will be checked for
         * collision.
         * Hence, to be sure to not miss a collision the cells must be at least as big as half the
         * length of the biggest entity.
         */
        let mut grid = Grid::new(min_x, max_x, min_y, max_y);
        for (body, info) in iter_components!(ecs, BodyComponent) {
            grid.add(info.entity, *body);
        }

        /* Move each "colliding" entity to a safe position within the cell & surronding cells, if
         * possible
         */
        for (body, info) in iter_components!(ecs, BodyComponent) {
            grid.handle_collision(info.entity, body);
        }

        //        ecs.apply(updates);
    }
}
