// Iron Furnaces, from BlockIronFurnaceTileBase, BlockIronFurnaceBase, the screens and items of its 4.4.2 jar (Apache-2.0).
use super::Game;
ironfurnaces_optional::kit_use!(guide as kit_guide);
ironfurnaces_optional::if_mod_items!("jei", {
ironfurnaces_optional::kit_use!(layer as kit_layer);
ironfurnaces_optional::kit_use!(window as kit_window);
});
use horizon_client::containers::{ModBlockEntity, ModContainer, ModEnergyBuffer};
use horizon_client::hud::HudBatch;
use horizon_client::items::ItemStack;
use horizon_core::geometry::Direction;
use horizon_world::nbt::{Compound, JavaString, Tag};
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::OnceLock;

type Pos = [i32; 3];

pub const INPUT: usize = 0;
pub const FUEL: usize = 1;
pub const OUTPUT: usize = 2;
pub const AUGMENT_RED: usize = 3;
pub const AUGMENT_GREEN: usize = 4;
pub const AUGMENT_BLUE: usize = 5;
pub const GENERATOR_FUEL: usize = 6;
pub const FACTORY_INPUT: [usize; 6] = [7, 8, 9, 10, 11, 12];
const SLOTS: usize = 19;

// The menu's data slots, read by the screen.
const D_BURN: usize = 0;
const D_BURN_TOTAL: usize = 1;
const D_COOK: usize = 2;
const D_COOK_TOTAL: usize = 3;
const D_GEN: usize = 4;
const D_GEN_TOTAL: usize = 5;
const D_ENERGY: usize = 6;
const D_CAPACITY: usize = 7;
const D_FACTORY_COOK: usize = 8;
const D_FACTORY_TOTAL: usize = 14;
const D_SIDES: usize = 20; // bottom, top, front, back, left, right
const D_AUTO_IN: usize = 26;
const D_AUTO_OUT: usize = 27;
const D_REDSTONE: usize = 28;
const D_AUGMENT_GUI: usize = 29;
const D_SPLIT: usize = 30;
const D_MODE: usize = 31; // 0 furnace, 1 factory, 2 generator
const D_TIER: usize = 32;
const D_RAINBOW: usize = 33;
const D_ENERGY_HI: usize = 34;
const D_CAPACITY_HI: usize = 35;
pub const DATA_SLOTS: usize = 36;

/// A menu's data slot carries 16 bits: a number past that goes in two.
fn wide(data: &[i32], lo: usize, hi: usize) -> i32 {
    (data[lo] & 0xFFFF) | (data[hi] << 16)
}

// Menu buttons.
const B_SPLIT: i32 = 0;
const B_AUGMENT_GUI: i32 = 1;
const B_AUTO_IN: i32 = 2;
const B_AUTO_OUT: i32 = 3;
const B_REDSTONE: i32 = 4; // +0..2
const B_RESET_SIDES: i32 = 7;
const B_SIDE: i32 = 10; // + side * 5 + setting

fn java(s: &str) -> JavaString {
    JavaString::from(s)
}

fn get_i(nbt: &Compound, key: &str, default: i32) -> i32 {
    match nbt.get(&java(key)) {
        Some(Tag::Int(v)) => *v,
        _ => default,
    }
}

fn set_i(nbt: &mut Compound, key: &str, value: i32) {
    nbt.insert(java(key), Tag::Int(value));
}

fn get_f(nbt: &Compound, key: &str) -> f64 {
    match nbt.get(&java(key)) {
        Some(Tag::Double(bits)) => f64::from_bits(*bits),
        _ => 0.0,
    }
}

fn set_f(nbt: &mut Compound, key: &str, value: f64) {
    nbt.insert(java(key), Tag::Double(value.to_bits()));
}

// ---- Config (ironfurnaces.toml, the Java mod's keys and defaults)

struct Config {
    values: HashMap<String, i32>,
}

const DEFAULT_CONFIG: &str = "# Iron Furnaces' options, as the Java mod's.
misc.lightupdates = false
energy.tier_0 = 80000
energy.tier_1 = 200000
energy.tier_2 = 1000000
iron_furnace.speed = 160
gold_furnace.speed = 120
diamond_furnace.speed = 80
emerald_furnace.speed = 40
obsidian_furnace.speed = 20
crystal_furnace.speed = 40
netherite_furnace.speed = 5
copper_furnace.speed = 180
silver_furnace.speed = 140
rainbow_furnace.speed = 20
allthemodium_furnace.speed = 5
vibranium_furnace.speed = 3
unobtainium_furnace.speed = 1
allthemodium_furnace.mult = 16
vibranium_furnace.mult = 32
unobtainium_furnace.mult = 64
iron_furnace.generation = 40
gold_furnace.generation = 160
diamond_furnace.generation = 240
emerald_furnace.generation = 320
obsidian_furnace.generation = 500
crystal_furnace.generation = 360
netherite_furnace.generation = 1000
copper_furnace.generation = 40
silver_furnace.generation = 100
rainbow_furnace.generation = 2000
allthemodium_furnace.generation = 2000
vibranium_furnace.generation = 3000
unobtainium_furnace.generation = 5000
rainbow_furnace.rainbow_generation = 50000
recipeMaxXPLevel.level = 100
copper_furnace.tier = 0
iron_furnace.tier = 0
silver_furnace.tier = 1
gold_furnace.tier = 1
diamond_furnace.tier = 1
emerald_furnace.tier = 1
crystal_furnace.tier = 2
obsidian_furnace.tier = 2
netherite_furnace.tier = 2
rainbow_furnace.tier = 2
allthemodium_furnace.tier = 2
vibranium_furnace.tier = 2
unobtainium_furnace.tier = 2
";

fn config() -> &'static Config {
    static CONFIG: OnceLock<Config> = OnceLock::new();
    CONFIG.get_or_init(|| {
        let text = Game::native_config_dir().ok().and_then(|dir| {
            let file = dir.join("ironfurnaces.toml");
            match std::fs::read_to_string(&file) {
                Ok(text) => Some(text),
                Err(_) => {
                    let _ = std::fs::write(&file, DEFAULT_CONFIG);
                    None
                }
            }
        });
        // misc.lightupdates is read by the light table (light_state.rs), made when content loads.
        let parse = |text: &str, values: &mut HashMap<String, i32>| {
            for line in text.lines() {
                let Some((k, v)) = line.split_once('=') else { continue };
                let (k, v) = (k.trim(), v.trim());
                if let Ok(n) = v.parse() {
                    values.insert(k.to_owned(), n);
                }
            }
        };
        let mut values = HashMap::new();
        parse(DEFAULT_CONFIG, &mut values);
        if let Some(text) = text {
            parse(&text, &mut values);
        }
        Config { values }
    })
}

fn cfg(key: &str) -> i32 {
    config().values.get(key).copied().unwrap_or(0)
}

/// A furnace block's config name.
fn cfg_name(block: &str) -> &str {
    let name = block.strip_prefix("ironfurnaces:").unwrap_or(block);
    if name == "million_furnace" { "rainbow_furnace" } else { name }
}

pub fn is_furnace(block: &str) -> bool {
    block.strip_prefix("ironfurnaces:").is_some_and(|n| n.ends_with("_furnace"))
}

fn is_heater(block: &str) -> bool {
    block == "ironfurnaces:heater"
}

fn is_rainbow(block: &str) -> bool {
    block == "ironfurnaces:million_furnace"
}

fn tier(block: &str) -> i32 {
    cfg(&format!("{}.tier", cfg_name(block)))
}

/// `smelt`: how many at a time.
fn smelt_mult(block: &str) -> i32 {
    match block {
        "ironfurnaces:million_furnace" => 64,
        "ironfurnaces:allthemodium_furnace" | "ironfurnaces:vibranium_furnace" | "ironfurnaces:unobtainium_furnace" => cfg(&format!("{}.mult", cfg_name(block))).min(64),
        _ => 1,
    }
}

fn capacity_of(block: &str) -> i32 {
    match tier(block) {
        1 => cfg("energy.tier_1"),
        2 => cfg("energy.tier_2"),
        _ => cfg("energy.tier_0"),
    }
}

// ---- State that isn't saved

#[derive(Default)]
struct State {
    /// A player clicked an output slot and may have taken the result (SlotIronFurnace.onTake).
    taken: Vec<(u128, Pos)>,
    /// Who has which furnace open.
    open: HashMap<u128, Pos>,
    /// PlayerFurnacesList: each player's furnaces, by the owner's id (the Rainbow Furnace looks through them).
    furnaces: BTreeMap<u128, HashSet<Pos>>,
    /// The pointer on the open screen, from its tooltip hook.
    mouse: [f32; 2],
    config_open: bool,
    /// Screen data as last drawn: which slots show.
    shown: Vec<bool>,
    /// The furnace screen drawn this frame: its container, block, menu data and corner, for its tooltips.
    drawn: Option<(Option<(i32, i32, i32)>, String, Vec<i32>, [f32; 2])>,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::default());
}

// ---- Settings: side settings by the Direction ids (down, up, north, south, west, east), as FurnaceSettings.

const FACES: [Direction; 6] = [Direction::Down, Direction::Up, Direction::North, Direction::South, Direction::West, Direction::East];

fn dir_step(i: usize) -> [i32; 3] {
    [[0, -1, 0], [0, 1, 0], [0, 0, -1], [0, 0, 1], [-1, 0, 0], [1, 0, 0]][i]
}

fn opposite(i: usize) -> usize {
    [1, 0, 3, 2, 5, 4][i]
}

fn int_array(nbt: &Compound, key: &str, len: usize) -> Vec<i32> {
    let mut out = match nbt.get(&java(key)) {
        Some(Tag::IntArray(a)) => a.to_vec(),
        _ => Vec::new(),
    };
    out.resize(len, 0);
    out
}

fn set_int_array(nbt: &mut Compound, key: &str, values: Vec<i32>) {
    nbt.insert(java(key), Tag::IntArray(values.into()));
}

fn side(nbt: &Compound, i: usize) -> i32 {
    int_array(nbt, "Settings", 6)[i]
}

fn set_side(nbt: &mut Compound, i: usize, value: i32) {
    let mut sides = int_array(nbt, "Settings", 6);
    sides[i] = value;
    set_int_array(nbt, "Settings", sides);
}

fn auto_io(nbt: &Compound, i: usize) -> bool {
    int_array(nbt, "AutoIO", 2)[i] != 0
}

/// Direction ids of front, back, left and right for a facing.
fn relative_sides(facing: &str) -> [usize; 4] {
    match facing {
        "south" => [3, 2, 4, 5],
        "west" => [4, 5, 2, 3],
        "east" => [5, 4, 3, 2],
        _ => [2, 3, 5, 4],
    }
}

// ---- What an item is

fn is_heater_item(item: &str) -> bool {
    item == "ironfurnaces:item_heater"
}

fn augment_slot(item: &str) -> Option<usize> {
    match item {
        "ironfurnaces:augment_blasting" | "ironfurnaces:augment_smoking" => Some(AUGMENT_RED),
        "ironfurnaces:augment_speed" | "ironfurnaces:augment_fuel" => Some(AUGMENT_GREEN),
        "ironfurnaces:augment_factory" | "ironfurnaces:augment_generator" => Some(AUGMENT_BLUE),
        _ => None,
    }
}

fn augment_value(stack: &Option<ItemStack>) -> i32 {
    match stack.as_ref().map(|s| &*s.item) {
        Some("ironfurnaces:augment_blasting" | "ironfurnaces:augment_speed" | "ironfurnaces:augment_factory") => 1,
        Some("ironfurnaces:augment_smoking" | "ironfurnaces:augment_fuel" | "ironfurnaces:augment_generator") => 2,
        _ => 0,
    }
}

fn is_item(stack: &Option<ItemStack>, item: &str) -> bool {
    stack.as_ref().is_some_and(|s| &*s.item == item)
}

// ---- The block entity

pub fn block_entity(block: &str, entity: &mut Option<(String, Compound)>) {
    if entity.is_none() && (is_furnace(block) || is_heater(block)) {
        *entity = Some((block.to_owned(), Compound::new()));
    }
}

pub fn block_entity_ticks(block: &str, ticks: &mut bool) {
    if is_furnace(block) || is_heater(block) {
        *ticks = true;
    }
}

pub fn block_entity_container(block: &str, out: &mut Option<ModContainer>) {
    if is_furnace(block) {
        // Which face reaches what depends on the furnace's own settings: `container_accepts` and `hidden from
        // outputs` answer for each (IgetSlotsForFace, IcanExtractItem).
        let all: Vec<usize> = vec![INPUT, FUEL, OUTPUT, GENERATOR_FUEL, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18];
        *out = Some(ModContainer {
            slots: SLOTS,
            faces: std::array::from_fn(|_| all.clone()),
            inputs: vec![INPUT, FUEL, GENERATOR_FUEL, 7, 8, 9, 10, 11, 12],
            outputs: vec![OUTPUT, FUEL, GENERATOR_FUEL, 13, 14, 15, 16, 17, 18],
            menu: Some("ironfurnaces:furnace".to_owned()),
        });
    } else if is_heater(block) {
        *out = Some(ModContainer {
            slots: 1,
            faces: std::array::from_fn(|_| Vec::new()),
            inputs: Vec::new(),
            outputs: Vec::new(),
            menu: Some("ironfurnaces:heater".to_owned()),
        });
    }
}

