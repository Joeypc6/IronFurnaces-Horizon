fn jei_window_released(screen: &crate::game::native_hooks::NativeScreenView<'_>) {
    ironfurnaces_optional::if_kit_jei!("ironfurnaces", {
    crate::game::hz_kit_window_ironfurnaces::release(screen.mouse);
} else { () })
}
