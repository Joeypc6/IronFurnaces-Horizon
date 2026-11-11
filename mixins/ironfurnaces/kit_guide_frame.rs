fn kit_guide_frame_ironfurnaces(game: &mut crate::game::Game<'_>) {
    ironfurnaces_optional::if_kit!("ironfurnaces", {
    if game.player.is_some() {
        crate::game::hz_kit_guide_ironfurnaces::frame_hook(game);
    }
} else { let _ = game; () })
}
