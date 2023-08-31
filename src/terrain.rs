use bevy::prelude::Component;

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
