fn kit_guide_scrolled_ironfurnaces(game: &crate::game::Game<'_>, scroll: [f64; 2], canceled: &mut bool) {
    ironfurnaces_optional::if_kit!("ironfurnaces", {
    if game.player.is_some() {
        crate::game::hz_kit_guide_ironfurnaces::scroll(scroll[1], canceled);
    }
} else { let _ = (game, scroll, canceled); () })
}