pub fn block_entity_energy(block: &str, out: &mut Option<ModEnergyBuffer>) {
    if is_furnace(block) {
        // FEnergyStorage: a factory takes power, a generator gives it; `energy_limits` holds a plain furnace to neither.
        let c = capacity_of(block);
        *out = Some(ModEnergyBuffer { capacity: c, max_insert: c, max_extract: c, insert_faces: [true; 6], extract_faces: [true; 6] });
    } else if is_heater(block) {
        *out = Some(ModEnergyBuffer { capacity: 1_000_000, max_insert: 1_000_000, max_extract: 0, insert_faces: [true; 6], extract_faces: [false; 6] });
    }
}

fn energy(entity: &ModBlockEntity) -> i32 {
    get_i(&entity.nbt, "energy", 0)
}

fn set_energy(entity: &mut ModBlockEntity, value: i32) {
    let c = capacity_of(&entity.id);
    set_i(&mut entity.nbt, "energy", value.clamp(0, c));
}

fn mode(entity: &ModBlockEntity) -> i32 {
    augment_value(&entity.items[AUGMENT_BLUE])
}

fn recipe_kind(entity: &ModBlockEntity) -> &'static str {
    match entity.items[AUGMENT_RED].as_ref().map(|s| &*s.item) {
        Some("ironfurnaces:augment_smoking") => "minecraft:smoking",
        Some("ironfurnaces:augment_blasting") => "minecraft:blasting",
        _ => "minecraft:smelting",
    }
}

/// The world's recipes (its data packs' and a server's included), as a vanilla furnace finds them.
fn cook_recipe<'a>(game: &'a Game<'_>, kind: &str, stack: &ItemStack) -> Option<&'a horizon_client::crafting::Cooking> {
    game.recipes().cook(kind, stack)
}

fn burn_time(game: &Game<'_>, stack: &ItemStack) -> i32 {
    // ItemRainbowCoal.getBurnTime.
    if &*stack.item == "ironfurnaces:rainbow_coal" {
        return 200;
    }
    game.res.providers().burn_time(game.res.items(), stack, false) as i32
}

fn remainder(game: &Game<'_>, item: &str) -> Option<ItemStack> {
    game.res.items().get(item).and_then(|i| i.crafting_remainder.as_deref()).map(|r| ItemStack::new(r, 1))
}

fn max_stack(game: &Game<'_>, item: &str) -> u32 {
    game.res.items().get(item).map_or(64, |i| i.max_stack)
}

/// getSpeed: the tier's time, scaled by the recipe's own against 200.
fn speed_for_stack(game: &Game<'_>, entity: &ModBlockEntity, stack: &ItemStack) -> i32 {
    let regular = cfg(&format!("{}.speed", cfg_name(&entity.id)));
    let Some(recipe) = cook_recipe(game, recipe_kind(entity), stack) else { return 0 };
    ((f64::from(regular) / (200.0 / f64::from(recipe.time.max(1)))) as i32).max(1)
}

/// getCookTime and getFactoryCookTime.
fn cook_time(game: &Game<'_>, entity: &ModBlockEntity, slot: usize, current: i32) -> i32 {
    match entity.items[slot].as_ref() {
        None => current,
        Some(stack) => cook_time_of(game, entity, stack),
    }
}

/// The ticks one smelt of `stack` takes in this furnace now: its tier, the recipe's time, the green augment.
fn cook_time_of(game: &Game<'_>, entity: &ModBlockEntity, stack: &ItemStack) -> i32 {
    let mut speed = speed_for_stack(game, entity, stack);
    if is_item(&entity.items[AUGMENT_GREEN], "ironfurnaces:augment_speed") {
        speed = (speed / 2).max(1);
    }
    if is_item(&entity.items[AUGMENT_GREEN], "ironfurnaces:augment_fuel") {
        speed = ((f64::from(speed) * 1.25).ceil() as i32).max(1);
    }
    speed.max(1)
}

fn generation(entity: &ModBlockEntity) -> i32 {
    let rf = cfg(&format!("{}.generation", cfg_name(&entity.id)));
    if is_item(&entity.items[AUGMENT_GREEN], "ironfurnaces:augment_speed") {
        rf * 2
    } else if is_item(&entity.items[AUGMENT_GREEN], "ironfurnaces:augment_fuel") {
        (f64::from(rf) * 0.75) as i32
    } else {
        rf
    }
}

/// getSmokingBurnTime.
fn smoking_burn(game: &Game<'_>, stack: &ItemStack) -> i32 {
    game.res.items().get(&stack.item).and_then(|i| i.food).map_or(0, |(n, _)| n as i32 * 800)
}

/// `ironfurnaces:generator_blasting`'s energy for a stack (GeneratorRecipe).
fn generator_energy(game: &Game<'_>, stack: &ItemStack) -> Option<i32> {
    game.native_recipes_of_type("ironfurnaces:generator_blasting").iter().find_map(|r| {
        let ingredient = game.native_recipe_ingredient(r.json.get("ingredient")?)?;
        ingredient.test(stack).then(|| r.json.get("energy").and_then(|e| e.as_i64()).unwrap_or(0) as i32)
    })
}

/// getGeneratorBurn and isItemGeneratorFuel.
fn generator_burn_of(game: &Game<'_>, entity: &ModBlockEntity, stack: &ItemStack) -> i32 {
    let mut burn = match entity.items[AUGMENT_RED].as_ref().map(|s| &*s.item) {
        Some("ironfurnaces:augment_smoking") => smoking_burn(game, stack),
        Some("ironfurnaces:augment_blasting") => generator_energy(game, stack).map_or(0, |e| e / 20),
        _ => burn_time(game, stack),
    };
    if is_item(&entity.items[AUGMENT_GREEN], "ironfurnaces:augment_speed") {
        burn /= 2;
    } else if is_item(&entity.items[AUGMENT_GREEN], "ironfurnaces:augment_fuel") {
        burn *= 2;
    }
    burn
}

fn factory_range(block: &str) -> std::ops::Range<usize> {
    match tier(block) {
        0 => 2..4,
        1 => 1..5,
        _ => 0..6,
    }
}

/// The most of `result` the output slot may hold: the item's own stack limit, at most 64.
fn output_limit(game: &Game<'_>, result: &ItemStack) -> u32 {
    max_stack(game, &result.item).clamp(1, 64)
}

/// How many smelts fit at once: `per` made each, `have` already in a slot that holds `limit`, up to `mult` (1 for a
/// plain furnace) and the `inputs` there are. An empty slot (`have` 0) holds `limit` too.
fn smelt_times(limit: u32, have: u32, per: u32, mult: u32, inputs: u32) -> u32 {
    (limit.saturating_sub(have) / per.max(1)).min(mult.max(1)).min(inputs)
}

/// canSmelt, canFactorySmelt.
fn can_smelt(game: &Game<'_>, entity: &ModBlockEntity, input: usize, output: usize) -> bool {
    let Some(stack) = entity.items[input].as_ref() else { return false };
    let Some(recipe) = cook_recipe(game, recipe_kind(entity), stack) else { return false };
    let limit = output_limit(game, &recipe.result);
    match entity.items[output].as_ref() {
        None => smelt_times(limit, 0, recipe.result.count, 1, 1) == 1,
        Some(out) => out.same_item_same_components(&recipe.result) && smelt_times(limit, out.count, recipe.result.count, 1, 1) == 1,
    }
}

/// smeltItem, smeltItemMult and the factory's: what's made, kept for its experience (setRecipeUsed).
fn smelt(game: &Game<'_>, entity: &mut ModBlockEntity, input: usize, output: usize, wet_sponge: bool) {
    let Some(stack) = entity.items[input].clone() else { return };
    let Some(recipe) = cook_recipe(game, recipe_kind(entity), &stack) else { return };
    let (result, id, experience) = (recipe.result.clone(), recipe.id.clone(), recipe.experience);
    let mult = smelt_mult(&entity.id) as u32;
    let have = entity.items[output].as_ref().map_or(0, |s| s.count);
    let times = smelt_times(output_limit(game, &result), have, result.count, mult, stack.count);
    if times == 0 {
        return;
    }
    match entity.items[output].as_mut() {
        Some(out) => out.count += result.count * times,
        None => {
            let mut made = result.clone();
            made.count *= times;
            entity.items[output] = Some(made);
        }
    }
    add_recipe_used(entity, &id, experience, times);
    if wet_sponge && &*stack.item == "minecraft:wet_sponge" && is_item(&entity.items[FUEL], "minecraft:bucket") {
        entity.items[FUEL] = Some(ItemStack::new("minecraft:water_bucket", 1));
    }
    if let Some(s) = entity.items[input].as_mut() {
        s.count -= times;
        if s.count == 0 {
            entity.items[input] = None;
        }
    }
}

/// RecipesUsed: the crafted count and experience, by recipe, until someone takes the result.
fn add_recipe_used(entity: &mut ModBlockEntity, id: &str, experience: f32, times: u32) {
    let key = java(&format!("RecipesUsed.{id}"));
    let (count, _) = match entity.nbt.get(&key) {
        Some(Tag::Compound(c)) => (get_i(c, "n", 0), ()),
        _ => (0, ()),
    };
    let mut entry = Compound::new();
    set_i(&mut entry, "n", count + times as i32);
    set_f(&mut entry, "xp", f64::from(experience));
    entity.nbt.insert(key, Tag::Compound(entry));
}

/// splitAndSpawnExperience over what's kept; clears it.
fn take_experience(entity: &mut ModBlockEntity) -> i32 {
    let keys: Vec<JavaString> = entity.nbt.iter().filter(|(k, _)| k.to_string().is_ok_and(|s| s.starts_with("RecipesUsed."))).map(|(k, _)| k.clone()).collect();
    let mut total = 0;
    for key in keys {
        if let Some(Tag::Compound(c)) = entity.nbt.remove(&key) {
            let amount = f64::from(get_i(&c, "n", 0)) * get_f(&c, "xp");
            total += amount.floor() as i32;
            let roll = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.subsec_nanos()) % 1000;
            if f64::from(roll) / 1000.0 < amount.fract() {
                total += 1;
            }
        }
    }
    total
}

/// Item moved from the neighbour at `dir` into `slot` (autoIO's input half).
fn pull_into(game: &mut Game<'_>, dimension: &str, entity: &mut ModBlockEntity, pos: Pos, dir: usize, slot: usize, accept: &dyn Fn(&Game<'_>, &ModBlockEntity, &ItemStack) -> bool) {
    let there = [pos[0] + dir_step(dir)[0], pos[1] + dir_step(dir)[1], pos[2] + dir_step(dir)[2]];
    let face = FACES[opposite(dir)];
    let Some(slots) = game.native_container_slots(dimension, there, face) else { return };
    for s in slots {
        let Some(stack) = s.stack.as_ref() else { continue };
        let room = match entity.items[slot].as_ref() {
            Some(have) if !have.same_item_same_components(stack) => continue,
            Some(have) => max_stack(game, &have.item).saturating_sub(have.count),
            None => {
                if !accept(game, entity, stack) {
                    continue;
                }
                max_stack(game, &stack.item)
            }
        };
        if room == 0 {
            return;
        }
        if let Some(got) = game.native_extract_item(dimension, there, face, s.slot, room.min(stack.count)) {
            match entity.items[slot].as_mut() {
                Some(have) => have.count += got.count,
                None => entity.items[slot] = Some(got),
            }
        }
    }
}

/// Item pushed from `slot` into the neighbour at `dir` (autoIO's output half).
fn push_from(game: &mut Game<'_>, entity: &mut ModBlockEntity, pos: Pos, dir: usize, slot: usize) {
    let Some(stack) = entity.items[slot].take() else { return };
    let there = [pos[0] + dir_step(dir)[0], pos[1] + dir_step(dir)[1], pos[2] + dir_step(dir)[2]];
    entity.items[slot] = match game.native_insert_item(there, FACES[opposite(dir)], stack.clone()) {
        Some(rest) => Some(rest),
        None => None,
    };
    let _ = stack;
}

fn has_cook_recipe(game: &Game<'_>, entity: &ModBlockEntity, stack: &ItemStack) -> bool {
    cook_recipe(game, recipe_kind(entity), stack).is_some()
}

