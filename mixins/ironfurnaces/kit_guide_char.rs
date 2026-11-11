fn kit_guide_char_ironfurnaces(game: &crate::game::Game<'_>, c: char, canceled: &mut bool) {
    ironfurnaces_optional::if_kit!("ironfurnaces", {
    let _ = c;
    if game.player.is_some() && crate::game::hz_kit_guide_ironfurnaces::is_open() {
        *canceled = true;
    }
} else { let _ = (game, c, canceled); () })
}
