"""Writes the Iron Furnaces bundle's horizon-mixin.toml and its one-line hook helpers, and packs the content.

Usage: python make_bundle.py
"""

import json
import pathlib
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
BUNDLE = ROOT / "mixins" / "ironfurnaces"
BUILD = ROOT.parent / "native-build" / "main" / "ironfurnaces"
JAR = ROOT.parent / "ironfurnaces-neoforge-26.1.2-4.4.2.jar"
if not JAR.exists():
    JAR = ROOT.parent / "Ported" / JAR.name
HORIZON = "6560641ba51a452f6bc2407baa4eb1b3be517ca8"
MIXINS = "5a64af938ec86b00d53d16a408aabc0038fb3e57"
VERSION = "4.4.2"
M = "crate::game::ironfurnaces"

GAME_MUT = ("game", "&mut crate::game::Game<'_>", "&mut *self")
GAME = ("game", "&crate::game::Game<'_>", "&*self")
ENTITY = ("entity", "&mut horizon_client::containers::ModBlockEntity", "&mut *entity")

# (id, hook, bindings, body)
HOOKS = [
    ("be", "block_entity", [("block", "&str", "block"), ("entity", "&mut Option<(String, horizon_world::nbt::Compound)>", "&mut *entity")], "block_entity(block, entity)"),
    ("be-ticks", "block_entity_ticks", [("block", "&str", "block"), ("ticks", "&mut bool", "&mut *ticks")], "block_entity_ticks(block, ticks)"),
    ("be-container", "block_entity_container", [("block", "&str", "block"), ("container", "&mut Option<horizon_client::containers::ModContainer>", "&mut *container")], "block_entity_container(block, container)"),
    ("be-energy", "block_entity_energy", [("block", "&str", "block"), ("energy", "&mut Option<horizon_client::containers::ModEnergyBuffer>", "&mut *energy")], "block_entity_energy(block, energy)"),
    ("be-tick", "block_entity_tick", [GAME_MUT, ("pos", "[i32; 3]", "pos"), ENTITY], "block_entity_tick(game, pos, entity)"),
    ("accepts", "target:horizon_client::game::native_hooks::Game<'_>::native_container_accepts", [GAME, ("entity", "&horizon_client::containers::ModBlockEntity", "&*entity"), ("slot", "usize", "slot"), ("stack", "&horizon_client::items::ItemStack", "stack"), ("face", "Option<horizon_core::geometry::Direction>", "face"), ("cancel", "&mut bool", "&mut *cancel")], "container_accepts(game, entity, slot, stack, face, cancel)"),
    ("slot-rules", "menu_slot_rules", [("menu", "&str", "menu"), ("slot", "usize", "slot"), ("stack", "&ItemStack", "stack"), ("may_place", "&mut bool", "&mut *may_place"), ("max_stack", "&mut u32", "&mut *max_stack")], None),
    ("quick-move", "menu_quick_move", [GAME_MUT, ENTITY, ("container_slots", "usize", "container_slots"), ("slot", "usize", "slot"), ("stack", "&mut horizon_client::items::ItemStack", "&mut *stack"), ("handled", "&mut bool", "&mut *handled")], "quick_move(game, entity, slot, container_slots, stack, handled)"),
    ("menu-clicked", "menu_clicked", [("uuid", "u128", "uuid"), ("slot", "i32", "slot")], "menu_clicked(uuid, slot)"),
    ("menu-button", "menu_button_v2", [("pos", "[i32; 3]", "pos"), ENTITY, ("id", "i32", "id")], "menu_button(pos, entity, id)"),
    ("placed", "place_block", [GAME_MUT, ("pos", "[i32; 3]", "pos")], "placed(game, pos)"),
    ("right-click", "right_click_block", [GAME_MUT, ("pos", "[i32; 3]", "pos"), ("canceled", "&mut bool", "&mut *canceled")], "right_click(game, pos, canceled)"),
    ("screen", "screen", [("block", "&str", "block"), ("screen", "&mut Option<crate::game::native_hooks::NativeScreen>", "&mut *screen")], "screen(block, screen)"),
    ("screen-draw", "screen_draw", [GAME, ("block", "&str", "block"), ("data", "&[i32]", "data"), ("batch", "&mut horizon_client::hud::HudBatch", "&mut *batch"), ("origin", "[f32; 2]", "origin")], "screen_draw(game, block, data, batch, origin)"),
    ("screen-click", "screen_click", [GAME, ("block", "&str", "block"), ("data", "&[i32]", "data"), ("at", "[f32; 2]", "at"), ("right", "bool", "right"), ("button", "&mut Option<i32>", "&mut *button")], "screen_click(block, data, at, right, game.has_shift_down(), button)"),
    ("screen-tooltip", "screen_tooltip", [("block", "&str", "block"), ("data", "&[i32]", "data"), ("at", "[f32; 2]", "at"), ("lines", "&mut Vec<String>", "&mut *lines")], "screen_tooltip(block, data, at, lines)"),
    ("tooltip", "tooltip", [("stack", "&ItemStack", "stack"), ("lines", "&mut Vec<(Text, u32)>", "&mut *lines")], "tooltip(stack, lines)"),
    ("kit-frame", "frame", [GAME_MUT], "kit_frame(game)"),
    ("tick", "tick", [GAME_MUT], "client_tick(game)"),
    ("update-tag", "block_entity_update_tag", [("entity", "&horizon_client::containers::ModBlockEntity", "&*entity"), ("tag", "&mut Option<horizon_world::nbt::Compound>", "&mut *tag")], "update_tag(entity, tag)"),
]

