fn ironfurnaces_has_analog(level: &crate::game::GameLevel<'_>, state: horizon_world::registry::BlockStateId, output: &mut bool) {
    if crate::game::ironfurnaces::is_furnace(&level.facts.get(state).name) {
        *output = true;
    }
}
