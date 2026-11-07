fn kit_guide_pressed_ironfurnaces(game: &crate::game::Game<'_>, screen: &crate::game::native_hooks::NativeScreenView<'_>, button: i32, canceled: &mut bool) {
    ironfurnaces_optional::if_kit!("ironfurnaces", {
    if game.player.is_none() {
        return;
    }
    let _ = button;
    crate::game::hz_kit_guide_ironfurnaces::click(game, screen.mouse, canceled);
} else { () })
}
