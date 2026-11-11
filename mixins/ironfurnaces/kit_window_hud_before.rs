fn jei_hud_before(game: &mut crate::game::Game<'_>) {
    ironfurnaces_optional::if_kit_jei!("ironfurnaces", {
    crate::game::hz_kit_window_ironfurnaces::hud_before(game);
} else { let _ = game; () })
}
