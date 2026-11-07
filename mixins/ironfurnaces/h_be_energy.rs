fn ironfurnaces_be_energy(block: &str, energy: &mut Option<horizon_client::containers::ModEnergyBuffer>) {
    crate::game::ironfurnaces::block_entity_energy(block, energy);
}
