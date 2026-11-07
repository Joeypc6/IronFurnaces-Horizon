// getRedstoneSignalFromContainer over getComparatorInputOverride's slots.
fn ironfurnaces_analog(level: &crate::game::GameLevel<'_>, pos: horizon_core::position::BlockPos, state: horizon_world::registry::BlockStateId, output: &mut i32) {
    if !crate::game::ironfurnaces::is_furnace(&level.facts.get(state).name) {
        return;
    }
    let Some(horizon_client::containers::BlockEntity::Mod(m)) = level.world.block_entities.get(&(pos.x, pos.y, pos.z)) else { return };
    let slots = crate::game::ironfurnaces::comparator_slots(m);
    let (mut total, mut any) = (0.0f32, false);
    for stack in slots.iter().filter_map(|i| m.items.get(*i)).flatten() {
        total += stack.count as f32 / level.items.get(&stack.item).map_or(64, |i| i.max_stack.clamp(1, 64)) as f32;
        any = true;
    }
    total /= slots.len().max(1) as f32;
    *output = (total.min(1.0) * 14.0).floor() as i32 + i32::from(any);
}
