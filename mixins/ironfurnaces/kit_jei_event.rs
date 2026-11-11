fn ae2_jei_event(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    event: &crate::game::native_hooks::NativePanelEvent,
) {
    ironfurnaces_optional::if_kit_jei!("ironfurnaces", {
    crate::game::hz_kit_layer_ironfurnaces::event(game, mod_id, event);
} else { let _ = (game, mod_id, event); () })
}
