fn kit_guide_key_ironfurnaces(game: &crate::game::Game<'_>, screen: &crate::game::native_hooks::NativeScreenView<'_>, key: &str, repeat: bool, canceled: &mut bool) {
    ironfurnaces_optional::if_kit!("ironfurnaces", {
    if game.player.is_none() {
        return;
    }
    crate::game::hz_kit_guide_ironfurnaces::key(game, screen.menu, key, repeat, canceled);
} else { let _ = (game, screen, key, repeat, canceled); () })
}