/// autoIO, autoIOGenerator and autoFactoryIO.
fn auto_transfer(game: &mut Game<'_>, entity: &mut ModBlockEntity, pos: Pos) {
    let dimension = game.native_dimension().to_owned();
    let (auto_in, auto_out) = (auto_io(&entity.nbt, 0), auto_io(&entity.nbt, 1));
    if !auto_in && !auto_out {
        return;
    }
    let m = mode(entity);
    for dir in 0..6 {
        let setting = side(&entity.nbt, dir);
        if setting == 0 {
            continue;
        }
        match m {
            0 => {
                if auto_in && matches!(setting, 1 | 3) {
                    pull_into(game, &dimension, entity, pos, dir, INPUT, &has_cook_recipe);
                }
                if auto_in && setting == 4 {
                    pull_into(game, &dimension, entity, pos, dir, FUEL, &|g, _, s| burn_time(g, s) > 0 || is_heater_item(&s.item));
                }
                if auto_out && setting == 4 && entity.items[FUEL].as_ref().is_some_and(|f| burn_time(game, f) <= 0 && !is_heater_item(&f.item)) {
                    push_from(game, entity, pos, dir, FUEL);
                }
                if auto_out && matches!(setting, 2 | 3) {
                    push_from(game, entity, pos, dir, OUTPUT);
                }
            }
            1 => {
                let range = factory_range(&entity.id);
                if auto_in && matches!(setting, 1 | 3) {
                    for j in range.clone() {
                        pull_into(game, &dimension, entity, pos, dir, FACTORY_INPUT[j], &has_cook_recipe);
                    }
                }
                if auto_out && matches!(setting, 2 | 3) {
                    for j in range {
                        push_from(game, entity, pos, dir, FACTORY_INPUT[j] + 6);
                    }
                }
            }
            _ => {
                if setting != 4 {
                    continue;
                }
                if auto_in {
                    pull_into(game, &dimension, entity, pos, dir, GENERATOR_FUEL, &|g, e, s| &*s.item != "minecraft:bucket" && !is_heater_item(&s.item) && generator_burn_of(g, e, s) > 0);
                }
                if auto_out && entity.items[GENERATOR_FUEL].as_ref().is_some_and(|f| generator_burn_of(game, entity, f) <= 0) {
                    push_from(game, entity, pos, dir, GENERATOR_FUEL);
                }
            }
        }
    }
}

/// energyOut: to every side set to output, split by what each takes.
fn energy_out(game: &mut Game<'_>, entity: &mut ModBlockEntity, pos: Pos) {
    for dir in 0..6 {
        if !matches!(side(&entity.nbt, dir), 2 | 3) {
            continue;
        }
        let have = energy(entity);
        if have <= 0 {
            return;
        }
        let there = [pos[0] + dir_step(dir)[0], pos[1] + dir_step(dir)[1], pos[2] + dir_step(dir)[2]];
        let moved = game.native_energy_insert(there, FACES[opposite(dir)], have, false);
        set_energy(entity, have - moved);
    }
}

fn redstone_on(game: &mut Game<'_>, pos: Pos) -> bool {
    let dimension = game.native_dimension().to_owned();
    game.native_neighbor_signal(&dimension, pos) > 0
}

fn set_prop(game: &mut Game<'_>, pos: Pos, key: &str, value: &str) {
    if game.native_block_property(pos, key).as_deref() != Some(value) {
        game.native_set_block_property_with(pos, key, value, 3);
    }
}

/// split, fillEmptySlots and getSplitCounts: the factory's inputs even each other out. `full` is `split(true, ..)`, after
/// a smelt; otherwise it splits only while a row is empty. A row's stack never goes to nothing here, so what cooks
/// keeps cooking: empty rows get one each from the first row with more, then the rows of the same item share evenly.
fn auto_split(entity: &mut ModBlockEntity, range: std::ops::Range<usize>, full: bool) {
    // canAutoSplitWork: not the Million, Allthemodium, Vibranium or Unobtainium furnaces.
    if smelt_mult(&entity.id) > 1 {
        return;
    }
    let slots: Vec<usize> = range.map(|j| FACTORY_INPUT[j]).collect();
    if !full && slots.iter().all(|&s| entity.items[s].is_some()) {
        return;
    }
    // itemToCheck: the last row's stack.
    let Some(check) = slots.iter().rev().find_map(|&s| entity.items[s].clone()) else { return };
    // fillEmptySlots.
    let mut amount = slots.iter().filter(|&&s| entity.items[s].is_none()).count() as u32;
    if amount > 0 {
        for &j in &slots {
            let Some(stack) = entity.items[j].clone() else { continue };
            if stack.count > 1 && amount > 0 {
                if amount >= stack.count {
                    amount = stack.count - 1;
                }
                if let Some(s) = entity.items[j].as_mut() {
                    s.count -= amount;
                }
                for &e in &slots {
                    if entity.items[e].is_none() && amount > 0 {
                        let mut one = stack.clone();
                        one.count = 1;
                        entity.items[e] = Some(one);
                        amount -= 1;
                    }
                }
                break;
            }
        }
    }
    // getSplitCounts over the rows holding that item, the odd ones to the first rows.
    let rows: Vec<usize> = slots.iter().copied().filter(|&s| entity.items[s].as_ref().is_some_and(|i| i.item == check.item)).collect();
    let n = rows.len() as u32;
    if n == 0 {
        return;
    }
    let sum: u32 = rows.iter().map(|&s| entity.items[s].as_ref().map_or(0, |i| i.count)).sum();
    let (each, extra) = (sum / n, sum % n);
    for (i, &s) in rows.iter().enumerate() {
        if let Some(stack) = entity.items[s].as_mut() {
            stack.count = each + u32::from((i as u32) < extra);
        }
    }
}

// ---- AE2: how long a furnace takes, for a crafting job's time left

ironfurnaces_optional::if_mod_items!("port_ae2_me", {
/// A pattern's craft in this furnace: the input it smelts, times the tier's speed and the augments, over what it
/// smelts at once (the higher tiers' multiplier, a factory's slots when Auto Split spreads the input over them).
fn ae2_machine(game: &Game<'_>, pos: [i32; 3], block: &str, inputs: &[(&str, i64)]) -> Option<crate::game::port_ae2_me::MachineTime> {
    if !is_furnace(block) {
        return None;
    }
    let Some(horizon_client::containers::BlockEntity::Mod(entity)) = game.world.block_entities.get(&(pos[0], pos[1], pos[2])) else { return None };
    let m = mode(entity);
    if m == 2 || entity.items.len() < SLOTS {
        return None;
    }
    let kind = recipe_kind(entity);
    let (stack, count) = inputs.iter().find_map(|(id, n)| {
        let stack = ItemStack::new(id, 1);
        cook_recipe(game, kind, &stack).map(|_| (stack, (*n).max(1)))
    })?;
    let each = f64::from(cook_time_of(game, entity, &stack));
    let at_once = i64::from(smelt_mult(&entity.id).max(1));
    let lanes = if m == 1 && get_i(&entity.nbt, "AutoSplit", 0) != 0 { factory_range(&entity.id).len() as i64 } else { 1 };
    let done = if m == 1 {
        int_array(&entity.nbt, "FactoryCookTime", 6).into_iter().max().filter(|c| *c > 0)
    } else {
        Some(get_i(&entity.nbt, "CookTime", 0)).filter(|c| *c > 0)
    };
    Some(crate::game::port_ae2_me::MachineTime {
        ticks: each * count as f64 / (at_once * lanes) as f64,
        latency: each * (count as f64 / (at_once * lanes) as f64).ceil(),
        done: done.map(f64::from),
    })
}
});

fn register_ae2() {
    ironfurnaces_optional::if_mod!("port_ae2_me", {
        static ONCE: std::sync::Once = std::sync::Once::new();
        ONCE.call_once(|| crate::game::port_ae2_me::register_machine(ae2_machine));
    } else { () });
}

pub(in crate::game) fn block_entity_tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity) {
    register_ae2();
    if is_heater(&entity.id) {
        heater_tick(pos, entity);
        return;
    }
    if !is_furnace(&entity.id) {
        return;
    }
    entity.items.resize(SLOTS, None);
    if entity.data.len() < DATA_SLOTS {
        entity.data.resize(DATA_SLOTS, 0);
    }
    if entity.nbt.get(&java("hz_placed")).is_none() {
        set_i(&mut entity.nbt, "hz_placed", 1);
        // placeConfig, for a furnace placed by a command too.
        let mut sides = vec![4; 6];
        sides[0] = 2;
        sides[1] = 1;
        set_int_array(&mut entity.nbt, "Settings", sides);
        let speed = cfg(&format!("{}.speed", cfg_name(&entity.id)));
        set_i(&mut entity.nbt, "CookTimeTotal", speed);
    }
    let time = game.world.game_time as i64;
    // PlayerFurnacesList, from each furnace's owner: back as soon as its chunk is loaded.
    if let Some(owner) = owner_of(entity) {
        STATE.with_borrow_mut(|s| {
            s.furnaces.entry(owner).or_default().insert(pos);
        });
    }
    let was_burning = get_i(&entity.nbt, "BurnTime", 0) > 0;
    // currentAugment: a change restarts the burn; a change of the blue one or a generator's red drops the contents.
    let mut current = int_array(&entity.nbt, "Augment", 3);
    for i in 0..3 {
        let value = augment_value(&entity.items[AUGMENT_RED + i]);
        if current[i] != value {
            let generator = current[2] == 2;
            current[i] = value;
            set_i(&mut entity.nbt, "BurnTime", 0);
            set_f(&mut entity.nbt, "GeneratorBurn", 0.0);
            if i == 2 || (generator && i == 0) {
                drop_contents(game, pos, entity);
            }
        }
    }
    set_int_array(&mut entity.nbt, "Augment", current);
    let block = entity.id.clone();
    let m = mode(entity);
    let mut total = get_i(&entity.nbt, "CookTimeTotal", 0);
    total = cook_time(game, entity, INPUT, total);
    set_i(&mut entity.nbt, "CookTimeTotal", total);
    let redstone = get_i(&entity.nbt, "Redstone", 0);
    let stopped = match redstone {
        1 => redstone_on(game, pos),
        2 => !redstone_on(game, pos),
        _ => false,
    };
    if stopped {
        set_i(&mut entity.nbt, "CookTime", 0);
        set_i(&mut entity.nbt, "BurnTime", 0);
        set_prop(game, pos, "lit", "false");
        fill_data(entity, m);
        return;
    }
    match m {
        1 => factory_tick(game, pos, entity, time),
        2 => generator_tick(game, pos, entity, time),
        _ => furnace_tick(game, pos, entity, time, was_burning),
    }
    if time % 24 == 0 {
        let state_type = match entity.items[AUGMENT_RED].as_ref().map(|s| &*s.item) {
            Some("ironfurnaces:augment_smoking") => "1",
            Some("ironfurnaces:augment_blasting") => "2",
            _ => "0",
        };
        set_prop(game, pos, "type", state_type);
        let jovial = get_i(&entity.nbt, "Jovial", 0).to_string();
        set_prop(game, pos, "jovial", &jovial);
    }
    if is_rainbow(&block) {
        rainbow_tick(game, pos, entity);
    }
    // SlotIronFurnace.onTake: who took a result gets its experience.
    let takers: Vec<u128> = STATE.with_borrow_mut(|s| {
        let mine: Vec<u128> = s.taken.iter().filter(|(_, p)| *p == pos).map(|(u, _)| *u).collect();
        s.taken.retain(|(_, p)| *p != pos);
        mine
    });
    for uuid in takers {
        let amount = take_experience(entity);
        if amount > 0 {
            game.native_give_experience(uuid, amount as u32);
        }
    }
    fill_data(entity, mode(entity));
}

fn furnace_tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity, time: i64, was_burning: bool) {
    let mut burn = get_i(&entity.nbt, "BurnTime", 0);
    let mut cook = get_i(&entity.nbt, "CookTime", 0);
    let total = get_i(&entity.nbt, "CookTimeTotal", 200);
    if burn > 0 {
        burn -= 1;
    }
    let has_fuel = entity.items[FUEL].is_some();
    if burn > 0 || (has_fuel && entity.items[INPUT].is_some()) {
        let valid = can_smelt(game, entity, INPUT, OUTPUT);
        if burn <= 0 && valid {
            burn = consume_fuel(game, entity, total);
        }
        if burn > 0 && valid {
            cook += 1;
            if cook >= total {
                cook = 0;
                let next = cook_time(game, entity, INPUT, total);
                set_i(&mut entity.nbt, "CookTimeTotal", next);
                smelt(game, entity, INPUT, OUTPUT, true);
                auto_transfer(game, entity, pos);
            }
        } else {
            cook = 0;
        }
    } else if cook > 0 {
        cook = (cook - 2).clamp(0, total);
    }
    set_i(&mut entity.nbt, "BurnTime", burn);
    set_i(&mut entity.nbt, "CookTime", cook);
    if time % 24 == 0 && cook <= 0 {
        auto_transfer(game, entity, pos);
    }
    if was_burning != (burn > 0) {
        set_prop(game, pos, "lit", if burn > 0 { "true" } else { "false" });
    }
}