SPECIAL_HELPERS = {
    "embedded_content.rs": """// Iron Furnaces is one file on the registry: its blocks, recipes and textures are the content mod in this bundle's data/
// (made by tools/make_content.py from the jar). It is written out once and loaded as a dev mod is.
fn ironfurnaces_embedded_content(root: &std::path::Path, enabled: &std::collections::BTreeSet<String>, dev: &[std::path::PathBuf]) -> core::ops::ControlFlow<std::path::PathBuf> {
    static CONTENT: &[u8] = horizon_client::native_data!("ironfurnaces", "ironfurnaces-content.hmod");
    let dir = std::env::current_dir().unwrap_or_default().join(".horizon-cache");
    let file = dir.join("ironfurnaces-embedded-content.hmod");
    if dev.contains(&file) {
        return core::ops::ControlFlow::Continue(());
    }
    let same = std::fs::read(&file).is_ok_and(|on_disk| on_disk == CONTENT);
    if !same && std::fs::create_dir_all(&dir).and_then(|()| std::fs::write(&file, CONTENT)).is_err() {
        log::warn!("Horizon: Iron Furnaces' content could not be written to {}", file.display());
        return core::ops::ControlFlow::Continue(());
    }
    let mut with = dev.to_vec();
    with.push(file);
    core::ops::ControlFlow::Break(content_root(root, enabled, &with))
}
""",
    "has_analog.rs": """fn ironfurnaces_has_analog(level: &crate::game::GameLevel<'_>, state: horizon_world::registry::BlockStateId, output: &mut bool) {
    if crate::game::ironfurnaces::is_furnace(&level.facts.get(state).name) {
        *output = true;
    }
}
""",
    "analog.rs": """// getRedstoneSignalFromContainer over getComparatorInputOverride's slots.
fn ironfurnaces_analog(level: &crate::game::GameLevel<'_>, pos: horizon_core::position::BlockPos, state: horizon_world::registry::BlockStateId, output: &mut i32) {
    if !crate::game::ironfurnaces::is_furnace(&level.facts.get(state).name) {
        return;
    }
    let Some(horizon_client::containers::BlockEntity::Mod(m)) = level.world.block_entities.get(&(pos.x, pos.y, pos.z)) else { return };
    let slots = crate::game::ironfurnaces::comparator_slots(m);
    let (mut total, mut any) = (0.0f32, false);
    for stack in slots.iter().filter_map(|i| m.items.get(*i)).flatten() {
        total += stack.count as f32 / level.items.get(&stack.item).map_or(64, |i| i.max_stack.clamp(1, 64)) as f32;
        any = true;
    }
    total /= slots.len().max(1) as f32;
    *output = (total.min(1.0) * 14.0).floor() as i32 + i32::from(any);
}
""",
    "list.rs": """// The mod's row in Options > Mods, and its one key.
fn ironfurnaces_listed(mods: &mut Vec<crate::game::native_hooks::NativeMod>) {
    mods.push(crate::game::native_hooks::NativeMod {
        id: "ironfurnaces".into(),
        title: "Iron Furnaces 4.4.2".into(),
        description: "Furnaces of every metal, with augments, factories and generators.".into(),
        keys: vec![["guide".to_string(), "Open Guide for Items".to_string(), "key.keyboard.w".to_string()]],
    });
}
""",
    "light_state.rs": """// BlockIronFurnaceBase.getLightEmission: a lit furnace gives off 14 (a content block's light is one number otherwise).
fn ironfurnaces_light_state(b: &NewBlock, combo: &[(String, String)], row: &mut J) {
    if !b.decl.id.starts_with("ironfurnaces:") || !b.decl.id.ends_with("_furnace") {
        return;
    }
    let lit = combo.iter().any(|(k, v)| k == "lit" && v == "true");
    row.set("emission", J::Num(if lit { 14i64 } else { 0i64 }.into()));
}
""",
    "drops.rs": """fn ironfurnaces_drops(game: &mut crate::game::Game<'_>, pos: [i32; 3], state: BlockStateId, explosion: Option<f32>, drops: &mut Vec<ItemStack>) {
    let block = game.res.facts().get(state).name.to_string();
    crate::game::ironfurnaces::block_drops(game, pos, &block, explosion.is_some(), drops);
}
""",
    "slot_shown.rs": """// The Slot subclasses' isActive: a page shows some of the 19 slots. A slot it doesn't show has no place to be drawn or clicked.
fn ironfurnaces_slot_shown(game: &crate::game::Game<'_>, slot: usize) -> core::ops::ControlFlow<Option<(f32, f32)>> {
    if game.native_mod_screen() == Some("ironfurnaces") && !crate::game::ironfurnaces::client_slot_shown(slot) {
        return core::ops::ControlFlow::Break(None);
    }
    core::ops::ControlFlow::Continue(())
}
""",
    "hidden_slots.rs": """fn ironfurnaces_hidden_slots(game: &crate::game::Game<'_>, pos: Pos, size: usize) -> core::ops::ControlFlow<Vec<usize>> {
    match crate::game::ironfurnaces::hidden_slots(game, [pos.0, pos.1, pos.2], size) {
        Some(hidden) => core::ops::ControlFlow::Break(hidden),
        None => core::ops::ControlFlow::Continue(()),
    }
}
""",
}


