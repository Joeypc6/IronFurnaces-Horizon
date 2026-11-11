fn jei_panel_tint(game: &Game<'_>, batch: &mut HudBatch) {
    ironfurnaces_optional::if_kit_jei!("ironfurnaces", {
    crate::game::hz_kit_window_ironfurnaces::tint(game, batch);
} else { let _ = (game, batch); () })
}
