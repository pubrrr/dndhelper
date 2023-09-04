use bevy::prelude::{Component, Query};

#[derive(Component, Debug)]
pub struct ActionPoints {
    max: usize,
    pub left: usize,
    max_attacks: usize,
    pub attacks_this_round: usize,
    attack_action_point_cost: usize,
}

impl ActionPoints {
    pub fn new(max: usize, max_attacks: usize, attack_action_point_cost: usize) -> Self {
        ActionPoints {
            left: max,
            max,
            max_attacks,
            attacks_this_round: 0,
            attack_action_point_cost,
        }
    }

    pub fn get_max(&self) -> usize {
        self.max
    }

    pub fn get_max_attacks(&self) -> usize {
        self.max_attacks
    }

    pub fn attack_action_point_cost(&self) -> usize {
        self.attack_action_point_cost
    }

    pub fn can_still_attack_this_turn(&self) -> bool {
        self.attacks_this_round < self.max_attacks && self.left >= self.attack_action_point_cost
    }
}

pub fn reset_action_points(mut action_points_entities: Query<&mut ActionPoints>) {
    action_points_entities.for_each_mut(|mut action_points| {
        action_points.left = action_points.max;
        action_points.attacks_this_round = 0;
    });
}
