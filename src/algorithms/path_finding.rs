use crate::algorithms::rng;
use crate::components::body_component::BodyComponent;
use crate::constants::NB_PRM_POSITIONS_GENERATED;
use crate::ecs::EntityId;
use crate::shared_data::body_grid;
use ordered_float::OrderedFloat;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash, Copy, Clone)]
pub struct Node {
    x: OrderedFloat<f64>,
    y: OrderedFloat<f64>,
    is_cell_center: bool,
}

impl Node {
    pub fn new(x: f64, y: f64) -> Self {
        Node {
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            is_cell_center: false,
        }
    }

    pub fn new_cell_center(x: f64, y: f64) -> Self {
        Node {
            x: OrderedFloat(x),
            y: OrderedFloat(y),
            is_cell_center: true,
        }
    }

    pub fn x(&self) -> f64 {
        self.x.into_inner()
    }

    pub fn y(&self) -> f64 {
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

    fn to_x(&self, cell_x: isize) -> f64 {
        (cell_x as f64) * self.grid_cell_size + self.offset_x
    }

    fn to_y(&self, cell_y: isize) -> f64 {
        (cell_y as f64) * self.grid_cell_size + self.offset_y
    }
}

#[derive(Clone)]
pub struct Graph {
    neighbours: HashMap<Node, Vec<Node>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            neighbours: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.neighbours.clear();
    }

    pub fn neighbours(&self) -> &HashMap<Node, Vec<Node>> {
        &self.neighbours
    }

    pub fn neighbours_of(&self, n: &Node) -> Vec<Node> {
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
        target_entity: EntityId,
        goal_x: f64,
        goal_y: f64,
    ) {
        // Get the coordinates of the start and goal cells in the body grid, while making sure that
        // the grid is resized to hold both, so that coordinates are not invalidated by the resize.
        body_grid::get_cell_coords(start_x, start_y);
        let (g_cell_x, g_cell_y) = body_grid::get_cell_coords(goal_x, goal_y);
        let (s_cell_x, s_cell_y) = body_grid::get_cell_coords(start_x, start_y);
        let (grid_x, grid_y, _, _, grid_cell_size, nb_cells_x, nb_cells_y) = body_grid::coords();

        // Get the bounds of a rectangle that contains the start and goal cells in the body grid,
        // with a margin (to allow going around obstacles close to start or goal)
        let margin_nb_cells = 1;
        let min_x = isize::max(
            0,
            isize::min(s_cell_x as isize, g_cell_x as isize) - margin_nb_cells,
        );
        let max_x = isize::min(
            nb_cells_x as isize - 1,
            isize::max(s_cell_x as isize, g_cell_x as isize) + margin_nb_cells,
        );
        let min_y = isize::max(
            0,
            isize::min(s_cell_y as isize, g_cell_y as isize) - margin_nb_cells,
        );
        let max_y = isize::min(
            nb_cells_y as isize - 1,
            isize::max(s_cell_y as isize, g_cell_y as isize) + margin_nb_cells,
        );

        // Create the nodes corresponding to the center of the cells, and for each store a list of
        // their 4 adjacent neigbhors (up, down, left, right)
        let coords = Grid2CenterCoordConvertor::new(grid_x, grid_y, grid_cell_size);
        for cell_x in min_x..(max_x + 1) {
            let cell_center_x = coords.to_x(cell_x);

            for cell_y in min_y..(max_y + 1) {
                let cell_center_y = coords.to_y(cell_y);

                // Add the node to graph only if the corresponding cell is not colliding anything
                let cell_body = BodyComponent::new_traversable(
                    cell_center_x,
                    cell_center_y,
                    grid_cell_size,
                    grid_cell_size,
                );
                if body_grid::collides_except_target(entity, target_entity, &cell_body) {
                    continue;
                }

                // Compute the neighbours nodes & add them to the graph if not colliding anything
                let mut neighbours = Vec::with_capacity(4);

                // Left node
                if cell_x > 0 {
                    add_neighbour_cell_center(
                        entity,
                        target_entity,
                        cell_center_x - grid_cell_size,
                        cell_center_y,
                        grid_cell_size,
                        &mut neighbours,
                    );
                }
                // Right node
                if cell_x < max_x {
                    add_neighbour_cell_center(
                        entity,
                        target_entity,
                        cell_center_x + grid_cell_size,
                        cell_center_y,
                        grid_cell_size,
                        &mut neighbours,
                    );
                }
                // Up node
                if cell_y > 0 {
                    add_neighbour_cell_center(
                        entity,
                        target_entity,
                        cell_center_x,
                        cell_center_y - grid_cell_size,
                        grid_cell_size,
                        &mut neighbours,
                    );
                }
                // Down node
                if cell_y < max_y {
                    add_neighbour_cell_center(
                        entity,
                        target_entity,
                        cell_center_x,
                        cell_center_y + grid_cell_size,
                        grid_cell_size,
                        &mut neighbours,
                    );
                }

                self.neighbours.insert(
                    Node::new_cell_center(cell_center_x, cell_center_y),
                    neighbours,
                );
            }
        }
    }

