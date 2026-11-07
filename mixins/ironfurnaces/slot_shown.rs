// The Slot subclasses' isActive: a page shows some of the 19 slots. A slot it doesn't show has no place to be drawn or clicked.
fn ironfurnaces_slot_shown(game: &crate::game::Game<'_>, slot: usize) -> core::ops::ControlFlow<Option<(f32, f32)>> {
    if game.native_mod_screen() == Some("ironfurnaces") && !crate::game::ironfurnaces::client_slot_shown(slot) {
        return core::ops::ControlFlow::Break(None);
    }
    core::ops::ControlFlow::Continue(())
}
