use crate::infrastructure::{infrastructure_definitions, MAINTENANCE_INTERVAL_SECONDS};
use crate::world::types::EconomicAccount;
use crate::world::World;

impl World {
    pub(super) fn update_infrastructure_maintenance(&mut self, dt_seconds: f64) {
        self.maintenance_time_accumulator += dt_seconds.max(0.0);
        let intervals =
            (self.maintenance_time_accumulator / MAINTENANCE_INTERVAL_SECONDS).floor() as u32;
        if intervals == 0 {
            return;
        }
        self.maintenance_time_accumulator -= intervals as f64 * MAINTENANCE_INTERVAL_SECONDS;
        self.charge_infrastructure_maintenance(intervals);
        for body in self.celestial_data.values_mut() {
            body.procurement_spend.clear();
        }
    }

    fn charge_infrastructure_maintenance(&mut self, intervals: u32) {
        let mut entity_ids: Vec<_> = self.infrastructure.keys().copied().collect();
        entity_ids.sort_unstable();

        for entity_id in entity_ids {
            let account = self.procurement_account(entity_id);
            for definition in infrastructure_definitions() {
                let infrastructure_type = definition.infrastructure_type;
                let Some((completed_units, existing_arrears)) =
                    self.infrastructure.get(&entity_id).map(|infrastructure| {
                        (
                            infrastructure.get_count(infrastructure_type),
                            infrastructure.maintenance_arrears(infrastructure_type),
                        )
                    })
                else {
                    continue;
                };
                if completed_units == 0 {
                    continue;
                }

                let current_upkeep = definition.maintenance_credits_per_interval
                    * completed_units as f64
                    * intervals as f64;
                let due = existing_arrears + current_upkeep;
                let paid = self.debit_account(account, due);
                self.infrastructure
                    .get_mut(&entity_id)
                    .unwrap()
                    .set_maintenance_arrears(infrastructure_type, due - paid);
            }
        }
    }

    fn debit_account(&mut self, account: EconomicAccount, amount: f64) -> f64 {
        if !amount.is_finite() || amount <= 0.0 {
            return 0.0;
        }
        let Some(balance) = self.account_balance(account) else {
            return 0.0;
        };
        let paid = amount.min(balance.max(0.0));
        match account {
            EconomicAccount::PlayerTreasury => self.player_credits -= paid,
            EconomicAccount::Civilian(entity_id) => {
                self.celestial_data.get_mut(&entity_id).unwrap().credits -= paid;
            }
        }
        paid
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::location::Point;
    use crate::world::types::{
        ConstructionLayer, Good, InfrastructureType, ProcurementKey, ProcurementPolicy,
        RawResource, Storable,
    };

    #[test]
    fn maintenance_debits_player_and_civilian_owners() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let player_body = world.spawn_planet("earth".to_string(), star_id, 8.0, 0.0, 0.0);
        let civilian_body = world.spawn_planet("mars".to_string(), star_id, 10.0, 0.0, 0.0);
        world.set_player_controlled(player_body);
        world.player_credits = 2.0;
        world.celestial_data.get_mut(&player_body).unwrap().credits = 11.0;
        world
            .celestial_data
            .get_mut(&civilian_body)
            .unwrap()
            .credits = 2.0;
        world
            .infrastructure
            .get_mut(&player_body)
            .unwrap()
            .infra
            .insert(InfrastructureType::Mine, 1);
        world
            .infrastructure
            .get_mut(&civilian_body)
            .unwrap()
            .infra
            .insert(InfrastructureType::Farm, 1);

        world.update_infrastructure_maintenance(MAINTENANCE_INTERVAL_SECONDS);

        assert_eq!(world.player_credits, 0.0);
        assert_eq!(world.celestial_data[&player_body].credits, 11.0);
        assert_eq!(world.celestial_data[&civilian_body].credits, 0.0);
        assert!(world.infrastructure[&player_body]
            .maintenance_statuses()
            .iter()
            .all(|status| status.active));
        assert!(world.infrastructure[&civilian_body]
            .maintenance_statuses()
            .iter()
            .all(|status| status.active));
    }

