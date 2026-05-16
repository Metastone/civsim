#[cfg(test)]
use std::any::type_name;
use std::{collections::HashMap, mem};

use log::error;
use ordered_float::OrderedFloat;

use crate::{
    components::agent_component::AgentComponent,
    configuration::Config,
    ecs::{Ecs, EntityInfo},
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Symbol {
    Energy,
    IsNearPlant,
    IsNearCorpse,
    IsNearHerbivorous,

    #[cfg(test)]
    HasHouse,
    #[cfg(test)]
    SproutCount,
    #[cfg(test)]
    TreeCount,
    #[cfg(test)]
    WoodCount,
    #[cfg(test)]
    MoneyCount,
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub enum Value {
    F32(OrderedFloat<f32>),
    #[allow(unused)]
    Isize(isize),
    Bool(bool),
}

#[derive(PartialEq, Eq, Debug)]
pub enum Operator {
    Equal,
    #[allow(unused)]
    Less,
    #[allow(unused)]
    LessOrEqual,
    #[allow(unused)]
    Greater,
    GreaterOrEqual,
    #[allow(unused)]
    Not,
}

#[derive(PartialEq, Eq, Debug)]
pub enum Modifier {
    SetValue,
    Increment,
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Fact {
    symbol: Symbol,
    value: Value,
}

impl Fact {
    #[allow(unused)]
    pub fn new(symbol: Symbol, value: Value) -> Self {
        Fact { symbol, value }
    }
}

pub struct Condition {
    symbol: Symbol,
    operator: Operator,
    value: Value,
}

impl Condition {
    pub fn new(symbol: Symbol, operator: Operator, value: Value) -> Self {
        Condition {
            symbol,
            operator,
            value,
        }
    }
}

pub struct Effect {
    symbol: Symbol,
    modifier: Modifier,
    value: Value,
}

impl Effect {
    pub fn new(symbol: Symbol, modifier: Modifier, value: Value) -> Self {
        Effect {
            symbol,
            modifier,
            value,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct WorldState {
    // Perfo: Consider using a hashmap in the future the WorldState becomes large
    facts: Vec<Fact>,
}
impl WorldState {
    pub fn new() -> Self {
        Self { facts: Vec::new() }
    }
}

pub enum ActionResult {
    OnGoing,
    Success,
    Failure,
}

pub trait Action {
    #[cfg(test)]
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    fn preconditions(&self) -> &[Condition];
    fn effects(&self) -> &[Effect];

    fn perform(
        &self,
        ecs: &mut Ecs,
        info: &EntityInfo,
        config: &Config,
    ) -> Result<ActionResult, String>;
}

pub trait Goal {
    fn conditions(&self) -> &[Condition];
    fn utility(&self, ecs: &Ecs, info: &EntityInfo) -> f32;
}

pub struct GoalSet {
    goals: Vec<Box<dyn Goal>>,
}
impl GoalSet {
    pub fn new() -> Self {
        GoalSet { goals: Vec::new() }
    }
    pub fn add(&mut self, goal: Box<dyn Goal>) {
        self.goals.push(goal);
    }
}

pub struct ActionSet {
    actions: Vec<Box<dyn Action>>,
}

impl ActionSet {
    pub fn new() -> Self {
        ActionSet {
            actions: Vec::new(),
        }
    }
    pub fn add(&mut self, goal: Box<dyn Action>) {
        self.actions.push(goal);
    }
    pub fn len(&self) -> usize {
        self.actions.len()
    }
}

/// Goal-Oriented Action Planner
pub struct Goap {
    goal_sets: Vec<GoalSet>,
    action_sets: Vec<ActionSet>,
}

macro_rules! goal_exists_or_return {
    ($self:ident, $goal_set:ident, $goal:ident, $return_value:expr) => {
        goal_set_exists_or_return!($self, $goal_set, $return_value);
        if $goal >= $self.goal_sets[$goal_set].goals.len() {
            error!("No goal with index {} in goal set {}", $goal, $goal_set);
            return $return_value;
        }
    };
}
macro_rules! goal_set_exists_or_return {
    ($self:ident, $goal_set:ident, $return_value:expr) => {
        if $goal_set >= $self.goal_sets.len() {
            error!("No goal set with index {}", $goal_set);
            return $return_value;
        }
    };
}
macro_rules! action_exists_or_return {
    ($self:ident, $action_set:ident, $action:ident, $return_value:expr) => {
        action_set_exists_or_return!($self, $action_set, $return_value);
        if $action >= $self.action_sets[$action_set].actions.len() {
            error!(
                "No action with index {} in action set {}",
                $action, $action_set
            );
            return $return_value;
        }
    };
}
macro_rules! action_set_exists_or_return {
    ($self:ident, $action_set:ident, $return_value:expr) => {
        if $action_set >= $self.action_sets.len() {
            error!("No action set with index {}", $action_set);
            return $return_value;
        }
    };
}

impl Goap {
    pub fn new() -> Self {
        Goap {
            goal_sets: Vec::new(),
            action_sets: Vec::new(),
        }
    }

    pub fn add_goal_set(&mut self, goal_set: GoalSet) -> usize {
        self.goal_sets.push(goal_set);
        self.goal_sets.len() - 1
    }

    pub fn add_action_set(&mut self, action_set: ActionSet) -> usize {
        self.action_sets.push(action_set);
        self.action_sets.len() - 1
    }

    pub fn find_goal(&self, ecs: &Ecs, info: &EntityInfo, goal_set: usize) -> Option<usize> {
        if goal_set >= self.goal_sets.len() {
            error!("No goal set with index {}", goal_set);
            return None;
        }

        let mut found = false;
        let mut best_goal_idx = 0;
        let mut best_utility = 0.0;
        for (idx, goal) in self.goal_sets[goal_set].goals.iter().enumerate() {
            let utility = goal.utility(ecs, info);
            if utility > best_utility {
                found = true;
                best_goal_idx = idx;
                best_utility = utility;
            }
        }
        if found { Some(best_goal_idx) } else { None }
    }

    pub fn compute_plan(
        &self,
        agent: &AgentComponent,
        world_state: &WorldState,
        goal: usize,
        goal_set: usize,
        action_set: usize,
    ) -> Option<Vec<usize>> {
        goal_exists_or_return!(self, goal_set, goal, None);
        action_set_exists_or_return!(self, action_set, None);
        let goal_conditions = self.goal_sets[goal_set].goals[goal].conditions();
        find_path(
            agent,
            world_state,
            goal_conditions,
            &self.action_sets[action_set].actions,
        )
    }

    pub fn perform_action(
        &self,
        ecs: &mut Ecs,
        info: &EntityInfo,
        config: &Config,
        world_state: &mut WorldState,
        action: usize,
        action_set: usize,
    ) -> ActionResult {
        action_exists_or_return!(self, action_set, action, ActionResult::Failure);
        let act = &self.action_sets[action_set].actions[action];
        match act.perform(ecs, info, config) {
            Ok(ActionResult::Success) => {
                apply_effects(act.as_ref(), world_state);
                ActionResult::Success
            }
            Ok(result) => result,
            Err(msg) => {
                error!("{msg}");
                ActionResult::Failure
            }
        }
    }

    #[cfg(test)]
    fn get_action(&self, action_set: usize, action: usize) -> Option<&dyn Action> {
        action_exists_or_return!(self, action_set, action, None);
        Some(self.action_sets[action_set].actions[action].as_ref())
    }
}

/// Apply the A* algorithm to find a path of actions from the initial world state to the goal world state
///
/// # Note about optimisation
/// See note in path_finding (similar A* implementation)
pub fn find_path(
    agent: &AgentComponent,
    start_state: &WorldState,
    goal_conditions: &[Condition],
    action_set: &[Box<dyn Action>],
) -> Option<Vec<usize>> {
    // Set of discovered nodes
    let mut open_list: Vec<(WorldState, OrderedFloat<f64>)> = Vec::new();
    open_list.push((
        start_state.clone(),
        OrderedFloat(distance(start_state, goal_conditions)),
    ));

    let mut came_from: HashMap<WorldState, (WorldState, usize)> = HashMap::new();

    let mut g_cost: HashMap<WorldState, f64> = HashMap::new();
    g_cost.insert(start_state.clone(), 0.0);

    // TODO limit the search depth more properly
    // Currently I search from start to goal, and I try all available actions.
    // Maybe I could try in priority the actions that have an effect that satisfy one of the
    // goal ?
    // Maybe searching from goal to start with a reversed logic would help ? (but then using
    // the ecs to compute the action cost becomes hard)
    let mut i = 0;
    let i_max = 100_000;
    while !open_list.is_empty() && i < i_max {
        // Get the node with the best score (estimated distance to goal with the current path)
        let (u_index, u): (usize, WorldState) = open_list
            .iter()
            .enumerate()
            .min_by(|x, y| x.1.1.cmp(&y.1.1))
            .map(|x| (x.0, x.1.0.clone()))
            .unwrap();
        open_list.remove(u_index);

        // Goal reached, return this path (the best one found yet)
        if validate_conditions(&u, goal_conditions) {
            return Some(reconstruct_path(&came_from, u));
        }

        for (action_index, action) in action_set
            .iter()
            .enumerate()
            .filter(|(_, a)| validate_conditions(&u, a.preconditions()))
        {
            // Agents maintains a custom cost for each action (increased when performing an action
            // fails, to allow alternative plans to emerge)
            let action_cost = agent.get_action_cost(action_index);

            // Create the neighbour v (world state obtained when performing the action)
            let mut v = u.clone();
            apply_effects(action.as_ref(), &mut v);

            // Check if this path is better than any previous one that passes through v.
            // To do this, compute the length of the path from start to v.
            let try_g_cost = *g_cost.get(&u).unwrap() + action_cost;
            let g_cost_v = g_cost.get(&v);
            if g_cost_v.is_none() || try_g_cost < *g_cost_v.unwrap() {
                // Best path through v ! Estimate total distance to the goal
                let f_score = try_g_cost + distance(&v, goal_conditions);

                // Store (or update if v already known) the estimated distance
                came_from.insert(v.clone(), (u.clone(), action_index));
                g_cost.insert(v.clone(), try_g_cost);
                if let Some((_, f_score_in_list)) = open_list.iter_mut().find(|x| x.0 == v) {
                    *f_score_in_list = OrderedFloat(f_score);
                } else {
                    open_list.push((v.clone(), OrderedFloat(f_score)));
                }
            }
        }

        i += 1;
    }
    if i >= i_max {
        error!("No action plan found after {i} iterations");
    }

    None
}

fn distance(state: &WorldState, conditions: &[Condition]) -> f64 {
    let mut d = 0.0;
    for c in conditions {
        if !validates_condition(state, c) {
            d += 1.0;
        }
    }
    d
}

fn validates_condition(world_state: &WorldState, condition: &Condition) -> bool {
    // Remark: We assume that there is only one entry per symbol in the world state.

    for Fact { symbol, value } in &world_state.facts {
        if *symbol != condition.symbol {
            continue;
        }

        // Symbol found in the world state: check the condition
        match (&condition.operator, &condition.value, value) {
            (Operator::Equal, c_v, f_v) => {
                return *f_v == *c_v;
            }
            (Operator::Not, c_v, f_v) => {
                return *f_v != *c_v;
            }
            (Operator::Less, Value::F32(c_v), Value::F32(f_v)) => {
                return *f_v < *c_v;
            }
            (Operator::Less, Value::Isize(c_v), Value::Isize(f_v)) => {
                return *f_v < *c_v;
            }
            (Operator::LessOrEqual, Value::F32(c_v), Value::F32(f_v)) => {
                return *f_v <= *c_v;
            }
            (Operator::LessOrEqual, Value::Isize(c_v), Value::Isize(f_v)) => {
                return *f_v <= *c_v;
            }
            (Operator::Greater, Value::F32(c_v), Value::F32(f_v)) => {
                return *f_v > *c_v;
            }
            (Operator::Greater, Value::Isize(c_v), Value::Isize(f_v)) => {
                return *f_v > *c_v;
            }
            (Operator::GreaterOrEqual, Value::F32(c_v), Value::F32(f_v)) => {
                return *f_v >= *c_v;
            }
            (Operator::GreaterOrEqual, Value::Isize(c_v), Value::Isize(f_v)) => {
                return *f_v >= *c_v;
            }
            _ => {}
        }

        // If we reach this point, it means that something went wrong when checking the condition
        error!(
            "Inconsistency for symbol {:?}
            Fact:      value = {:?}
            Condition: value = {:?}, operator = {:?}",
            symbol, value, condition.value, condition.operator
        );
        return false;
    }

    // No symbol matching the condition found in the world state
    false
}

fn validate_conditions(state: &WorldState, conditions: &[Condition]) -> bool {
    distance(state, conditions) == 0.0
}

fn reconstruct_path(
    came_from: &HashMap<WorldState, (WorldState, usize)>,
    final_state: WorldState,
) -> Vec<usize> {
    let mut path = Vec::new();
    let mut state = final_state;
    while let Some((s, action)) = came_from.get(&state) {
        state = s.clone();
        path.push(*action);
    }
    path.reverse();
    path
}

fn apply_effects(action: &dyn Action, state: &mut WorldState) {
    for Effect {
        symbol,
        modifier,
        value,
    } in action.effects()
    {
        // Look for the symbol in the world state
        let mut symbol_in_state = false;
        for fact in state.facts.iter_mut() {
            if fact.symbol != *symbol {
                continue;
            }

            // Symbol matches, edit the existing symbol in the world state
            match (modifier, value, &mut fact.value) {
                (Modifier::SetValue, e_v, f_v) => {
                    if mem::discriminant(f_v) == mem::discriminant(e_v) {
                        *f_v = e_v.clone();
                        symbol_in_state = true;
                        break;
                    }
                }
                (Modifier::Increment, Value::F32(e_v), Value::F32(f_v)) => {
                    fact.value = Value::F32(*f_v + e_v);
                    symbol_in_state = true;
                    break;
                }
                (Modifier::Increment, Value::Isize(e_v), Value::Isize(f_v)) => {
                    fact.value = Value::Isize(*f_v + e_v);
                    symbol_in_state = true;
                    break;
                }
                _ => {}
            };

            // If we reach this point, it means that something went wrong when editing the already
            // existing fact
            error!(
                "Inconsistency for symbol {:?}\n
                Fact:   value = {:?}\n
                Effect: value = {:?}, modifier = {:?}",
                fact.symbol, fact.value, value, modifier
            );
        }

        // If the symbol is not in the world state, create it.
        // For increment / decrement operations, acts as if there was a zero value and increment /
        // decrement it.
        if !symbol_in_state {
            match (modifier, value) {
                (Modifier::SetValue, v) => state.facts.push(Fact {
                    symbol: symbol.clone(),
                    value: v.clone(),
                }),
                (Modifier::Increment, Value::F32(e_v)) => state.facts.push(Fact {
                    symbol: symbol.clone(),
                    value: Value::F32(*e_v),
                }),
                (Modifier::Increment, Value::Isize(e_v)) => state.facts.push(Fact {
                    symbol: symbol.clone(),
                    value: Value::Isize(*e_v),
                }),
                _ => {}
            };
        }
    }
}

#[cfg(test)]
mod tests {
    use std::any::type_name;

    use crate::{
        components::agent_component::AgentComponent,
        configuration::Config,
        ecs::{Ecs, EntityInfo},
        goap::{
            Action, ActionResult, ActionSet, Condition, Effect, Fact, Goal, GoalSet, Goap,
            Modifier, Operator, Symbol, Value, WorldState,
        },
    };
    macro_rules! define_perform_success {
        () => {
            fn perform(
                &self,
                _ecs: &mut Ecs,
                _info: &EntityInfo,
                _config: &Config,
            ) -> Result<ActionResult, String> {
                Ok(ActionResult::Success)
            }
        };
    }

    struct HaveHouseAndGardenGoal {
        preconditions: [Condition; 2],
    }
    impl HaveHouseAndGardenGoal {
        fn new() -> Self {
            Self {
                preconditions: [
                    Condition::new(Symbol::HasHouse, Operator::Equal, Value::Bool(true)),
                    Condition::new(Symbol::TreeCount, Operator::GreaterOrEqual, Value::Isize(3)),
                ],
            }
        }
    }
    impl Goal for HaveHouseAndGardenGoal {
        fn conditions(&self) -> &[Condition] {
            &self.preconditions
        }
        fn utility(&self, _ecs: &Ecs, _info: &EntityInfo) -> f32 {
            0.0
        }
    }

    struct BuildWoodHouseAction {
        preconditions: [Condition; 2],
        effects: [Effect; 2],
    }
    impl BuildWoodHouseAction {
        fn new() -> Self {
            Self {
                preconditions: [
                    Condition::new(Symbol::HasHouse, Operator::Equal, Value::Bool(false)),
                    Condition::new(Symbol::WoodCount, Operator::GreaterOrEqual, Value::Isize(2)),
                ],
                effects: [
                    Effect::new(Symbol::HasHouse, Modifier::SetValue, Value::Bool(true)),
                    Effect::new(Symbol::WoodCount, Modifier::Increment, Value::Isize(-2)),
                ],
            }
        }
    }
    impl Action for BuildWoodHouseAction {
        fn preconditions(&self) -> &[Condition] {
            &self.preconditions
        }
        fn effects(&self) -> &[Effect] {
            &self.effects
        }
        define_perform_success!();
    }

    struct PlantTreeAction {
        effects: [Effect; 1],
    }
    impl PlantTreeAction {
        fn new() -> Self {
            Self {
                effects: [Effect::new(
                    Symbol::SproutCount,
                    Modifier::Increment,
                    Value::Isize(1),
                )],
            }
        }
    }
    impl Action for PlantTreeAction {
        fn preconditions(&self) -> &[Condition] {
            &[]
        }
        fn effects(&self) -> &[Effect] {
            &self.effects
        }
        define_perform_success!();
    }

    struct WaitForOneTreeToGrowAction {
        preconditions: [Condition; 1],
        effects: [Effect; 2],
    }
    impl WaitForOneTreeToGrowAction {
        fn new() -> Self {
            Self {
                preconditions: [Condition::new(
                    Symbol::SproutCount,
                    Operator::GreaterOrEqual,
                    Value::Isize(1),
                )],
                effects: [
                    Effect::new(Symbol::SproutCount, Modifier::Increment, Value::Isize(-1)),
                    Effect::new(Symbol::TreeCount, Modifier::Increment, Value::Isize(1)),
                ],
            }
        }
    }
    impl Action for WaitForOneTreeToGrowAction {
        fn preconditions(&self) -> &[Condition] {
            &self.preconditions
        }
        fn effects(&self) -> &[Effect] {
            &self.effects
        }
        define_perform_success!();
    }

    struct CutTreeAction {
        preconditions: [Condition; 1],
        effects: [Effect; 2],
    }
    impl CutTreeAction {
        fn new() -> Self {
            Self {
                preconditions: [Condition::new(
                    Symbol::TreeCount,
                    Operator::GreaterOrEqual,
                    Value::Isize(1),
                )],
                effects: [
                    Effect::new(Symbol::TreeCount, Modifier::Increment, Value::Isize(-1)),
                    Effect::new(Symbol::WoodCount, Modifier::Increment, Value::Isize(1)),
                ],
            }
        }
    }
    impl Action for CutTreeAction {
        fn preconditions(&self) -> &[Condition] {
            &self.preconditions
        }
        fn effects(&self) -> &[Effect] {
            &self.effects
        }
        define_perform_success!();
    }

    struct BuyWoodAction {
        preconditions: [Condition; 1],
        effects: [Effect; 1],
    }
    impl BuyWoodAction {
        fn new() -> Self {
            Self {
                preconditions: [Condition::new(
                    Symbol::MoneyCount,
                    Operator::GreaterOrEqual,
                    Value::Isize(10),
                )],
                effects: [Effect::new(
                    Symbol::WoodCount,
                    Modifier::Increment,
                    Value::Isize(2),
                )],
            }
        }
    }
    impl Action for BuyWoodAction {
        fn preconditions(&self) -> &[Condition] {
            &self.preconditions
        }
        fn effects(&self) -> &[Effect] {
            &self.effects
        }
        define_perform_success!();
    }

    #[test]
    fn test_plan_found_plant_trees() {
        let mock_agent = AgentComponent::new(0, 0, 0);
        let goap = create_goap(/*patient*/ true);
        let world_state =
            create_world_state(/*house*/ false, /*trees*/ 3, /*money*/ 0);

        let plan = goap
            .compute_plan(&mock_agent, &world_state, 0, 0, 0)
            .expect("No plan found");
        print_plan(&goap, 0, &plan);

        let expected_plan = vec![
            type_name::<PlantTreeAction>(),
            type_name::<PlantTreeAction>(),
            type_name::<WaitForOneTreeToGrowAction>(),
            type_name::<CutTreeAction>(),
            type_name::<WaitForOneTreeToGrowAction>(),
            type_name::<CutTreeAction>(),
            type_name::<BuildWoodHouseAction>(),
        ];
        validate_plan(&goap, 0, &plan, expected_plan);
    }

    #[test]
    fn test_plan_found_buy_wood() {
        let mock_agent = AgentComponent::new(0, 0, 0);
        let goap = create_goap(/*patient*/ false);
        let world_state =
            create_world_state(/*house*/ false, /*trees*/ 3, /*money*/ 10);

        let plan = goap
            .compute_plan(&mock_agent, &world_state, 0, 0, 0)
            .expect("No plan found");
        print_plan(&goap, 0, &plan);

        let expected_plan = vec![
            type_name::<BuyWoodAction>(),
            type_name::<BuildWoodHouseAction>(),
        ];
        validate_plan(&goap, 0, &plan, expected_plan);
    }

    #[test]
    fn test_plan_not_found() {
        let mock_agent = AgentComponent::new(0, 0, 0);
        let goap = create_goap(/*patient*/ false);
        let world_state =
            create_world_state(/*house*/ false, /*trees*/ 3, /*money*/ 0);
        assert_eq!(goap.compute_plan(&mock_agent, &world_state, 0, 0, 0), None);
    }

    fn create_goap(patient: bool) -> Goap {
        let mut goap = Goap::new();

        let mut goal_set = GoalSet::new();
        goal_set.add(Box::new(HaveHouseAndGardenGoal::new()));
        goap.add_goal_set(goal_set);

        let mut action_set = ActionSet::new();
        action_set.add(Box::new(BuildWoodHouseAction::new()));
        action_set.add(Box::new(PlantTreeAction::new()));
        action_set.add(Box::new(CutTreeAction::new()));
        action_set.add(Box::new(BuyWoodAction::new()));
        if patient {
            action_set.add(Box::new(WaitForOneTreeToGrowAction::new()));
        }
        goap.add_action_set(action_set);

        goap
    }

    fn create_world_state(has_house: bool, trees: isize, money: isize) -> WorldState {
        WorldState {
            facts: vec![
                Fact::new(Symbol::HasHouse, Value::Bool(has_house)),
                Fact::new(Symbol::TreeCount, Value::Isize(trees)),
                Fact::new(Symbol::MoneyCount, Value::Isize(money)),
            ],
        }
    }

    fn print_plan(goap: &Goap, action_set: usize, plan: &Vec<usize>) {
        println!("Plan:");
        for a_index in plan {
            let action = goap.get_action(action_set, *a_index).unwrap();
            println!("{:}", action.type_name());
        }
    }

    fn validate_plan(goap: &Goap, action_set: usize, plan: &[usize], expected_plan: Vec<&str>) {
        assert_eq!(plan.len(), expected_plan.len());
        for (i, a_index) in plan.iter().enumerate() {
            let action = goap.get_action(action_set, *a_index).unwrap();
            assert_eq!(action.type_name(), expected_plan[i]);
        }
    }
}