/// The furnace's fuel, or the heater's power: how long it now burns.
fn consume_fuel(game: &mut Game<'_>, entity: &mut ModBlockEntity, total: i32) -> i32 {
    let Some(fuel) = entity.items[FUEL].clone() else { return 0 };
    let fuel_augment = is_item(&entity.items[AUGMENT_GREEN], "ironfurnaces:augment_fuel");
    let speed_augment = is_item(&entity.items[AUGMENT_GREEN], "ironfurnaces:augment_speed");
    let burn;
    if is_heater_item(&fuel.item) {
        let Some(bound) = heater_position(&fuel) else { return 0 };
        let stored = game.native_with_mod_block_entity(bound, |e| if is_heater(&e.id) { energy_raw(e) } else { 0 }).unwrap_or(0);
        if stored < 2000 {
            return 0;
        }
        burn = if fuel_augment {
            400 * total / 200
        } else if !speed_augment {
            200 * total / 200
        } else if stored >= 4000 {
            100 * total / 200
        } else {
            0
        };
        if burn > 0 {
            game.native_with_mod_block_entity(bound, |e| {
                let left = energy_raw(e) - 2000;
                set_i(&mut e.nbt, "energy", left);
            });
        }
        set_i(&mut entity.nbt, "RecipesUsed", burn);
        return burn;
    }
    let time = burn_time(game, &fuel);
    burn = if fuel_augment {
        time * total / 200 * 2
    } else if speed_augment {
        time * total / 200 / 2
    } else {
        time * total / 200
    };
    set_i(&mut entity.nbt, "RecipesUsed", burn);
    if burn > 0 && &*fuel.item == "ironfurnaces:rainbow_coal" {
        // ItemRainbowCoal.getCraftingRemainder: itself, a use worn, 5120 uses in all.
        let damage = fuel.damage + 1;
        if damage >= 5120 {
            entity.items[FUEL] = None;
        } else if let Some(f) = entity.items[FUEL].as_mut() {
            f.damage = damage;
        }
        return burn;
    }
    if burn > 0 {
        let rest = remainder(game, &fuel.item);
        match rest {
            Some(rest) => entity.items[FUEL] = Some(rest),
            None => {
                if let Some(f) = entity.items[FUEL].as_mut() {
                    f.count -= 1;
                    if f.count == 0 {
                        entity.items[FUEL] = None;
                    }
                }
            }
        }
    }
    burn
}

fn energy_raw(e: &ModBlockEntity) -> i32 {
    get_i(&e.nbt, "energy", 0)
}

fn factory_tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity, time: i64) {
    let range = factory_range(&entity.id);
    if get_i(&entity.nbt, "AutoSplit", 0) != 0 {
        auto_split(entity, range.clone(), false);
    }
    let mut cook = int_array(&entity.nbt, "FactoryCookTime", 6);
    let mut totals = int_array(&entity.nbt, "FactoryTotalCookTime", 6);
    for j in range.clone() {
        let slot = FACTORY_INPUT[j];
        totals[j] = cook_time(game, entity, slot, totals[j]);
        let used_key = format!("UsedRF{j}");
        let mut used = get_f(&entity.nbt, &used_key);
        if let Some(stack) = entity.items[slot].clone() {
            let valid = can_smelt(game, entity, slot, slot + 6);
            if valid {
                let recipe_time = cook_recipe(game, recipe_kind(entity), &stack).map_or(200, |r| r.time as i32);
                let base = recipe_time * 20;
                let need = if is_item(&entity.items[AUGMENT_GREEN], "ironfurnaces:augment_speed") {
                    base * 2
                } else if is_item(&entity.items[AUGMENT_GREEN], "ironfurnaces:augment_fuel") {
                    base / 2
                } else {
                    base
                };
                if energy(entity) >= need || cook[j] > 0 {
                    cook[j] += 1;
                    let step = f64::from(need) / f64::from(totals[j].max(1));
                    used += step;
                    set_energy(entity, (f64::from(energy(entity)) - step) as i32);
                    if cook[j] >= totals[j] {
                        cook[j] = 0;
                        if used < f64::from(need) {
                            set_energy(entity, (f64::from(energy(entity)) - (f64::from(need) - used)) as i32);
                        }
                        used = 0.0;
                        totals[j] = cook_time(game, entity, slot, totals[j]);
                        if get_i(&entity.nbt, "AutoSplit", 0) != 0 {
                            auto_split(entity, range.clone(), true);
                        }
                        smelt(game, entity, slot, slot + 6, false);
                        auto_transfer(game, entity, pos);
                    }
                }
            }
        } else {
            cook[j] = 0;
        }
        set_f(&mut entity.nbt, &used_key, used);
    }
    let lit = cook.iter().any(|c| *c > 0);
    set_prop(game, pos, "lit", if lit { "true" } else { "false" });
    set_int_array(&mut entity.nbt, "FactoryCookTime", cook.clone());
    set_int_array(&mut entity.nbt, "FactoryTotalCookTime", totals);
    if time % 24 == 0 && cook.iter().any(|c| *c <= 0) {
        auto_transfer(game, entity, pos);
    }
}

fn generator_tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity, time: i64) {
    let capacity = capacity_of(&entity.id);
    let mut burn = get_f(&entity.nbt, "GeneratorBurn");
    let mut recent = get_i(&entity.nbt, "GeneratorRecent", 0);
    let mut gotten = get_f(&entity.nbt, "GottenRF");
    if energy(entity) < capacity {
        if let Some(fuel) = entity.items[GENERATOR_FUEL].clone()
            && burn <= 0.0
        {
            let b = generator_burn_of(game, entity, &fuel);
            if b > 0 {
                burn = f64::from(b);
                recent = b;
                match remainder(game, &fuel.item) {
                    Some(rest) => entity.items[GENERATOR_FUEL] = Some(rest),
                    None => {
                        if let Some(f) = entity.items[GENERATOR_FUEL].as_mut() {
                            f.count -= 1;
                            if f.count == 0 {
                                entity.items[GENERATOR_FUEL] = None;
                            }
                        }
                    }
                }
            }
        }
        set_prop(game, pos, "lit", if burn > 0.0 { "true" } else { "false" });
        if burn > 0.0 {
            let rf = generation(entity);
            let max = f64::from(recent) * 20.0;
            gotten += f64::from(rf);
            set_energy(entity, energy(entity) + rf);
            if burn - f64::from(rf) / 20.0 <= 0.0 {
                if gotten + f64::from(rf) > max && gotten + f64::from(rf) < f64::from(capacity) {
                    let diff = (gotten + f64::from(rf) - max) as i32;
                    set_energy(entity, energy(entity) + rf - diff);
                }
                if gotten + f64::from(rf) < max {
                    let diff = (max - gotten + f64::from(rf)) as i32;
                    set_energy(entity, energy(entity) + rf + diff);
                }
                gotten = 0.0;
            }
            burn -= f64::from(rf) / 20.0;
            if burn <= 0.0 {
                auto_transfer(game, entity, pos);
                burn = 0.0;
            }
        }
    }
    if burn <= 0.0 {
        burn = 0.0;
    }
    set_f(&mut entity.nbt, "GeneratorBurn", burn);
    set_i(&mut entity.nbt, "GeneratorRecent", recent);
    set_f(&mut entity.nbt, "GottenRF", gotten);
    energy_out(game, entity, pos);
    if time % 24 == 0 && burn <= 0.0 {
        auto_transfer(game, entity, pos);
    }
}

/// The Rainbow Furnace as a generator: every tier running as one, one Rainbow generator per player.
fn rainbow_tick(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity) {
    if mode(entity) != 2 {
        set_prop(game, pos, "rainbow", "false");
        return;
    }
    let owner = owner_of(entity);
    let list: Vec<Pos> = STATE.with_borrow(|s| owner.and_then(|o| s.furnaces.get(&o)).map(|l| l.iter().copied().collect()).unwrap_or_default());
    const TIERS: [&str; 9] = ["iron", "gold", "diamond", "emerald", "obsidian", "crystal", "netherite", "copper", "silver"];
    let mut running = [false; 9];
    let mut rainbows = 0;
    for p in list {
        if p == pos {
            rainbows += 1;
            continue;
        }
        let Some((id, burn, stored, m)) = game.native_with_mod_block_entity(p, |e| (e.id.clone(), get_f(&e.nbt, "GeneratorBurn"), get_i(&e.nbt, "energy", 0), mode(e))) else { continue };
        if is_rainbow(&id) && m == 2 {
            rainbows += 1;
        }
        if let Some(i) = TIERS.iter().position(|t| id == format!("ironfurnaces:{t}_furnace"))
            && burn > 0.0
            && stored < capacity_of(&id)
        {
            running[i] = true;
        }
    }
    let generating = owner.is_some() && rainbows <= 1 && running.iter().all(|r| *r);
    set_prop(game, pos, "rainbow", if generating { "true" } else { "false" });
    if generating {
        // rainbowEnergyOut: the whole amount, split over the sides that take it. It doesn't add to the buffer.
        let sides: Vec<usize> = (0..6).filter(|&d| matches!(side(&entity.nbt, d), 2 | 3)).collect();
        if sides.is_empty() {
            return;
        }
        let each = cfg("rainbow_furnace.rainbow_generation") / sides.len() as i32;
        for dir in sides {
            let there = [pos[0] + dir_step(dir)[0], pos[1] + dir_step(dir)[1], pos[2] + dir_step(dir)[2]];
            game.native_energy_insert(there, FACES[opposite(dir)], each, false);
        }
    }
}

fn owner_of(entity: &ModBlockEntity) -> Option<u128> {
    match entity.nbt.get(&java("Owner")) {
        Some(Tag::Long(l)) => {
            let low = match entity.nbt.get(&java("OwnerLow")) {
                Some(Tag::Long(v)) => *v as u64,
                _ => 0,
            };
            Some((u128::from(*l as u64) << 64) | u128::from(low))
        }
        _ => None,
    }
}

fn set_owner(entity: &mut ModBlockEntity, uuid: u128) {
    entity.nbt.insert(java("Owner"), Tag::Long((uuid >> 64) as i64));
    entity.nbt.insert(java("OwnerLow"), Tag::Long(uuid as u64 as i64));
}

fn heater_tick(pos: Pos, entity: &mut ModBlockEntity) {
    entity.items.resize(1, None);
    if entity.data.len() < 2 {
        entity.data.resize(2, 0);
    }
    entity.data[0] = energy_raw(entity);
    entity.data[1] = 1_000_000;
    if let Some(stack) = entity.items[0].as_mut() {
        set_heater_position(stack, pos);
    }
}

fn heater_position(stack: &ItemStack) -> Option<Pos> {
    let t = stack.extra.as_ref()?.other.iter().find(|(k, _)| k == "minecraft:custom_data").map(|(_, t)| t)?;
    let Tag::Compound(c) = t else { return None };
    Some([get_i(c, "HeaterPosX", 0), get_i(c, "HeaterPosY", 0), get_i(c, "HeaterPosZ", 0)])
}

fn set_heater_position(stack: &mut ItemStack, pos: Pos) {
    if heater_position(stack) == Some(pos) {
        return;
    }
    let mut c = Compound::new();
    set_i(&mut c, "HeaterPosX", pos[0]);
    set_i(&mut c, "HeaterPosY", pos[1]);
    set_i(&mut c, "HeaterPosZ", pos[2]);
    let data = stack.extra.get_or_insert_with(Default::default);
    data.other.retain(|(k, _)| k != "minecraft:custom_data");
    data.other.push(("minecraft:custom_data".into(), Tag::Compound(c)));
}

/// dropContents: everything but the augments, where the furnace stands.
fn drop_contents(game: &mut Game<'_>, pos: Pos, entity: &mut ModBlockEntity) {
    for i in 0..SLOTS {
        if (AUGMENT_RED..=AUGMENT_BLUE).contains(&i) {
            continue;
        }
        if let Some(stack) = entity.items[i].take() {
            spawn_stack(game, pos, &stack);
        }
    }
}

fn spawn_stack(game: &mut Game<'_>, pos: Pos, stack: &ItemStack) {
    game.spawn_item(stack.clone(), [f64::from(pos[0]) + 0.5, f64::from(pos[1]) + 0.5, f64::from(pos[2]) + 0.5], [0.0, 0.0, 0.0], 10);
}

fn fill_data(entity: &mut ModBlockEntity, m: i32) {
    let facing = get_i(&entity.nbt, "hz_facing", 2);
    let relative = relative_sides(["north", "north", "north", "south", "west", "east"][facing.clamp(0, 5) as usize]);
    let order = [0, 1, relative[0], relative[1], relative[2], relative[3]];
    let sides = int_array(&entity.nbt, "Settings", 6);
    let auto = int_array(&entity.nbt, "AutoIO", 2);
    let cook = int_array(&entity.nbt, "FactoryCookTime", 6);
    let totals = int_array(&entity.nbt, "FactoryTotalCookTime", 6);
    let block = entity.id.clone();
    let horizon_client::containers::ModBlockEntityFields { nbt, data: d, .. } = &mut **entity;
    d[D_BURN] = get_i(nbt, "BurnTime", 0);
    d[D_BURN_TOTAL] = get_i(nbt, "RecipesUsed", 0);
    d[D_COOK] = get_i(nbt, "CookTime", 0);
    d[D_COOK_TOTAL] = get_i(nbt, "CookTimeTotal", 0);
    d[D_GEN] = (get_f(nbt, "GeneratorBurn")) as i32;
    d[D_GEN_TOTAL] = get_i(nbt, "GeneratorRecent", 0);
    let (energy_now, cap) = (get_i(nbt, "energy", 0), capacity_of(&block));
    d[D_ENERGY] = energy_now & 0xFFFF;
    d[D_ENERGY_HI] = energy_now >> 16;
    d[D_CAPACITY] = cap & 0xFFFF;
    d[D_CAPACITY_HI] = cap >> 16;
    for j in 0..6 {
        d[D_FACTORY_COOK + j] = cook[j];
        d[D_FACTORY_TOTAL + j] = totals[j];
        d[D_SIDES + j] = sides[order[j]];
    }
    d[D_AUTO_IN] = auto[0];
    d[D_AUTO_OUT] = auto[1];
    d[D_REDSTONE] = get_i(nbt, "Redstone", 0);
    d[D_AUGMENT_GUI] = get_i(nbt, "AugmentGUI", 0);
    d[D_SPLIT] = get_i(nbt, "AutoSplit", 0);
    d[D_MODE] = m;
    d[D_TIER] = tier(&block);
    d[D_RAINBOW] = i32::from(is_rainbow(&block));
}

