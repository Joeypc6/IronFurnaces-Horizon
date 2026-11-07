fn ironfurnaces_slot_rules(menu: &str, slot: usize, stack: &ItemStack, may_place: &mut bool, max_stack: &mut u32) {
    if menu == "ironfurnaces:heater" {
        *may_place = &*stack.item == "ironfurnaces:item_heater";
        *max_stack = 1;
    } else if menu == "ironfurnaces:furnace" {
        let augment = match &*stack.item {
            "ironfurnaces:augment_blasting" | "ironfurnaces:augment_smoking" => Some(3),
            "ironfurnaces:augment_speed" | "ironfurnaces:augment_fuel" => Some(4),
            "ironfurnaces:augment_factory" | "ironfurnaces:augment_generator" => Some(5),
            _ => None,
        };
        match slot {
            2 | 13..=18 => *may_place = false,
            3..=5 => {
                *may_place = augment == Some(slot);
                *max_stack = 1;
            }
            _ => {}
        }
    }
}
