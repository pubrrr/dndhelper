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
    /// `field_of_movement` adds 1 to all costs, so we need to subtract 1 from the cost
    pub(super) fn get_modified_algorithm_cost(&self) -> Option<u32> {
        match self {
            MovementCost::Impassable => None,
            MovementCost::Passable(cost) => Some(max(*cost as u32, 1) - 1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_modified_algorithm_cost() {
        assert_eq!(MovementCost::Impassable.get_modified_algorithm_cost(), None);
        assert_eq!(
            MovementCost::Passable(0).get_modified_algorithm_cost(),
            Some(0)
        );
        assert_eq!(
            MovementCost::Passable(1).get_modified_algorithm_cost(),
            Some(0)
        );
        assert_eq!(
            MovementCost::Passable(2).get_modified_algorithm_cost(),
            Some(1)
        );
        assert_eq!(
            MovementCost::Passable(3).get_modified_algorithm_cost(),
            Some(2)
        );
    }
}