def binding_lines(bindings):
    return "".join(f'  {{ name = "{n}", ty = "{t}", value = "{v}" }},\n' for n, t, v in bindings)


def main():
    BUNDLE.mkdir(parents=True, exist_ok=True)
    out = [
        "[mod]",
        'id = "ironfurnaces"',
        f'version = "{VERSION}"',
        "",
        "# The Horizon commit this bundle was built and tested on.",
        "[compatibility]",
        f'horizon_revision = "{HORIZON}"',
        "",
        "[[module]]",
        'id = "ironfurnaces"',
        'file = "crates/horizon-client/src/game.rs"',
        'name = "ironfurnaces"',
        'source = "ironfurnaces.rs"',
        "",
    ]
    for hook_id, hook, bindings, body in HOOKS:
        helper = f"h_{hook_id.replace('-', '_')}.rs"
        args = ", ".join(f"{n}: {t}" for n, t, _ in bindings)
        if body is None:
            # In the client library: no `crate::game` there, so it answers from a copy of the rules.
            text = (
                f"fn ironfurnaces_{hook_id.replace('-', '_')}({args}) {{\n"
                '    if menu == "ironfurnaces:heater" {\n'
                '        *may_place = &*stack.item == "ironfurnaces:item_heater";\n'
                "        *max_stack = 1;\n"
                '    } else if menu == "ironfurnaces:furnace" {\n'
                "        let augment = match &*stack.item {\n"
                '            "ironfurnaces:augment_blasting" | "ironfurnaces:augment_smoking" => Some(3),\n'
                '            "ironfurnaces:augment_speed" | "ironfurnaces:augment_fuel" => Some(4),\n'
                '            "ironfurnaces:augment_factory" | "ironfurnaces:augment_generator" => Some(5),\n'
                "            _ => None,\n"
                "        };\n"
                "        match slot {\n"
                "            2 | 13..=18 => *may_place = false,\n"
                "            3..=5 => {\n"
                "                *may_place = augment == Some(slot);\n"
                "                *max_stack = 1;\n"
                "            }\n"
                "            _ => {}\n"
                "        }\n"
                "    }\n"
                "}\n"
            )
        else:
            text = f"fn ironfurnaces_{hook_id.replace('-', '_')}({args}) {{\n    {M}::{body};\n}}\n"
        (BUNDLE / helper).write_text(text, encoding="utf8")
        out += ["[[injection]]", f'id = "if-{hook_id}"', 'kind = "HEAD"', *(([f'target = "{hook[7:]}"'] if hook.startswith("target:") else [f'hook = "{hook}"', "hook_version = 1"])), f'helper = "{helper}"', "bindings = [", binding_lines(bindings).rstrip("\n"), "]", ""]
    for name, text in SPECIAL_HELPERS.items():
        (BUNDLE / name).write_text(text, encoding="utf8")
    level = ("level", "&crate::game::GameLevel<'_>", "&*self")
    state = ("state", "horizon_world::registry::BlockStateId", "state")
    out += ["[[injection]]", 'id = "if-has-analog"', 'kind = "HEAD"', 'hook = "has_analog_output"', "hook_version = 1", 'helper = "has_analog.rs"', "bindings = [", binding_lines([level, state, ("output", "&mut bool", "&mut *output")]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "if-analog"', 'kind = "HEAD"', 'hook = "analog_output"', "hook_version = 1", 'helper = "analog.rs"', "bindings = [", binding_lines([level, ("pos", "horizon_core::position::BlockPos", "pos"), state, ("output", "&mut i32", "&mut *output")]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "if-list"', 'kind = "HEAD"', 'hook = "mods"', "hook_version = 1", 'helper = "list.rs"', "bindings = [", binding_lines([("mods", "&mut Vec<crate::game::native_hooks::NativeMod>", "&mut *mods")]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "if-light-state"', 'kind = "HEAD"', 'target = "horizon_client::mod_content::Plan<\'_>::override_gameplay"', 'helper = "light_state.rs"', "bindings = [", binding_lines([("b", "&NewBlock", "b"), ("combo", "&[(String, String)]", "combo"), ("row", "&mut J", "&mut *row")]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "if-drops"', 'kind = "HEAD"', 'target = "horizon_client::game::native_hooks::Game<\'_>::native_block_drops"', 'helper = "drops.rs"', "bindings = [", binding_lines([GAME_MUT, ("pos", "[i32; 3]", "pos"), ("state", "BlockStateId", "state"), ("explosion", "Option<f32>", "explosion"), ("drops", "&mut Vec<ItemStack>", "&mut *drops")]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "if-slot-shown"', 'kind = "HEAD"', "cancellable = true", 'target = "horizon_client::game::native_hooks::Game<\'_>::native_screen_slot"', 'helper = "slot_shown.rs"', "bindings = [", binding_lines([GAME, ("slot", "usize", "slot")]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "if-hidden-slots"', 'kind = "HEAD"', "cancellable = true", 'target = "horizon_client::game::native_hooks::Game<\'_>::native_hidden_slots"', 'helper = "hidden_slots.rs"', "bindings = [", binding_lines([GAME, ("pos", "Pos", "pos"), ("size", "usize", "size")]).rstrip("\n"), "]", ""]
    out += ["[[injection]]", 'id = "if-embedded-content"', 'kind = "HEAD"', "cancellable = true", 'target = "horizon_client::game::mods::content_root"', 'helper = "embedded_content.rs"', "bindings = [", binding_lines([("root", "&std::path::Path", "root"), ("enabled", "&std::collections::BTreeSet<String>", "enabled"), ("dev", "&[std::path::PathBuf]", "dev")]).rstrip("\n"), "]", ""]
    (BUNDLE / "horizon-mixin.toml").write_text("\n".join(out), encoding="utf8")
    # The JEI window, Mob Drops, World Gen, the creative panel and the W guide: this mod's own copy.
    subprocess.run([sys.executable, str(ROOT.parent / "jei-kit" / "apply.py"), "ironfurnaces", str(BUNDLE)], check=True)
    (BUNDLE / "data").mkdir(exist_ok=True)
    subprocess.run([sys.executable, str(ROOT / "tools" / "make_content.py"), str(JAR), str(BUNDLE / "data" / "ironfurnaces-content.hmod")], check=True)
    # The dev build folder: pinned, with no hooks number, so the pin applies.
    target = BUILD / "mixins" / "ironfurnaces"
    target.mkdir(parents=True, exist_ok=True)
    for old in target.rglob("*"):
        if old.is_file():
            old.unlink()
    for f in BUNDLE.rglob("*"):
        if f.is_file():
            to = target / f.relative_to(BUNDLE)
            to.parent.mkdir(parents=True, exist_ok=True)
            to.write_bytes(f.read_bytes())
    (BUILD / "horizon-native.json").write_text(json.dumps({"game": "26.3", "horizon": HORIZON, "id": "ironfurnaces", "mixins": MIXINS, "schema": 1, "version": VERSION}, indent=2), encoding="utf8")
    print("bundle written:", len(HOOKS) + 6, "injections")


if __name__ == "__main__":
    main()
