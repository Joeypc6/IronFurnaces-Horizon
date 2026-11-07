fn jei_no_recipes_line(text: &str) -> core::ops::ControlFlow<()> {
    ironfurnaces_optional::if_kit_jei!("ironfurnaces", {
    if crate::game::hz_kit_layer_ironfurnaces::chat_allowed(text) {
        core::ops::ControlFlow::Continue(())
    } else {
        core::ops::ControlFlow::Break(())
    }
} else { core::ops::ControlFlow::Continue(()) })
}