// ---- Who may put what where

/// IisItemValidForSlot.
fn valid_for_slot(game: &Game<'_>, entity: &ModBlockEntity, slot: usize, stack: &ItemStack) -> bool {
    match mode(entity) {
        0 => match slot {
            INPUT => has_cook_recipe(game, entity, stack),
            FUEL => burn_time(game, stack) > 0 || (&*stack.item == "minecraft:bucket" && !is_item(&entity.items[FUEL], "minecraft:bucket")) || is_heater_item(&stack.item),
            _ => false,
        },
        1 => {
            let range = factory_range(&entity.id);
            (7..=12).contains(&slot) && range.contains(&(slot - 7)) && has_cook_recipe(game, entity, stack)
        }
        _ => slot == GENERATOR_FUEL && !is_heater_item(&stack.item) && generator_burn_of(game, entity, stack) > 0,
    }
}

pub(in crate::game) fn container_accepts(game: &Game<'_>, entity: &ModBlockEntity, slot: usize, stack: &ItemStack, face: Option<Direction>, cancel: &mut bool) {
    if !is_furnace(&entity.id) {
        return;
    }
    if !valid_for_slot(game, entity, slot, stack) {
        *cancel = true;
        return;
    }
    // IgetSlotsForFace: only the slots this side's setting reaches.
    if let Some(face) = face {
        let i = FACES.iter().position(|f| *f == face).unwrap_or(0);
        let setting = side(&entity.nbt, i);
        let reached = match mode(entity) {
            0 => match setting {
                1 | 3 => matches!(slot, INPUT | FUEL),
                4 => slot == FUEL,
                _ => false,
            },
            1 => matches!(setting, 1 | 3),
            _ => setting == 4,
        };
        if !reached {
            *cancel = true;
        }
    }
}

/// `native_menu_quick_move`: quickMoveStack. Container slots go to the inventory, a stack from the inventory to the slot it fits.
pub(in crate::game) fn quick_move(game: &mut Game<'_>, entity: &mut ModBlockEntity, slot: usize, container_slots: usize, moving: &mut ItemStack, handled: &mut bool) {
    if !is_furnace(&entity.id) || slot < container_slots {
        return;
    }
    *handled = true;
    let targets: Vec<usize> = match mode(entity) {
        _ if augment_slot(&moving.item).is_some() => augment_slot(&moving.item).into_iter().collect(),
        0 => vec![INPUT, FUEL],
        1 => factory_range(&entity.id).map(|j| FACTORY_INPUT[j]).collect(),
        _ => vec![GENERATOR_FUEL],
    };
    let snapshot = moving.clone();
    for t in targets {
        if !(augment_slot(&snapshot.item) == Some(t) || valid_for_slot(game, entity, t, &snapshot)) {
            continue;
        }
        let most = if augment_slot(&snapshot.item) == Some(t) { 1 } else { max_stack_of(game, &snapshot.item) };
        match entity.items[t].as_mut() {
            Some(have) if have.same_item_same_components(moving) => {
                let n = most.saturating_sub(have.count).min(moving.count);
                have.count += n;
                moving.count -= n;
            }
            None => {
                let n = most.min(moving.count);
                let mut placed = moving.clone();
                placed.count = n;
                entity.items[t] = Some(placed);
                moving.count -= n;
            }
            _ => {}
        }
        if moving.count == 0 {
            return;
        }
    }
}

fn max_stack_of(game: &Game<'_>, item: &str) -> u32 {
    max_stack(game, item)
}

/// A player clicked a slot of a menu: a result may have left.
pub fn menu_clicked(uuid: u128, slot: i32) {
    STATE.with_borrow_mut(|s| {
        if (slot == OUTPUT as i32 || (13..=18).contains(&slot))
            && let Some(pos) = s.open.get(&uuid).copied()
        {
            s.taken.push((uuid, pos));
        }
    });
}

// ---- Placing, breaking, right clicks

/// setPlacedBy and getStateForPlacement: it faces the player; the player's list and, for the rainbow, its owner.
pub(in crate::game) fn placed(game: &mut Game<'_>, pos: Pos) {
    let block = game.native_block(pos);
    if !is_furnace(&block) {
        return;
    }
    let Some(player) = game.native_acting_player() else { return };
    let y = player.yaw.rem_euclid(360.0);
    // Direction.fromYRot, then its opposite.
    let facing = if !(45.0..315.0).contains(&y) {
        "north"
    } else if y < 135.0 {
        "east"
    } else if y < 225.0 {
        "south"
    } else {
        "west"
    };
    set_prop(game, pos, "facing", facing);
    let uuid = player.uuid;
    STATE.with_borrow_mut(|s| {
        s.furnaces.entry(uuid).or_default().insert(pos);
    });
    let rainbow = is_rainbow(&block);
    let index = ["down", "up", "north", "south", "west", "east"].iter().position(|f| *f == facing).unwrap_or(2) as i32;
    game.native_with_mod_block_entity(pos, |e| {
        set_i(&mut e.nbt, "hz_facing", index);
        set_owner(e, uuid);
        let _ = rainbow;
    });
}

/// `native_right_click_block`: BlockIronFurnaceBase.useWithoutItem and the items' useOn.
pub(in crate::game) fn right_click(game: &mut Game<'_>, pos: Pos, canceled: &mut bool) {
    let block = game.native_block(pos);
    let Some(player) = game.native_acting_player() else { return };
    let held = game.held().cloned();
    let item = held.as_ref().map(|h| h.item.to_string()).unwrap_or_default();
    let sneaking = game.keys.contains(&winit::keyboard::KeyCode::ShiftLeft) || game.keys.contains(&winit::keyboard::KeyCode::ShiftRight) || game.player.as_ref().is_some_and(|p| p.pose == horizon_client::pose::Pose::Crouching);
    if let Some((from, to)) = upgrade_of(&item)
        && sneaking
    {
        let vanilla = block == "minecraft:furnace" && from == "minecraft:furnace";
        if block == from || vanilla {
            upgrade(game, pos, &block, to, player.game_mode == 1);
            *canceled = true;
        }
        return;
    }
    if is_heater(&block) {
        STATE.with_borrow_mut(|s| {
            s.open.insert(player.uuid, pos);
        });
        return;
    }
    if !is_furnace(&block) {
        return;
    }
    STATE.with_borrow_mut(|s| {
        s.open.insert(player.uuid, pos);
    });
    let creative = player.game_mode == 1;
    if let Some(slot) = augment_slot(&item).filter(|_| !sneaking) {
        let old = game.native_with_mod_block_entity(pos, |e| {
            let old = e.items[slot].take();
            e.items[slot] = Some(ItemStack::new(&item, 1));
            old
        });
        if let Some(Some(old)) = old
            && !creative
        {
            spawn_stack(game, [pos[0], pos[1] + 1, pos[2]], &old);
        }
        if !creative {
            game.native_take_items(&item, 1);
        }
        game.with_level(|_, level, _| level.native_play_sound("minecraft:block.anvil.use", [f64::from(pos[0]), f64::from(pos[1]), f64::from(pos[2])], 4, 0.05, 1.0));
        *canceled = true;
    } else if item == "ironfurnaces:item_spooky" && !sneaking {
        game.native_with_mod_block_entity(pos, |e| set_i(&mut e.nbt, "Jovial", 1));
        *canceled = true;
    } else if item == "ironfurnaces:item_xmas" && !sneaking {
        game.native_with_mod_block_entity(pos, |e| set_i(&mut e.nbt, "Jovial", 2));
        *canceled = true;
    } else if item.is_empty() && sneaking {
        game.native_with_mod_block_entity(pos, |e| set_i(&mut e.nbt, "Jovial", 0));
        *canceled = true;
    } else if item == "ironfurnaces:item_copy" {
        copy_settings(game, pos, sneaking, player.uuid);
        *canceled = true;
    } else {
        // interactWith: the config page isn't left open from the last time.
        game.native_with_mod_block_entity(pos, |e| set_i(&mut e.nbt, "AugmentGUI", 0));
    }
}

/// ItemFurnaceCopy and interactCopy: settings onto the item, and from it onto a furnace (sneaking).
fn copy_settings(game: &mut Game<'_>, pos: Pos, sneaking: bool, uuid: u128) {
    if !sneaking {
        let settings = game.native_with_mod_block_entity(pos, |e| (int_array(&e.nbt, "Settings", 6), int_array(&e.nbt, "AutoIO", 2), get_i(&e.nbt, "Redstone", 0), get_i(&e.nbt, "hz_facing", 2)));
        let Some((sides, auto, redstone, facing)) = settings else { return };
        if let Some(held) = game.held_mut() {
            let mut c = Compound::new();
            set_int_array(&mut c, "Settings", sides);
            set_int_array(&mut c, "AutoIO", auto);
            set_i(&mut c, "Redstone", redstone);
            set_i(&mut c, "direction", facing);
            let data = held.extra.get_or_insert_with(Default::default);
            data.other.retain(|(k, _)| k != "minecraft:custom_data");
            data.other.push(("minecraft:custom_data".into(), Tag::Compound(c)));
        }
        game.native_action_bar(uuid, horizon_client::text::Text::literal("Settings copied".to_owned()));
    } else {
        let tag = game.held().and_then(|h| h.extra.as_ref()).and_then(|d| d.other.iter().find(|(k, _)| k == "minecraft:custom_data").map(|(_, t)| t.clone()));
        let Some(Tag::Compound(c)) = tag else { return };
        game.native_with_mod_block_entity(pos, |e| {
            set_int_array(&mut e.nbt, "Settings", int_array(&c, "Settings", 6));
            set_int_array(&mut e.nbt, "AutoIO", int_array(&c, "AutoIO", 2));
            set_i(&mut e.nbt, "Redstone", get_i(&c, "Redstone", 0));
        });
        game.native_action_bar(uuid, horizon_client::text::Text::literal("Settings applied".to_owned()));
    }
}

/// ItemUpgrade: (from, to).
fn upgrade_of(item: &str) -> Option<(&'static str, &'static str)> {
    Some(match item {
        "ironfurnaces:upgrade_iron" => ("minecraft:furnace", "ironfurnaces:iron_furnace"),
        "ironfurnaces:upgrade_gold" => ("ironfurnaces:iron_furnace", "ironfurnaces:gold_furnace"),
        "ironfurnaces:upgrade_diamond" => ("ironfurnaces:gold_furnace", "ironfurnaces:diamond_furnace"),
        "ironfurnaces:upgrade_emerald" => ("ironfurnaces:diamond_furnace", "ironfurnaces:emerald_furnace"),
        "ironfurnaces:upgrade_obsidian" => ("ironfurnaces:emerald_furnace", "ironfurnaces:obsidian_furnace"),
        "ironfurnaces:upgrade_crystal" => ("ironfurnaces:diamond_furnace", "ironfurnaces:crystal_furnace"),
        "ironfurnaces:upgrade_obsidian2" => ("ironfurnaces:crystal_furnace", "ironfurnaces:obsidian_furnace"),
        "ironfurnaces:upgrade_netherite" => ("ironfurnaces:obsidian_furnace", "ironfurnaces:netherite_furnace"),
        "ironfurnaces:upgrade_copper" => ("minecraft:furnace", "ironfurnaces:copper_furnace"),
        "ironfurnaces:upgrade_silver" => ("ironfurnaces:copper_furnace", "ironfurnaces:silver_furnace"),
        "ironfurnaces:upgrade_silver2" => ("ironfurnaces:iron_furnace", "ironfurnaces:silver_furnace"),
        "ironfurnaces:upgrade_iron2" => ("ironfurnaces:copper_furnace", "ironfurnaces:iron_furnace"),
        "ironfurnaces:upgrade_gold2" => ("ironfurnaces:silver_furnace", "ironfurnaces:gold_furnace"),
        "ironfurnaces:upgrade_allthemodium" => ("ironfurnaces:netherite_furnace", "ironfurnaces:allthemodium_furnace"),
        "ironfurnaces:upgrade_vibranium" => ("ironfurnaces:allthemodium_furnace", "ironfurnaces:vibranium_furnace"),
        "ironfurnaces:upgrade_unobtainium" => ("ironfurnaces:vibranium_furnace", "ironfurnaces:unobtainium_furnace"),
        _ => return None,
    })
}

