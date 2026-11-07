fn kit_guide_released_ironfurnaces(game: &crate::game::Game<'_>, canceled: &mut bool) {
    ironfurnaces_optional::if_kit!("ironfurnaces", {
    crate::game::hz_kit_guide_ironfurnaces::release(game, canceled);
} else { () })
}
