use bevy::prelude::Component;
use std::cmp::max;
use std::fmt::{Display, Formatter};

#[derive(Component, Debug, Clone)]
pub struct Terrain {
    pub name: String,
    pub movement_cost: MovementCost,
}

#[derive(Debug, Clone)]
pub enum MovementCost {
    Impassable,
    Passable(usize),
}

impl Display for MovementCost {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            MovementCost::Impassable => write!(f, "Impassable"),
            MovementCost::Passable(cost) => write!(f, "{cost}"),
        }
    }
}

impl MovementCost {
    pub fn get_modified_algorithm_cost(&self) -> Option<u32> {
        match self {
            MovementCost::Impassable => None,
            MovementCost::Passable(cost) => Some(max(*cost as u32, 1) - 1),
        }
    }
}
