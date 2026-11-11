fn kit_guide_dragged_ironfurnaces(delta: [f32; 2], canceled: &mut bool) {
    ironfurnaces_optional::if_kit!("ironfurnaces", {
    crate::game::hz_kit_guide_ironfurnaces::drag(delta, canceled);
} else { let _ = (delta, canceled); () })
}