fn upgrade(game: &mut Game<'_>, pos: Pos, from: &str, to: &str, creative: bool) {
    let facing = game.native_block_property(pos, "facing").filter(|f| f != "up" && f != "down").unwrap_or_else(|| "north".to_owned());
    let lit = game.native_block_property(pos, "lit").unwrap_or_else(|| "false".to_owned());
    let Some(state) = horizon_client::block_sprites::parse_state(&game.pack.assets.registry, &format!("{to}[facing={facing},lit={lit}]")) else { return };
    let dimension = game.native_dimension().to_owned();
    let item = game.held().map(|h| h.item.to_string()).unwrap_or_default();
    let kept = if from == "minecraft:furnace" {
        // A vanilla furnace's contents drop (ItemUpgrade.useOn).
        for face in FACES {
            let slots = game.native_container_slots(&dimension, pos, face).unwrap_or_default();
            for s in slots {
                if let Some(stack) = s.stack.as_ref()
                    && let Some(got) = game.native_extract_item(&dimension, pos, face, s.slot, stack.count)
                {
                    spawn_stack(game, pos, &got);
                }
            }
        }
        None
    } else {
        game.native_with_mod_block_entity(pos, |e| (std::mem::take(&mut e.items), std::mem::take(&mut e.nbt)))
    };
    game.native_set_block(&dimension, pos, state, 3);
    if let Some((items, nbt)) = kept {
        game.native_with_mod_block_entity(pos, |e| {
            e.items = items;
            e.nbt = nbt;
        });
    }
    let index = ["down", "up", "north", "south", "west", "east"].iter().position(|f| *f == facing).unwrap_or(2) as i32;
    game.native_with_mod_block_entity(pos, |e| set_i(&mut e.nbt, "hz_facing", index));
    if !creative {
        game.native_take_items(&item, 1);
    }
}

/// BlockIronFurnaceBase.onBlockStateChange: experience where it stood. (Its items drop through the container.) A Rainbow
/// Furnace in a blast leaves its core (onExplosionHit).
pub(in crate::game) fn block_drops(game: &mut Game<'_>, pos: Pos, state_block: &str, explosion: bool, drops: &mut Vec<ItemStack>) {
    if !is_furnace(state_block) {
        return;
    }
    STATE.with_borrow_mut(|s| {
        for list in s.furnaces.values_mut() {
            list.remove(&pos);
        }
    });
    // (To who broke it: the orbs AE2-style would need the console, which a player's hook can't reach.)
    let xp = game.native_with_mod_block_entity(pos, take_experience).unwrap_or(0);
    if xp > 0
        && let Some(player) = game.native_acting_player()
    {
        game.native_give_experience(player.uuid, xp as u32);
    }
    if explosion && is_rainbow(state_block) {
        drops.push(ItemStack::new("ironfurnaces:rainbow_core", 1));
    }
}

// ---- The screen

fn lang(key: &str, english: &str) -> String {
    horizon_client::lang::get_or(key, english).to_string()
}

pub fn screen(block: &str, out: &mut Option<super::native_hooks::NativeScreen>) {
    if is_furnace(block) {
        let mut slots: Vec<Option<[f32; 2]>> = vec![None; SLOTS];
        const AT: [[f32; 2]; SLOTS] = [
            [56.0, 17.0], [56.0, 53.0], [116.0, 35.0], [26.0, 35.0], [80.0, 35.0], [134.0, 35.0], [56.0, 40.0],
            [28.0, 6.0], [49.0, 6.0], [70.0, 6.0], [91.0, 6.0], [112.0, 6.0], [133.0, 6.0],
            [28.0, 55.0], [49.0, 55.0], [70.0, 55.0], [91.0, 55.0], [112.0, 55.0], [133.0, 55.0],
        ];
        for (s, at) in slots.iter_mut().zip(AT) {
            *s = Some(at);
        }
        *out = Some(super::native_hooks::NativeScreen {
            width: 176.0,
            height: 166.0,
            slots,
            inventory_y: 84.0,
            inventory_x: 8.0,
            own_labels: true,
            own_background: true,
        });
    } else if is_heater(block) {
        *out = Some(super::native_hooks::NativeScreen {
            width: 176.0,
            height: 166.0,
            // BlockWirelessEnergyHeaterContainer: SlotHeater(te, 0, 80, 37).
            slots: vec![Some([80.0, 37.0])],
            inventory_y: 84.0,
            inventory_x: 8.0,
            own_labels: false,
            own_background: true,
        });
    }
}

/// The slots each page shows (the Slot subclasses' isActive).
fn slot_shown(data: &[i32], slot: usize) -> bool {
    if data.len() < DATA_SLOTS {
        return false;
    }
    if data[D_AUGMENT_GUI] != 0 {
        return (AUGMENT_RED..=AUGMENT_BLUE).contains(&slot);
    }
    match data[D_MODE] {
        0 => matches!(slot, INPUT | FUEL | OUTPUT),
        1 => {
            let range = match data[D_TIER] {
                0 => 2..4,
                1 => 1..5,
                _ => 0..6,
            };
            ((7..=12).contains(&slot) && range.contains(&(slot - 7))) || ((13..=18).contains(&slot) && range.contains(&(slot - 13)))
        }
        _ => slot == GENERATOR_FUEL,
    }
}

/// Does the open screen show this slot? Asked by the game's slot drawing and hit testing.
pub fn client_slot_shown(slot: usize) -> bool {
    STATE.with_borrow(|s| s.shown.get(slot).copied().unwrap_or(true))
}

/// The server's slots that a client can't reach now (see `native_hidden_slots`).
pub(in crate::game) fn hidden_slots(game: &Game<'_>, pos: Pos, size: usize) -> Option<Vec<usize>> {
    let horizon_client::containers::BlockEntity::Mod(e) = game.world.block_entities.get(&(pos[0], pos[1], pos[2]))? else { return None };
    let (block, data) = (e.id.clone(), e.data.clone());
    if !is_furnace(&block) {
        return None;
    }
    Some((0..size).filter(|&i| !slot_shown(&data, i)).collect())
}

fn sprite(game: &Game<'_>, batch: &mut HudBatch, texture: &str, ox: f32, oy: f32, x: f32, y: f32, u: f32, v: f32, w: f32, h: f32) {
    // A texture's part, on the 256 by 256 grid IF draws them on.
    game.native_sprite(batch, texture, [u, v, w, h], [ox + x, oy + y, w, h]);
}

fn gui_texture(block: &str, generator: bool) -> &'static str {
    match (block, generator) {
        ("ironfurnaces:netherite_furnace", true) => "ironfurnaces:gui/furnace_generator_netherite",
        ("ironfurnaces:allthemodium_furnace", true) => "ironfurnaces:gui/furnace_generator_allthemodium",
        ("ironfurnaces:vibranium_furnace", true) => "ironfurnaces:gui/furnace_generator_vibranium",
        ("ironfurnaces:unobtainium_furnace", true) => "ironfurnaces:gui/furnace_generator_unobtainium",
        (_, true) => "ironfurnaces:gui/furnace_generator",
        ("ironfurnaces:netherite_furnace", false) => "ironfurnaces:gui/furnace_netherite",
        ("ironfurnaces:allthemodium_furnace", false) => "ironfurnaces:gui/furnace_allthemodium",
        ("ironfurnaces:vibranium_furnace", false) => "ironfurnaces:gui/furnace_vibranium",
        ("ironfurnaces:unobtainium_furnace", false) => "ironfurnaces:gui/furnace_unobtainium",
        _ => "ironfurnaces:gui/furnace",
    }
}

const WIDGETS: &str = "ironfurnaces:gui/widgets";
const FACTORY: &str = "ironfurnaces:gui/furnace_factory";
const AUGMENTS: &str = "ironfurnaces:gui/augment";

fn inside(rect: [f32; 4], at: [f32; 2]) -> bool {
    at[0] >= rect[0] && at[0] <= rect[0] + rect[2] && at[1] >= rect[1] && at[1] <= rect[1] + rect[3]
}

const R_AUGMENT: [f32; 4] = [161.0, 4.0, 11.0, 11.0];
const R_SPLIT: [f32; 4] = [9.0, 56.0, 14.0, 14.0];
const R_AUTO_IN: [f32; 4] = [-47.0, 8.0, 14.0, 14.0];
const R_AUTO_OUT: [f32; 4] = [-29.0, 8.0, 14.0, 14.0];
const R_RED: [[f32; 4]; 3] = [[-47.0, 66.0, 14.0, 14.0], [-31.0, 66.0, 14.0, 14.0], [-15.0, 66.0, 14.0, 14.0]];
// Bottom, top, front, back, left, right.
const R_SIDES: [[f32; 4]; 6] = [[-32.0, 51.0, 10.0, 10.0], [-32.0, 27.0, 10.0, 10.0], [-32.0, 39.0, 10.0, 10.0], [-20.0, 51.0, 10.0, 10.0], [-44.0, 39.0, 10.0, 10.0], [-20.0, 39.0, 10.0, 10.0]];

fn draw_button(game: &Game<'_>, batch: &mut HudBatch, o: [f32; 2], r: [f32; 4], mouse: [f32; 2], hover_uv: Option<[f32; 2]>, on_uv: Option<[f32; 2]>) {
    if inside(r, mouse)
        && let Some(uv) = hover_uv
    {
        sprite(game, batch, WIDGETS, o[0], o[1], r[0], r[1], uv[0], uv[1], r[2], r[3]);
    }
    if let Some(uv) = on_uv {
        sprite(game, batch, WIDGETS, o[0], o[1], r[0], r[1], uv[0], uv[1], r[2], r[3]);
    }
}

pub(in crate::game) fn screen_draw(game: &Game<'_>, block: &str, data: &[i32], batch: &mut HudBatch, origin: [f32; 2]) {
    STATE.with_borrow_mut(|s| s.drawn = Some((game.container, block.to_owned(), data.to_vec(), origin)));
    let [ox, oy] = origin;
    if is_heater(block) {
        sprite(game, batch, "ironfurnaces:gui/heater", ox, oy, 0.0, 0.0, 0.0, 0.0, 176.0, 166.0);
        if data.len() >= 2 && data[1] > 0 {
            let h = (42.0 * f64::from(data[0]) / f64::from(data[1])) as f32;
            sprite(game, batch, "ironfurnaces:gui/energy", ox, oy, 136.0, 22.0 + 42.0 - h, 0.0, 42.0 - h, 14.0, h);
        }
        return;
    }
    if !is_furnace(block) || data.len() < DATA_SLOTS {
        return;
    }
    let (mouse, config_open) = STATE.with_borrow_mut(|s| {
        s.shown = (0..SLOTS).map(|i| slot_shown(data, i)).collect();
        (s.mouse, s.config_open)
    });
    let m = data[D_MODE];
    let augment_page = data[D_AUGMENT_GUI] != 0;
    let texture = if augment_page {
        AUGMENTS
    } else if m == 1 {
        FACTORY
    } else {
        gui_texture(block, m == 2)
    };
    sprite(game, batch, texture, ox, oy, 0.0, 0.0, 0.0, 0.0, 176.0, 166.0);
    let scaled = |value: i32, total: i32, pixels: i32| -> i32 {
        if total > 0 { (i64::from(value) * i64::from(pixels) / i64::from(total)) as i32 } else { 0 }
    };
    if !augment_page {
        match m {
            0 => {
                if data[D_BURN] > 0 {
                    let i = scaled(data[D_BURN], data[D_BURN_TOTAL].max(1), 13) as f32;
                    sprite(game, batch, texture, ox, oy, 56.0, 36.0 + 12.0 - i, 176.0, 12.0 - i, 14.0, i + 1.0);
                }
                let i = scaled(data[D_COOK], data[D_COOK_TOTAL], 24) as f32;
                sprite(game, batch, texture, ox, oy, 79.0, 34.0, 176.0, 14.0, i + 1.0, 16.0);
            }
            2 => {
                if data[D_GEN] > 0 {
                    let i = scaled(data[D_GEN], data[D_GEN_TOTAL].max(1), 13) as f32;
                    sprite(game, batch, texture, ox, oy, 56.0, 23.0 + 12.0 - i, 176.0, 12.0 - i, 14.0, i + 1.0);
                }
                let h = scaled(wide(data, D_ENERGY, D_ENERGY_HI), wide(data, D_CAPACITY, D_CAPACITY_HI), 42) as f32;
                sprite(game, batch, "ironfurnaces:gui/energy", ox, oy, 109.0, 22.0 + 42.0 - h, 0.0, 42.0 - h, 14.0, h);
            }
            _ => {
                if data[D_TIER] > 0 {
                    sprite(game, batch, FACTORY, ox, oy, 48.0, 5.0, 176.0, 64.0, 18.0, 67.0);
                    sprite(game, batch, FACTORY, ox, oy, 111.0, 5.0, 176.0, 64.0, 18.0, 67.0);
                    if data[D_TIER] == 2 {
                        sprite(game, batch, FACTORY, ox, oy, 27.0, 5.0, 176.0, 64.0, 18.0, 67.0);
                        sprite(game, batch, FACTORY, ox, oy, 132.0, 5.0, 176.0, 64.0, 18.0, 67.0);
                    }
                }
                let h = scaled(wide(data, D_ENERGY, D_ENERGY_HI), wide(data, D_CAPACITY, D_CAPACITY_HI), 42) as f32;
                sprite(game, batch, "ironfurnaces:gui/energy", ox, oy, 9.0, 7.0 + 42.0 - h, 0.0, 42.0 - h, 14.0, h);
                for j in 0..6 {
                    let i = scaled(data[D_FACTORY_COOK + j], data[D_FACTORY_TOTAL + j], 22) as f32;
                    sprite(game, batch, FACTORY, ox, oy, 29.0 + 21.0 * j as f32, 27.0, 176.0, 0.0, 15.0, i + 1.0);
                }
                draw_button(game, batch, origin, R_SPLIT, mouse, Some([70.0, 132.0]), (data[D_SPLIT] != 0).then_some([70.0, 132.0]));
            }
        }
    }
    // The config tab beside the panel.
    if !config_open {
        sprite(game, batch, WIDGETS, ox, oy, -13.0, 2.0, 0.0, 0.0, 12.0, 13.0);
    } else {
        sprite(game, batch, WIDGETS, ox, oy, -56.0, 0.0, 0.0, 13.0, 60.0, 91.0);
        draw_button(game, batch, origin, R_AUTO_IN, mouse, Some([0.0, 132.0]), (data[D_AUTO_IN] != 0).then_some([0.0, 132.0]));
        draw_button(game, batch, origin, R_AUTO_OUT, mouse, Some([14.0, 132.0]), (data[D_AUTO_OUT] != 0).then_some([14.0, 132.0]));
        for (i, r) in R_RED.iter().enumerate() {
            let uv = [28.0 + 14.0 * i as f32, 132.0];
            draw_button(game, batch, origin, *r, mouse, Some(uv), (data[D_REDSTONE] == i as i32).then_some(uv));
        }
        for (i, r) in R_SIDES.iter().enumerate() {
            let s = data[D_SIDES + i];
            if s != 0 {
                draw_button(game, batch, origin, *r, mouse, None, Some([(10 * s - 10) as f32, 104.0]));
            }
        }
        if !augment_page {
            let settings = &data[D_SIDES..D_SIDES + 6];
            let input = settings.iter().any(|s| matches!(*s, 1 | 3));
            let output = settings.iter().any(|s| matches!(*s, 2 | 3));
            let fuel = settings.contains(&4);
            let columns: Vec<f32> = match data[D_TIER] {
                0 => vec![69.0, 90.0],
                1 => vec![48.0, 69.0, 90.0, 111.0],
                _ => vec![27.0, 48.0, 69.0, 90.0, 111.0, 132.0],
            };
            if input {
                if m == 0 {
                    sprite(game, batch, WIDGETS, ox, oy, 55.0, 16.0, 0.0, 114.0, 18.0, 18.0);
                }
                if m == 1 {
                    for x in &columns {
                        sprite(game, batch, WIDGETS, ox, oy, *x, 5.0, 0.0, 114.0, 18.0, 18.0);
                    }
                }
            }
            if output {
                if m == 0 {
                    sprite(game, batch, WIDGETS, ox, oy, 111.0, 30.0, 0.0, 146.0, 26.0, 26.0);
                }
                if m == 1 {
                    for x in &columns {
                        sprite(game, batch, WIDGETS, ox, oy, *x, 54.0, 36.0, 114.0, 18.0, 18.0);
                    }
                }
            }
            if fuel {
                if m == 0 {
                    sprite(game, batch, WIDGETS, ox, oy, 55.0, 52.0, 18.0, 114.0, 18.0, 18.0);
                }
                if m == 2 {
                    sprite(game, batch, WIDGETS, ox, oy, 55.0, 39.0, 18.0, 114.0, 18.0, 18.0);
                }
            }
        }
    }
    // The title and the inventory's label.
    let title = lang(&format!("container.{}", block.replace(':', ".")), &format!("block.{}", block.replace(':', ".")));
    let w = game.native_text_width(&title);
    let x = if m == 0 && !augment_page { 7.0 + 88.0 - w / 2.0 } else { 88.0 - w / 2.0 };
    let y = if m == 1 || data[D_RAINBOW] != 0 { -10.0 } else { 6.0 };
    let color = if m == 1 || data[D_RAINBOW] != 0 { 0xFFFFFF } else { 0x404040 };
    game.native_text(batch, &title, [ox + x, oy + y], color, false);
    game.native_text(batch, &lang("container.inventory", "Inventory"), [ox + 8.0, oy + 73.0], 0x404040, false);
}

