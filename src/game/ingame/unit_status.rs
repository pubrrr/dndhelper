use bevy::prelude::{debug, Component, DetectChanges, Entity, Query, Ref, ResMut};
use bevy::utils::HashSet;

use crate::game::ingame::hex::HexComponent;
use crate::game::ingame::selected_unit::SelectedUnitResource;
use crate::game::ingame::team_setup::Team;

#[derive(Component, Debug)]
pub struct UnitStatus {
    engaged_with_units: HashSet<Entity>,
}

impl UnitStatus {
    pub fn new() -> Self {
        Self {
            engaged_with_units: HashSet::new(),
        }
    }

    pub fn engage_with(&mut self, entity: Entity) {
        self.engaged_with_units.insert(entity);
    }

    pub fn disengage_with(&mut self, entity: &Entity) {
        self.engaged_with_units.remove(entity);
    }

    pub fn is_engaged_with(&self, entity: &Entity) -> bool {
        self.engaged_with_units.contains(entity)
    }

    pub fn get_engaged_with_units(&self) -> &HashSet<Entity> {
        &self.engaged_with_units
    }
}

pub(super) fn disengage_apart_units(
    mut selected_unit_resource: ResMut<SelectedUnitResource>,
    mut units: Query<(Entity, &mut UnitStatus, Ref<HexComponent>, &Team)>,
) {
    let moved_unit_and_enemy_neighbors: Vec<_> = units
        .iter()
        .filter(|(_, _, hex_component, _)| hex_component.is_changed())
        .map(
            |(changed_entity, _, changed_hex_component, changed_unit_team)| {
                let adjacent_units: Vec<_> = units
                    .iter()
                    .filter(|(_, _, _, team)| team != &changed_unit_team)
                    .filter(|(_, _, hex_component, _)| {
                        let distance = hex_component
                            .0
                            .unsigned_distance_to(changed_hex_component.0);
                        distance == 1
                    })
                    .map(|(entity, _, _, _)| *&entity)
                    .collect();

                (*&changed_entity, adjacent_units)
            },
        )
        .collect();

    let mut maybe_changed_something = false;
    for (changed_entity, adjacent_entities) in moved_unit_and_enemy_neighbors {
        maybe_changed_something = true;
        let (_, mut unit_status, _, _) = units.get_mut(changed_entity).unwrap();
        unit_status
            .engaged_with_units
            .retain(|engaged_with| adjacent_entities.contains(engaged_with));
        unit_status
            .engaged_with_units
            .extend(adjacent_entities.clone());
        debug!(
            "Handled engagement for unit {changed_entity:?}, now is {:?}",
            unit_status.engaged_with_units
        );

        units
            .iter_mut()
            .filter(|(entity, _, _, _)| entity != &changed_entity)
            .filter(|(entity, _, _, _)| !adjacent_entities.contains(entity))
            .for_each(
                |(not_adjacent_entity, mut not_adjacent_unit_status, _, _)| {
                    if not_adjacent_unit_status.is_engaged_with(&changed_entity) {
                        debug!("Disengaging {not_adjacent_entity:?} from {changed_entity:?}");
                        not_adjacent_unit_status.disengage_with(&changed_entity);
                    }
                },
            );

        for adjacent_entity in adjacent_entities {
            let (adjacent_entity, mut adjacent_unit_status, _, _) =
                units.get_mut(adjacent_entity).unwrap();
            if !adjacent_unit_status.is_engaged_with(&changed_entity) {
                debug!("Engaging {adjacent_entity:?} with {changed_entity:?}");
                adjacent_unit_status.engage_with(changed_entity);
            }
        }
    }

    if maybe_changed_something {
        selected_unit_resource.needs_reachable_hexes_recomputation();
    }
}