    // PRM-like algorithm to generate a graph of nodes around a position in which the body can
    // safely navigate without colliding anything.
    pub fn add_prm_nodes(
        &mut self,
        entity: EntityId,
        target_entity: EntityId,
        body: &BodyComponent,
        center_x: f64,
        center_y: f64,
    ) -> bool {
        // If the target position already collides, it will be impossible to find a path, so quit
        let temp_body = BodyComponent::new_traversable(center_x, center_y, body.w(), body.h());
        if body_grid::collides_except_target(entity, target_entity, &temp_body) {
            return false;
        }

        // Add the target position as a node
        self.neighbours
            .insert(Node::new(center_x, center_y), Vec::new());

        // In a radius (squared) around the target, randomly generate positions
        let r = graph_connection_radius();
        for _ in 0..NB_PRM_POSITIONS_GENERATED {
            let x = rng::random_range(center_x - r, center_x + r);
            let y = rng::random_range(center_y - r, center_y + r);
            let node = Node::new(x, y);
            self.neighbours.entry(node).or_default();
        }

        true
    }

    // Connect these positions if the resulting edge does not intersect anything
    // (to check this, inflate the bodies size by size of the entity to move)
    pub fn connect_nodes(
        &mut self,
        entity: EntityId,
        target_entity: EntityId,
        body: &BodyComponent,
    ) -> bool {
        let mut at_least_one_edge = false;

        let r = graph_connection_radius();
        let max_d = max_distance_for_connected_dots(r, NB_PRM_POSITIONS_GENERATED);
        let max_d2 = max_d.powi(2);
        let nodes: Vec<Node> = self.neighbours.keys().cloned().collect();

        for i in 0..nodes.len() {
            for j in (i + 1)..nodes.len() {
                let a = &nodes[i];
                let b = &nodes[j];
                if !(a.is_cell_center && b.is_cell_center) // cell centers are already connected
                    && square_euclidian_distance(a, b) < max_d2
                    && !body_grid::edge_collides(
                        (a.x(), a.y()),
                        (b.x(), b.y()),
                        entity,
                        target_entity,
                        (body.w(), body.h()),
                    )
                {
                    self.neighbours.get_mut(a).unwrap().push(*b);
                    self.neighbours.get_mut(b).unwrap().push(*a);
                    at_least_one_edge = true;
                }
            }
        }

        at_least_one_edge
    }
}

fn max_distance_for_connected_dots(r: f64, n: usize) -> f64 {
    /* Expected distance from a point to its neigbors in homogeneous spatial poisson process:
     * 1 / (2 * sqrt(lambda)) where lambda is the density.
     * Inside a disk lambda is N / (pi * r**2).
     * With N the number of points.
     * So the expected distance is (sqrt(pi) / 2) * (r / sqrt(N))
     */
    let half_pi_sqrt = 0.886;
    (r * half_pi_sqrt) / (n as f64).sqrt() * 5.0 // arbitrary factor
}

fn graph_connection_radius() -> f64 {
    let (_, _, _, _, grid_cell_size, ..) = body_grid::coords();
    1.1 * grid_cell_size
}

// Use the cell body to make sure that the whole cell is empty and does not collides anything,
// and make sure that we don't detect a collision with the entity for which we are looking for
// path.
fn add_neighbour_cell_center(
    entity: EntityId,
    target_entity: EntityId,
    cell_center_x: f64,
    cell_center_y: f64,
    grid_cell_size: f64,
    neighbours: &mut Vec<Node>,
) {
    let cell_body = BodyComponent::new_traversable(
        cell_center_x,
        cell_center_y,
        grid_cell_size,
        grid_cell_size,
    );
    if !body_grid::collides_except_target(entity, target_entity, &cell_body) {
        neighbours.push(Node::new_cell_center(cell_center_x, cell_center_y));
    }
}

// Apply the A* algorithm to find a path
pub fn find_reverse_path(graph: &Graph, start: Node, goal: Node) -> Option<Vec<Node>> {
    // Set of discovered nodes
    let mut open_list: Vec<(Node, OrderedFloat<f64>)> = Vec::new();
    open_list.push((
        start,
        OrderedFloat(square_euclidian_distance(&start, &goal)),
    ));

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

        // Note: when estimating distances, I assume that the graph is mostly a "grid"
        // => Hence the usage of manhattan distance
        // For short paths, this is not true (because of graph partly constructed with PRM-like algo)
        for v in graph.neighbours_of(&u).iter() {
            // Check if this path is better than any previous one that passes through v.
            // To do this, compute the length of the path from start to v.
            let try_g_cost = *g_cost.get(&u).unwrap() + square_euclidian_distance(&u, v);
            let g_cost_v = g_cost.get(v);
            if g_cost_v.is_none() || try_g_cost < *g_cost_v.unwrap() {
                // Best path through v ! Estimate total distance to the goal
                let f_score = try_g_cost + square_euclidian_distance(v, &goal);

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

fn square_euclidian_distance(a: &Node, b: &Node) -> f64 {
    (a.x() - b.x()).powi(2) + (a.y() - b.y()).powi(2)
}

// Accurate for distances in a "grid" graph where you can only go in the 4 cardinal directions
#[allow(dead_code)]
fn manhattan_distance(a: &Node, b: &Node) -> f64 {
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
