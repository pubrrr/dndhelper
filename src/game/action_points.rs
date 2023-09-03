use bevy::prelude::{Component, Query};

#[derive(Component, Debug)]
pub struct ActionPoints {
    max: usize,
    pub left: usize,
}

impl ActionPoints {
    pub fn with_max(max: usize) -> Self {
        ActionPoints { left: max, max }
    }

    pub fn get_max(&self) -> usize {
        self.max
    }
}

pub fn reset_action_points(mut action_points_entities: Query<&mut ActionPoints>) {
    action_points_entities.for_each_mut(|mut action_points| action_points.left = action_points.max);
}
