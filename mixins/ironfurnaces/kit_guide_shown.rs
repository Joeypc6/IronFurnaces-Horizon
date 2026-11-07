fn kit_guide_shown_ironfurnaces(stack: &ItemStack) {
    ironfurnaces_optional::if_kit!("ironfurnaces", {
    crate::game::hz_kit_guide_ironfurnaces::hovered(&stack.item);
} else { () })
}
