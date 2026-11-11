// BlockIronFurnaceBase.getLightEmission: a lit furnace gives off 14 (a content block's light is one number otherwise),
// none with `misc.lightupdates = true` (Config.disableLightupdates). The game folder is the working directory and the
// mod's config is config/ironfurnaces.toml in it (Game::native_config_dir); the light table is made once, at load.
fn ironfurnaces_light_state(b: &NewBlock, combo: &[(String, String)], row: &mut J) {
    if !b.decl.id.starts_with("ironfurnaces:") || !b.decl.id.ends_with("_furnace") {
        return;
    }
    static DISABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    let disabled = *DISABLED.get_or_init(|| {
        let file = std::env::current_dir().unwrap_or_default().join("config").join("ironfurnaces.toml");
        std::fs::read_to_string(file).is_ok_and(|text| {
            text.lines().filter_map(|l| l.split_once('=')).any(|(k, v)| k.trim() == "misc.lightupdates" && v.trim() == "true")
        })
    });
    let lit = combo.iter().any(|(k, v)| k == "lit" && v == "true");
    row.set("emission", J::Num(if lit && !disabled { 14i64 } else { 0i64 }.into()));
}
