// BlockIronFurnaceBase.getLightEmission: a lit furnace gives off 14 (a content block's light is one number otherwise).
fn ironfurnaces_light_state(b: &NewBlock, combo: &[(String, String)], row: &mut J) {
    if !b.decl.id.starts_with("ironfurnaces:") || !b.decl.id.ends_with("_furnace") {
        return;
    }
    let lit = combo.iter().any(|(k, v)| k == "lit" && v == "true");
    row.set("emission", J::Num(if lit { 14i64 } else { 0i64 }.into()));
}
