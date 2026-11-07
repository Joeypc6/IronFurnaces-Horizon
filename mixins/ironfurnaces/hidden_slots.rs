fn ironfurnaces_hidden_slots(game: &crate::game::Game<'_>, pos: Pos, size: usize) -> core::ops::ControlFlow<Vec<usize>> {
    match crate::game::ironfurnaces::hidden_slots(game, [pos.0, pos.1, pos.2], size) {
        Some(hidden) => core::ops::ControlFlow::Break(hidden),
        None => core::ops::ControlFlow::Continue(()),
    }
}
