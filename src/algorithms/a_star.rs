use crate::components::body_component::BodyComponent;
use crate::ecs::{self, EntityId};
use crate::shared_data::body_grid;
use ordered_float::OrderedFloat;
use std::cmp;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct Node {
    x: OrderedFloat<f64>,
    y: OrderedFloat<f64>,
}

impl Node {
    pub fn get_x(&self) -> f64 {
        self.x.into_inner()
    }

    pub fn get_y(&self) -> f64 {
        self.y.into_inner()
    }
}

// Helper to convert grid coordinates (index of cell on x & y axis)
// into simulation coordinates of the center of the cell
struct Grid2CenterCoordConvertor {
    grid_cell_size: f64,
    offset_x: f64,
    offset_y: f64,
}

impl Grid2CenterCoordConvertor {
    fn new(grid_x: f64, grid_y: f64, grid_cell_size: f64) -> Self {
        Grid2CenterCoordConvertor {
            grid_cell_size,
            offset_x: grid_x + grid_cell_size / 2.0,
            offset_y: grid_y + grid_cell_size / 2.0,
        }
    }

    fn to_x(&self, cell_x: isize) -> OrderedFloat<f64> {
        OrderedFloat((cell_x as f64) * self.grid_cell_size + self.offset_x)
    }

    fn to_y(&self, cell_y: isize) -> OrderedFloat<f64> {
        OrderedFloat((cell_y as f64) * self.grid_cell_size + self.offset_y)
    }
}

pub struct Graph {
    neighbours: HashMap<Node, Vec<Node>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            neighbours: HashMap::new(),
        }
    }

    pub fn get_neighbours(&self, n: &Node) -> Vec<Node> {
        if let Some(neighbours) = self.neighbours.get(n) {
            neighbours.clone()
        } else {
            Vec::new()
        }
    }

    // Return the start node and goal node (the center of the 2 body grid cells containing their
    // respective positions)
    pub fn add_body_grid_nodes(
        &mut self,
        entity: EntityId,
        start_x: f64,
        start_y: f64,
        goal_x: f64,
        goal_y: f64,
    ) -> (Node, Node) {
        // Get the coordinates of the start and goal cells in the body grid, while making sure that
        // the grid is resized to hold both, so that coordinates are not invalidated by the resize.
        body_grid::get_cell_coords_with_resize(start_x, start_y);
        let (g_cell_x, g_cell_y) = body_grid::get_cell_coords_with_resize(goal_x, goal_y);
        let (s_cell_x, s_cell_y) = body_grid::get_cell_coords_with_resize(start_x, start_y);
        let (grid_x, grid_y, _, _, grid_cell_size, nb_cells_x, nb_cells_y) =
            body_grid::get_coords();

        // Get the bounds of a rectangle that contains the start and goal cells in the body grid,
        // with a margin (to allow going around obstacles close to start or goal)
        let margin_nb_cells = 1;
        let min_x = cmp::max(
            0,
            cmp::min(s_cell_x as isize, g_cell_x as isize) - margin_nb_cells,
        );
        let max_x = cmp::min(
            nb_cells_x as isize - 1,
            cmp::max(s_cell_x as isize, g_cell_x as isize) + margin_nb_cells,
        );
        let min_y = cmp::max(
            0,
            cmp::min(s_cell_y as isize, g_cell_y as isize) - margin_nb_cells,
        );
        let max_y = cmp::min(
            nb_cells_y as isize - 1,
            cmp::max(s_cell_y as isize, g_cell_y as isize) + margin_nb_cells,
        );

        // Create the nodes corresponding to the center of the cells, and for each store a list of
        // their 4 adjacent neigbhors (up, down, left, right)
        let coords = Grid2CenterCoordConvertor::new(grid_x, grid_y, grid_cell_size);
        for cell_x in min_x..(max_x + 1) {
            let cell_center_x = coords.to_x(cell_x);

            for cell_y in min_y..(max_y + 1) {
                let cell_center_y = coords.to_y(cell_y);

                // Add the node to graph only if the corresponding cell is not colliding anything
                // But always add the starting cell (TODO until PRM is implemented)
                if cell_x as usize != s_cell_x && cell_y as usize != s_cell_y {
                    let cell_body = BodyComponent::new_traversable(
                        *cell_center_x,
                        *cell_center_y,
                        grid_cell_size,
                        grid_cell_size,
                    );
                    if body_grid::collides_in_surronding_cells(entity, &cell_body) {
                        continue;
                    }
                }

                // Compute the neighbours nodes & add them to the graph if not colliding anything
                let mut neighbours = Vec::with_capacity(4);

                // Left node
                if cell_x > 0 {
                    add_to_neighbour_if_ok(
                        entity,
                        cell_x - 1,
                        cell_y,
                        *cell_center_x - grid_cell_size,
                        *cell_center_y,
                        grid_cell_size,
                        &mut neighbours,
                        &coords,
                    );
                }
                // Right node
                if cell_x < max_x {
                    add_to_neighbour_if_ok(
                        entity,
                        cell_x + 1,
                        cell_y,
                        *cell_center_x + grid_cell_size,
                        *cell_center_y,
                        grid_cell_size,
                        &mut neighbours,
                        &coords,
                    );
                }
                // Up node
                if cell_y > 0 {
                    add_to_neighbour_if_ok(
                        entity,
                        cell_x,
                        cell_y - 1,
                        *cell_center_x,
                        *cell_center_y - grid_cell_size,
                        grid_cell_size,
                        &mut neighbours,
                        &coords,
                    );
                }
                // Down node
                if cell_y < max_y {
                    add_to_neighbour_if_ok(
                        entity,
                        cell_x,
                        cell_y + 1,
                        *cell_center_x,
                        *cell_center_y + grid_cell_size,
                        grid_cell_size,
                        &mut neighbours,
                        &coords,
                    );
                }

                self.neighbours.insert(
                    Node {
                        x: cell_center_x,
                        y: cell_center_y,
                    },
                    neighbours,
                );
            }
        }

        (
            // Start node
            Node {
                x: coords.to_x(s_cell_x as isize),
                y: coords.to_y(s_cell_y as isize),
            },
            // Goal node
            Node {
                x: coords.to_x(g_cell_x as isize),
                y: coords.to_y(g_cell_y as isize),
            },
        )
    }
}

