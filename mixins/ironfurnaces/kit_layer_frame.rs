fn kit_layer_frame_ironfurnaces(game: &mut crate::game::Game<'_>) {
    ironfurnaces_optional::if_kit_jei!("ironfurnaces", {
    if game.native_on(crate::game::jei::ID) {
        crate::game::hz_kit_layer_ironfurnaces::frame(game);
    }
} else { () })
}