pub fn screen_click(block: &str, data: &[i32], at: [f32; 2], right: bool, shift: bool, button: &mut Option<i32>) {
    if !is_furnace(block) || data.len() < DATA_SLOTS {
        return;
    }
    let config_open = STATE.with_borrow(|s| s.config_open);
    if !right && inside(R_AUGMENT, at) {
        *button = Some(B_AUGMENT_GUI);
        return;
    }
    if !right && data[D_MODE] == 1 && data[D_AUGMENT_GUI] == 0 && inside(R_SPLIT, at) {
        *button = Some(B_SPLIT);
        return;
    }
    if !config_open {
        if at[0] >= -17.0 && at[0] <= 1.0 && at[1] >= -1.0 && at[1] <= 17.0 {
            STATE.with_borrow_mut(|s| s.config_open = true);
        }
        return;
    }
    if at[0] >= -13.0 && at[0] <= 0.0 && at[1] >= 5.0 && at[1] <= 18.0 && !inside(R_AUTO_OUT, at) {
        STATE.with_borrow_mut(|s| s.config_open = false);
        return;
    }
    if !right && inside(R_AUTO_IN, at) {
        *button = Some(B_AUTO_IN);
    } else if !right && inside(R_AUTO_OUT, at) {
        *button = Some(B_AUTO_OUT);
    } else if !right && let Some(i) = R_RED.iter().position(|r| inside(*r, at)) {
        *button = Some(B_REDSTONE + i as i32);
    } else if let Some(i) = R_SIDES.iter().position(|r| inside(*r, at)) {
        if shift && i == 2 {
            *button = Some(B_RESET_SIDES);
            return;
        }
        let setting = data[D_SIDES + i];
        let next = if right {
            if setting == 0 { 4 } else { setting - 1 }
        } else if setting == 4 {
            0
        } else {
            setting + 1
        };
        *button = Some(B_SIDE + i as i32 * 5 + next);
    }
}

fn side_words(setting: i32) -> String {
    match setting {
        1 => lang("tooltip.ironfurnaces.gui_input", "Input"),
        2 => lang("tooltip.ironfurnaces.gui_output", "Output"),
        3 => lang("tooltip.ironfurnaces.gui_input_output", "Input/Output"),
        4 => lang("tooltip.ironfurnaces.gui_fuel", "Fuel Input/Eject"),
        _ => lang("tooltip.ironfurnaces.gui_none", "NONE"),
    }
}

/// The game's own path for a screen's tooltips draws them before the JEI panel, which then covers them: they are
/// drawn after it instead, in `screen_post`. This keeps the pointer for the screen's clicks.
pub fn screen_tooltip(block: &str, data: &[i32], at: [f32; 2], _lines: &mut Vec<String>) {
    if is_furnace(block) && data.len() >= DATA_SLOTS {
        STATE.with_borrow_mut(|s| s.mouse = at);
    }
}

/// `native_screen_render_post`: the furnace screen's tooltips, over everything drawn beside the screen.
pub(in crate::game) fn screen_post(game: &Game<'_>, batch: &mut HudBatch, _screen: &super::native_hooks::NativeScreenView<'_>) {
    let Some((key, block, data, origin)) = STATE.with_borrow_mut(|s| s.drawn.take()) else { return };
    if key != game.container || game.carried.is_some() || kit_guide::is_open() {
        return;
    }
    let (cx, cy) = game.cursor;
    let mut lines = Vec::new();
    furnace_tooltip(&block, &data, [cx - origin[0], cy - origin[1]], &mut lines);
    let lines: Vec<(horizon_client::text::Text, u32)> = lines.into_iter().map(|l| (horizon_client::text::Text::literal(l), u32::MAX)).collect();
    game.draw_tooltip_lines(batch, &lines, cx, cy);
}

fn furnace_tooltip(block: &str, data: &[i32], at: [f32; 2], lines: &mut Vec<String>) {
    if !is_furnace(block) || data.len() < DATA_SLOTS {
        return;
    }
    let config_open = STATE.with_borrow(|s| s.config_open);
    if inside(R_AUGMENT, at) {
        let key = if data[D_AUGMENT_GUI] != 0 { ("tooltip.ironfurnaces.gui_open_furnace", "Furnace") } else { ("tooltip.ironfurnaces.gui_open_augments", "Augments") };
        lines.push(lang(key.0, key.1));
    }
    let energy_bar: Option<[f32; 4]> = match (data[D_AUGMENT_GUI] != 0, data[D_MODE]) {
        (false, 1) => Some([9.0, 7.0, 14.0, 42.0]),
        (false, 2) => Some([109.0, 22.0, 14.0, 42.0]),
        _ => None,
    };
    if let Some(bar) = energy_bar
        && inside(bar, at)
    {
        lines.push(format!("{} / {} RF", wide(data, D_ENERGY, D_ENERGY_HI), wide(data, D_CAPACITY, D_CAPACITY_HI)));
    }
    if !config_open {
        if at[0] >= -17.0 && at[0] <= 1.0 && at[1] >= -1.0 && at[1] <= 17.0 {
            lines.push(lang("tooltip.ironfurnaces.gui_open", "Open Config"));
        }
        return;
    }
    if at[0] >= -13.0 && at[0] <= 0.0 && at[1] >= 5.0 && at[1] < 8.0 {
        lines.push(lang("tooltip.ironfurnaces.gui_close", "Hide Config"));
    }
    if inside(R_AUTO_IN, at) {
        lines.push(lang("tooltip.ironfurnaces.gui_auto_input", "Auto Input"));
    }
    if inside(R_AUTO_OUT, at) {
        lines.push(lang("tooltip.ironfurnaces.gui_auto_output", "Auto Output"));
    }
    const RED: [(&str, &str); 3] = [("gui_redstone_ignored", "Redstone Ignored"), ("gui_redstone_low", "Redstone Low"), ("gui_redstone_high", "Redstone High")];
    for (i, r) in R_RED.iter().enumerate() {
        if inside(*r, at) {
            lines.push(lang(&format!("tooltip.ironfurnaces.{}", RED[i].0), RED[i].1));
        }
    }
    const NAMES: [(&str, &str); 6] = [("gui_bottom", "Bottom"), ("gui_top", "Top"), ("gui_front", "Front"), ("gui_back", "Back"), ("gui_left", "Left"), ("gui_right", "Right")];
    for (i, r) in R_SIDES.iter().enumerate() {
        if inside(*r, at) {
            lines.push(lang(&format!("tooltip.ironfurnaces.{}", NAMES[i].0), NAMES[i].1));
            lines.push(format!("§7{}", side_words(data[D_SIDES + i])));
            if i == 2 {
                lines.push(format!("§8{}", lang("tooltip.ironfurnaces.gui_reset", "Reset All")));
            }
        }
    }
}

pub fn menu_button(pos: Pos, entity: &mut ModBlockEntity, id: i32) {
    if !is_furnace(&entity.id) {
        return;
    }
    let _ = pos;
    let toggle = |nbt: &mut Compound, key: &str| {
        let v = get_i(nbt, key, 0);
        set_i(nbt, key, i32::from(v == 0));
    };
    match id {
        B_SPLIT => toggle(&mut entity.nbt, "AutoSplit"),
        B_AUGMENT_GUI => toggle(&mut entity.nbt, "AugmentGUI"),
        B_AUTO_IN | B_AUTO_OUT => {
            let mut auto = int_array(&entity.nbt, "AutoIO", 2);
            let i = (id - B_AUTO_IN) as usize;
            auto[i] = i32::from(auto[i] == 0);
            set_int_array(&mut entity.nbt, "AutoIO", auto);
        }
        4..=6 => set_i(&mut entity.nbt, "Redstone", id - B_REDSTONE),
        B_RESET_SIDES => set_int_array(&mut entity.nbt, "Settings", vec![0; 6]),
        _ if id >= B_SIDE => {
            let (i, setting) = (((id - B_SIDE) / 5) as usize, (id - B_SIDE) % 5);
            let facing = get_i(&entity.nbt, "hz_facing", 2).clamp(0, 5) as usize;
            let relative = relative_sides(["north", "north", "north", "south", "west", "east"][facing]);
            let order = [0, 1, relative[0], relative[1], relative[2], relative[3]];
            if i < 6 {
                set_side(&mut entity.nbt, order[i], setting);
            }
        }
        _ => {}
    }
    let m = mode(entity);
    if entity.data.len() >= DATA_SLOTS {
        fill_data(entity, m);
    }
}

// ---- Item tooltips (the items' appendHoverText)

