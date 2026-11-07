fn ironfurnaces_be(block: &str, entity: &mut Option<(String, horizon_world::nbt::Compound)>) {
    crate::game::ironfurnaces::block_entity(block, entity);
}
