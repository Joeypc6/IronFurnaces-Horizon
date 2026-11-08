// The mod's row in Options > Mods, and its one key.
fn ironfurnaces_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "ironfurnaces".into(),
        title: "Iron Furnaces 4.4.3".into(),
        description: "Furnaces of every metal, with augments, factories and generators.".into(),
        keys: vec![["guide".to_string(), "Open Guide for Items".to_string(), "key.keyboard.w".to_string()]],
    });
}