    #[test]
    fn modeled_month_resets_procurement_spend() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let body_id = world.spawn_planet("earth".to_string(), star_id, 8.0, 0.0, 0.0);
        let key = ProcurementKey {
            layer: ConstructionLayer::Orbit,
            resource: Storable::Raw(RawResource::Metals),
        };
        world
            .celestial_data
            .get_mut(&body_id)
            .unwrap()
            .procurement_spend
            .insert(key, 25.0);

        world.update_infrastructure_maintenance(MAINTENANCE_INTERVAL_SECONDS - 1.0);
        assert_eq!(world.celestial_data[&body_id].procurement_spend[&key], 25.0);

        world.update_infrastructure_maintenance(1.0);
        assert!(world.celestial_data[&body_id].procurement_spend.is_empty());
    }

    #[test]
    fn shared_player_funds_are_charged_in_body_id_order() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let first_body = world.spawn_planet("first".to_string(), star_id, 8.0, 0.0, 0.0);
        let second_body = world.spawn_planet("second".to_string(), star_id, 10.0, 0.0, 0.0);
        for body in [first_body, second_body] {
            world.set_player_controlled(body);
            world
                .infrastructure
                .get_mut(&body)
                .unwrap()
                .infra
                .insert(InfrastructureType::SurfaceWarehouse, 1);
        }
        world.player_credits = 1.0;

        world.update_infrastructure_maintenance(MAINTENANCE_INTERVAL_SECONDS);

        assert_eq!(
            world.infrastructure[&first_body]
                .maintenance_arrears(InfrastructureType::SurfaceWarehouse),
            0.0
        );
        assert_eq!(
            world.infrastructure[&second_body]
                .maintenance_arrears(InfrastructureType::SurfaceWarehouse),
            1.0
        );
    }

    #[test]
    fn delinquent_storage_preserves_stock_and_recovers_after_arrears_are_paid() {
        let mut world = World::default();
        let star_id = world.spawn_star("sol".to_string(), Point { x: 0, y: 0 });
        let body_id = world.spawn_planet("earth".to_string(), star_id, 8.0, 0.0, 0.0);
        world.set_player_controlled(body_id);
        world
            .infrastructure
            .get_mut(&body_id)
            .unwrap()
            .infra
            .insert(InfrastructureType::OrbitalDepot, 1);
        let resource = Storable::Good(Good::Food);
        let procurement_key = ProcurementKey {
            layer: ConstructionLayer::Orbit,
            resource,
        };
        world
            .celestial_data
            .get_mut(&body_id)
            .unwrap()
            .procurement_policies
            .insert(
                procurement_key,
                ProcurementPolicy {
                    enabled: true,
                    reserve_target: 100.0,
                    maximum_unit_price: 100.0,
                    periodic_spend_cap: None,
                },
            );
        world
            .celestial_data
            .get_mut(&body_id)
            .unwrap()
            .deposit_bounded_at(ConstructionLayer::Orbit, resource, 10.0, 1_000.0);

        world.update_infrastructure_maintenance(MAINTENANCE_INTERVAL_SECONDS);

        assert_eq!(
            world.storage_capacity(body_id, ConstructionLayer::Orbit),
            Some((1_000.0, 10.0))
        );
        assert_eq!(
            world.infrastructure[&body_id].accepting_storage_capacity(ConstructionLayer::Orbit),
            0.0
        );
        let status = world.infrastructure[&body_id]
            .maintenance_statuses()
            .into_iter()
            .find(|status| status.infrastructure_type == InfrastructureType::OrbitalDepot)
            .unwrap();
        assert!(!status.active);
        assert_eq!(status.upkeep_per_interval, 1.5);
        assert_eq!(status.arrears, 1.5);
        world.player_credits = 100.0;
        assert_eq!(world.procurement_quote(body_id, procurement_key), None);

        world.player_credits = 3.0;
        world.update_infrastructure_maintenance(MAINTENANCE_INTERVAL_SECONDS);

        assert_eq!(
            world.infrastructure[&body_id].accepting_storage_capacity(ConstructionLayer::Orbit),
            1_000.0
        );
        assert_eq!(
            world.celestial_data[&body_id].amount_at(ConstructionLayer::Orbit, resource),
            10.0
        );
        assert_eq!(world.player_credits, 0.0);
    }
}