pub fn tooltip(stack: &ItemStack, lines: &mut Vec<(horizon_client::text::Text, u32)>) {
    use horizon_client::text::Text;
    let item: &str = &stack.item;
    if !item.starts_with("ironfurnaces:") {
        return;
    }
    let mut add = |key: &str, english: &str, color: &str| {
        lines.push((Text::legacy(&format!("{color}{}", lang(key, english))), u32::MAX));
    };
    if let Some(name) = item.strip_prefix("ironfurnaces:")
        && name.ends_with("_furnace")
    {
        let time = cfg(&format!("{}.speed", cfg_name(item)));
        lines.push((Text::legacy(&format!("§9{} §7({time})", lang("tooltip.ironfurnaces.cooktime", "Cooktime"))), u32::MAX));
        if name == "million_furnace" {
            lines.push((Text::legacy(&format!("§7{}", lang("tooltip.ironfurnaces.rainbow_blowup", "Something is telling me to blow it up..."))), u32::MAX));
        }
        return;
    }
    match item {
        "ironfurnaces:augment_blasting" => {
            add("tooltip.ironfurnaces.augment_right_click", "Right-click to insert augment", "§7");
            add("tooltip.ironfurnaces.augment_blasting", "Only allows for blasting recipes.", "§5");
        }
        "ironfurnaces:augment_smoking" => {
            add("tooltip.ironfurnaces.augment_right_click", "Right-click to insert augment", "§7");
            add("tooltip.ironfurnaces.augment_smoking", "Only allows for smoking recipes.", "§5");
        }
        "ironfurnaces:augment_speed" => {
            add("tooltip.ironfurnaces.augment_right_click", "Right-click to insert augment", "§7");
            add("tooltip.ironfurnaces.augment_speed_pro", "+Halves the cooktime for all recipes.", "§a");
            add("tooltip.ironfurnaces.augment_speed_con", "-Uses up twice the amount of fuel.", "§c");
        }
        "ironfurnaces:augment_fuel" => {
            add("tooltip.ironfurnaces.augment_right_click", "Right-click to insert augment", "§7");
            add("tooltip.ironfurnaces.augment_fuel_pro", "+Fuel heats up the furnace twice as much.", "§a");
            add("tooltip.ironfurnaces.augment_fuel_con", "-Slows down the cooktime by 25% for all recipes.", "§c");
        }
        "ironfurnaces:augment_factory" => {
            add("tooltip.ironfurnaces.augment_right_click", "Right-click to insert augment", "§7");
            add("tooltip.ironfurnaces.augment_factory_pro", "+Converts the furnace into a factory.", "§a");
            add("tooltip.ironfurnaces.augment_factory_con", "-Uses power for heat instead of fuel.", "§c");
        }
        "ironfurnaces:augment_generator" => {
            add("tooltip.ironfurnaces.augment_right_click", "Right-click to insert augment", "§7");
            add("tooltip.ironfurnaces.augment_generator_pro", "+Heat from the furnace will generate power.", "§a");
            add("tooltip.ironfurnaces.augment_generator_con", "-The furnace can no longer smelt items.", "§c");
        }
        "ironfurnaces:item_heater" => match heater_position(stack) {
            Some(p) => {
                add("tooltip.ironfurnaces.heater", "Bound to: ", "§7");
                lines.push((Text::legacy(&format!("§7X: {}  Y: {}  Z: {}", p[0], p[1], p[2])), u32::MAX));
            }
            None => {
                add("tooltip.ironfurnaces.heater_not_bound", "Has yet to be bound to an energy source!", "§c");
                add("tooltip.ironfurnaces.heater_tip", "Place in the fuel slot of the furnace", "§7");
                add("tooltip.ironfurnaces.heater_tip1", "Only works from Iron tier and above", "§7");
            }
        },
        "ironfurnaces:heater" => {
            add("tooltip.ironfurnaces.heater_block", "Link with a Wireless Heat Receiver", "§7");
            add("tooltip.ironfurnaces.heater_block1", "Provide this block with Forge Energy", "§7");
        }
        "ironfurnaces:item_xmas" => {
            add("tooltip.ironfurnaces.xmas_right_click", "Right-click on furnace to wrap it as a gift!", "§7");
            add("tooltip.ironfurnaces.xmas1", "But won't the wrappings catch fire?", "§5");
            add("tooltip.ironfurnaces.xmas2", "Sneaking and right-clicking on a furnace opens the gift", "§8");
        }
        "ironfurnaces:item_spooky" => {
            add("tooltip.ironfurnaces.spooky_right_click", "Right-click on furnace to spookalize", "§7");
            add("tooltip.ironfurnaces.spooky1", "3spooky5me", "§5");
            add("tooltip.ironfurnaces.spooky2", "Sneaking and right-clicking on a furnace will cleanse any spookalizations", "§8");
        }
        _ if item.contains(":upgrade_") => add("tooltip.ironfurnaces.upgrade_right_click", "Sneak & right-click to upgrade", "§7"),
        _ => {}
    }
}

// ---- JEI and the guide

pub(in crate::game) fn kit_frame(game: &mut Game<'_>) {
    if !game.native_on("ironfurnaces") {
        return;
    }
    kit_guide::register("ironfurnaces", "Iron Furnaces", "Iron Furnaces guide");
    ironfurnaces_optional::if_mod!("jei", {
        let mut p = kit_layer::Provider::new("ironfurnaces");
        p.station_catalysts = |icon| match icon {
            "minecraft:furnace" => &[
                "ironfurnaces:copper_furnace", "ironfurnaces:iron_furnace", "ironfurnaces:silver_furnace", "ironfurnaces:gold_furnace",
                "ironfurnaces:diamond_furnace", "ironfurnaces:emerald_furnace", "ironfurnaces:crystal_furnace", "ironfurnaces:obsidian_furnace",
                "ironfurnaces:netherite_furnace", "ironfurnaces:allthemodium_furnace", "ironfurnaces:vibranium_furnace", "ironfurnaces:unobtainium_furnace",
                "ironfurnaces:million_furnace",
            ],
            _ => &[],
        };
        p.tabs = generator_tabs;
        kit_layer::register(p);
    } else { () });
}

// RecipeCategoryGeneratorRegular, Blasting and Smoking: what a generator burns, and the power it makes.
ironfurnaces_optional::if_mod_items!("jei", {
fn generator_tabs(game: &Game<'_>, item: &crate::game::jei::Stack, uses: bool) -> Vec<kit_window::Tab> {
    use crate::game::jei::Stack;
    use kit_window::{Card, Tab};
    if !uses {
        return Vec::new();
    }
    let catalysts = || vec!["ironfurnaces:augment_generator".to_owned()];
    let card = |from: Vec<Stack>, rf: i64| Card::Source { from, item: Stack::item("ironfurnaces:augment_generator"), lines: vec![format!("{rf} RF")] };
    let mut out = Vec::new();
    let stack = ItemStack::new(&item.item, 1);
    let every = item.item == "ironfurnaces:augment_generator";
    let mut blasting = Vec::new();
    for r in game.native_recipes_of_type("ironfurnaces:generator_blasting") {
        let Some(ingredient) = r.json.get("ingredient").and_then(|v| game.native_recipe_ingredient(v)) else { continue };
        let rf = r.json.get("energy").and_then(|e| e.as_i64()).unwrap_or(0);
        if every || ingredient.order.iter().any(|id| **id == *item.item) {
            blasting.push(card(ingredient.order.iter().map(|id| Stack::item(id)).collect(), rf));
        }
    }
    if !blasting.is_empty() {
        out.push(Tab { icon: "ironfurnaces:augment_blasting".to_owned(), title: lang("ironfurnaces.jei_category_blasting", "Generator | Blasting"), catalysts: catalysts(), cards: blasting });
    }
    let burn = burn_time(game, &stack);
    if !every && burn > 0 {
        out.push(Tab { icon: "ironfurnaces:augment_generator".to_owned(), title: lang("ironfurnaces.jei_category_regular", "Augment: Generator"), catalysts: catalysts(), cards: vec![card(vec![Stack::item(&item.item)], i64::from(burn) * 20)] });
    }
    let smoking = smoking_burn(game, &stack);
    if !every && smoking > 0 {
        out.push(Tab { icon: "ironfurnaces:augment_smoking".to_owned(), title: lang("ironfurnaces.jei_category_smoking", "Generator | Smoking"), catalysts: catalysts(), cards: vec![card(vec![Stack::item(&item.item)], i64::from(smoking) * 20)] });
    }
    out
}
});

/// BlockIronFurnaceBase.getComparatorInputOverride's slots, for what the furnace is now.
pub fn comparator_slots(entity: &ModBlockEntity) -> Vec<usize> {
    let mut slots = Vec::new();
    match mode(entity) {
        0 => slots.push(INPUT),
        2 => slots.push(GENERATOR_FUEL),
        _ => {}
    }
    slots.extend([FUEL, OUTPUT]);
    if mode(entity) == 1 {
        let t = tier(&entity.id);
        slots.extend([9, 10, 15, 16]);
        if t >= 1 {
            slots.extend([8, 11, 14, 17]);
        }
        if t >= 2 {
            slots.extend([7, 12, 13, 18]);
        }
    }
    slots
}

pub fn update_tag(entity: &ModBlockEntity, tag: &mut Option<Compound>) {
    if is_furnace(&entity.id) && tag.is_none() {
        *tag = Some(Compound::new());
    }
}

/// BlockIronFurnaceBase.animateTick: flames and smoke of what's lit near the player, with their sounds.
pub(in crate::game) fn client_tick(game: &mut Game<'_>) {
    use horizon_client::particles::{ParticleOptions, ParticleType};
    let Some(player) = game.native_player().map(|p| [p.x, p.y, p.z]) else { return };
    let lit: Vec<Pos> = game
        .world
        .block_entities
        .iter()
        .filter_map(|((x, y, z), e)| match e {
            horizon_client::containers::BlockEntity::Mod(m) if is_furnace(&m.id) => Some([*x, *y, *z]),
            _ => None,
        })
        .filter(|p| (f64::from(p[0]) - player[0]).abs() < 32.0 && (f64::from(p[1]) - player[1]).abs() < 32.0 && (f64::from(p[2]) - player[2]).abs() < 32.0)
        .filter(|p| game.native_block_property(*p, "lit").as_deref() == Some("true"))
        .collect();
    let mut seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(1, |d| d.subsec_nanos());
    let mut next = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        f64::from(seed) / f64::from(u32::MAX)
    };
    for p in lit {
        if next() > 0.05 {
            continue;
        }
        let kind = game.native_block_property(p, "type").unwrap_or_default();
        let facing = game.native_block_property(p, "facing").unwrap_or_default();
        let (x, y, z) = (f64::from(p[0]) + 0.5, f64::from(p[1]), f64::from(p[2]) + 0.5);
        let step = match facing.as_str() {
            "south" => [0.0, 1.0],
            "west" => [-1.0, 0.0],
            "east" => [1.0, 0.0],
            _ => [0.0, -1.0],
        };
        let side = next() * 0.6 - 0.3;
        let (dx, dz) = if step[0] != 0.0 { (step[0] * 0.52, side) } else { (side, step[1] * 0.52) };
        let smoke = ParticleOptions::simple(ParticleType::Smoke);
        let flame = ParticleOptions::simple(ParticleType::Flame);
        match kind.as_str() {
            "1" => {
                if next() < 0.1 {
                    game.audio.play("minecraft:block.smoker.smoke", Some([x, y, z]), 1.0, 1.0);
                }
                game.native_add_particle(&smoke, [x, y + 1.1, z], [0.0; 3]);
            }
            "2" => {
                if next() < 0.1 {
                    game.audio.play("minecraft:block.blastfurnace.fire_crackle", Some([x, y, z]), 1.0, 1.0);
                }
                game.native_add_particle(&smoke, [x + dx, y + next() * 9.0 / 16.0, z + dz], [0.0; 3]);
            }
            _ => {
                if next() < 0.1 {
                    game.audio.play("minecraft:block.furnace.fire_crackle", Some([x, y, z]), 1.0, 1.0);
                }
                let h = y + next() * 6.0 / 16.0;
                game.native_add_particle(&smoke, [x + dx, h, z + dz], [0.0; 3]);
                game.native_add_particle(&flame, [x + dx, h, z + dz], [0.0; 3]);
            }
        }
    }
}

#[cfg(test)]
mod stack_limit_tests {
    use super::smelt_times;

    #[test]
    fn empty_output_slot_respects_a_low_limit() {
        // A 4x multiplier furnace, a recipe making 4 of an item that stacks to 16: 4 smelts fill it, never 16 or 64.
        assert_eq!(smelt_times(16, 0, 4, 4, 64), 4);
        assert_eq!(smelt_times(16, 0, 5, 4, 64), 3);
        // An item that stacks to 1: one smelt into an empty slot, whatever the multiplier.
        assert_eq!(smelt_times(1, 0, 1, 4, 64), 1);
        // A result bigger than its own stack limit can't be smelted at all.
        assert_eq!(smelt_times(1, 0, 2, 1, 64), 0);
    }

    #[test]
    fn partly_filled_low_limit_stack_is_not_overfilled() {
        assert_eq!(smelt_times(16, 12, 1, 4, 64), 4);
        assert_eq!(smelt_times(16, 14, 1, 4, 64), 2);
        assert_eq!(smelt_times(16, 16, 1, 4, 64), 0);
        assert_eq!(smelt_times(16, 13, 4, 4, 64), 0);
        assert_eq!(smelt_times(1, 1, 1, 4, 64), 0);
    }

    #[test]
    fn plain_and_normal_cases_are_unchanged() {
        assert_eq!(smelt_times(64, 0, 1, 1, 64), 1);
        assert_eq!(smelt_times(64, 63, 1, 1, 64), 1);
        assert_eq!(smelt_times(64, 64, 1, 1, 64), 0);
        assert_eq!(smelt_times(64, 60, 1, 8, 64), 4);
        assert_eq!(smelt_times(64, 0, 1, 8, 3), 3);
        // A count above the limit (from another mod) never underflows.
        assert_eq!(smelt_times(16, 40, 1, 4, 64), 0);
    }
}
