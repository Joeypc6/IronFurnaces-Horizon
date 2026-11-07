fn jei_creative_width() -> core::ops::ControlFlow<Option<f32>> {
    ironfurnaces_optional::if_kit_jei!("ironfurnaces", {
    match crate::game::hz_kit_creative_ironfurnaces::screen_width() {
        Some(width) => core::ops::ControlFlow::Break(Some(width)),
        None => core::ops::ControlFlow::Continue(()),
    }
} else { core::ops::ControlFlow::Continue(()) })
}
