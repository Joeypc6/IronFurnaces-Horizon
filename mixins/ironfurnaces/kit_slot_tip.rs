fn jei_slot_tip(game: &crate::game::Game<'_>) -> core::ops::ControlFlow<()> {
    ironfurnaces_optional::if_kit_jei!("ironfurnaces", {
    if crate::game::hz_kit_window_ironfurnaces::defer_tooltip(game) {
        return core::ops::ControlFlow::Break(());
    }
} else { let _ = game; () });
    core::ops::ControlFlow::Continue(())
}
