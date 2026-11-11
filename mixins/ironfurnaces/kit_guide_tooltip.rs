fn kit_guide_tooltip_ironfurnaces(stack: &horizon_client::items::ItemStack, lines: &mut Vec<(horizon_client::text::Text, u32)>) {
    ironfurnaces_optional::if_kit!("ironfurnaces", {
    crate::game::hz_kit_guide_ironfurnaces::tooltip(stack, lines);
} else { let _ = (stack, lines); () })
}
