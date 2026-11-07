fn ironfurnaces_update_tag(entity: &horizon_client::containers::ModBlockEntity, tag: &mut Option<horizon_world::nbt::Compound>) {
    crate::game::ironfurnaces::update_tag(entity, tag);
}