fn add_to_neighbour_if_ok(
    entity: EntityId,
    cell_x: isize,
    cell_y: isize,
    cell_center_x: f64,
    cell_center_y: f64,
    grid_cell_size: f64,
    neighbours: &mut Vec<Node>,
    coords: &Grid2CenterCoordConvertor,
) {
    let cell_body = BodyComponent::new_traversable(
        cell_center_x,
        cell_center_y,
        grid_cell_size,
        grid_cell_size,
    );
    // Use the cell body to make sure that the whole cell is empty and does not collides anything,
    // and make sure that we don't detect a collision with the entity for which we are looking for
    // path.
    if !body_grid::collides_in_surronding_cells(entity, &cell_body) {
        neighbours.push(Node {
            x: coords.to_x(cell_x),
            y: coords.to_y(cell_y),
        });
    }
}

pub fn find_reverse_path(graph: &Graph, start: Node, goal: Node) -> Option<Vec<Node>> {
    // Set of discovered nodes
    let mut open_list: Vec<(Node, OrderedFloat<f64>)> = Vec::new();
    open_list.push((start, OrderedFloat(distance(&start, &goal))));

    let mut came_from: HashMap<Node, Node> = HashMap::new();

    let mut g_cost: HashMap<Node, f64> = HashMap::new();
    g_cost.insert(start, 0.0);

    while !open_list.is_empty() {
        // Get the node with the best score (estimated distance to goal with the current path)
        let (u_index, u): (usize, Node) = open_list
            .iter()
            .enumerate()
            .min_by(|x, y| x.1 .1.cmp(&y.1 .1))
            .map(|x| (x.0, x.1 .0))
            .unwrap();
        open_list.remove(u_index);

        // Goal reached, return this path (the best one found yet)
        if u.x == goal.x && u.y == goal.y {
            return Some(reconstruct_path(&came_from, &u));
        }

        for v in graph.get_neighbours(&u).iter() {
            // Check if this path is better than any previous one that passes through v.
            // To do this, compute the length of the path from start to v.
            let try_g_cost = *g_cost.get(&u).unwrap() + distance(&u, v);
            let g_cost_v = g_cost.get(v);
            if g_cost_v.is_none() || try_g_cost < *g_cost_v.unwrap() {
                // Best path through v ! Estimate total distance to the goal
                let f_score = try_g_cost + distance(v, &goal);

                // Store (or update if v already known) the estimated distance
                came_from.insert(*v, u);
                g_cost.insert(*v, try_g_cost);
                if let Some((_, f_score_in_list)) =
                    open_list.iter_mut().find(|x| x.0.x == v.x && x.0.y == v.y)
                {
                    *f_score_in_list = OrderedFloat(f_score);
                } else {
                    open_list.push((*v, OrderedFloat(f_score)));
                }
            }
        }
    }

    None
}

fn distance(a: &Node, b: &Node) -> f64 {
    (a.x.into_inner() - b.x.into_inner()).abs() + (a.y.into_inner() - b.y.into_inner()).abs()
}

fn reconstruct_path(came_from: &HashMap<Node, Node>, node: &Node) -> Vec<Node> {
    let mut reverse_path = vec![*node];
    let mut n = node;
    while came_from.contains_key(n) {
        n = came_from.get(n).unwrap();
        reverse_path.push(*n);
    }
    reverse_path
}
