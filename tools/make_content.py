"""Iron Furnaces' blocks, items, recipes and textures as a Horizon content mod, from the jar.

Usage: python make_content.py <ironfurnaces jar> <out.hmod>

What the jar has goes in as it is, with what Horizon needs on top:
- each block's declaration: a template with a block entity (the beacon, as Elevator), the strength and sound of the vanilla
  block IF copies, and its state properties;
- the `c:` tags NeoForge gives and Horizon doesn't, written out with vanilla's items;
- loot table conditions under `type`, Horizon's key;
- the menu, the creative tab and the mod's guide pages.
"""

import io
import json
import re
import sys
import zipfile

ID = "ironfurnaces"

# Registration: Properties.ofFullCopy(<block>), then destroyTime(3) in BlockIronFurnaceBase. (resistance, sound, tool)
FURNACES = {
    "iron_furnace": (6.0, "minecraft:iron_block", True),
    "gold_furnace": (6.0, "minecraft:gold_block", True),
    "diamond_furnace": (6.0, "minecraft:diamond_block", True),
    "emerald_furnace": (6.0, "minecraft:emerald_block", True),
    "obsidian_furnace": (1200.0, "minecraft:obsidian", True),
    "crystal_furnace": (0.3, "minecraft:glass", False),
    "netherite_furnace": (1200.0, "minecraft:netherite_block", True),
    "copper_furnace": (6.0, "minecraft:copper_block", True),
    "silver_furnace": (6.0, "minecraft:iron_block", True),
    # BlockMillionFurnace: explosionResistance(0), so a blast can reach it (a Rainbow Core comes out).
    "million_furnace": (0.0, "minecraft:diamond_block", True),
    "allthemodium_furnace": (6.0, "minecraft:netherite_block", True),
    "vibranium_furnace": (6.0, "minecraft:netherite_block", True),
    "unobtainium_furnace": (6.0, "minecraft:netherite_block", True),
}

ORDER = [
    "copper_furnace", "iron_furnace", "silver_furnace", "gold_furnace", "diamond_furnace", "emerald_furnace",
    "crystal_furnace", "obsidian_furnace", "netherite_furnace", "allthemodium_furnace", "vibranium_furnace",
    "unobtainium_furnace", "million_furnace", "heater", "item_heater",
    "upgrade_copper", "upgrade_iron", "upgrade_iron2", "upgrade_silver", "upgrade_silver2", "upgrade_gold", "upgrade_gold2",
    "upgrade_diamond", "upgrade_emerald", "upgrade_crystal", "upgrade_obsidian", "upgrade_obsidian2", "upgrade_netherite",
    "upgrade_allthemodium", "upgrade_vibranium", "upgrade_unobtainium",
    "augment_blasting", "augment_smoking", "augment_speed", "augment_fuel", "augment_factory", "augment_generator",
    "item_copy", "item_spooky", "item_xmas", "rainbow_core", "rainbow_plating", "rainbow_coal",
]

# NeoForge's tags these recipes use, as far as vanilla's items fill them. A tag of a metal vanilla doesn't have stays empty, so
# its recipes wait for a mod that adds one (as they do on NeoForge).
C_TAGS = {
    "ingots/iron": ["minecraft:iron_ingot"],
    "ingots/gold": ["minecraft:gold_ingot"],
    "ingots/copper": ["minecraft:copper_ingot"],
    "ingots/netherite": ["minecraft:netherite_ingot"],
    "ingots/allthemodium": [],
    "ingots/vibranium": [],
    "ingots/unobtainium": [],
    "gems/diamond": ["minecraft:diamond"],
    "gems/emerald": ["minecraft:emerald"],
    "gems/amethyst": ["minecraft:amethyst_shard"],
    "gems/lapis": ["minecraft:lapis_lazuli"],
    "gems/quartz": ["minecraft:quartz"],
    "glass_blocks/colorless": ["minecraft:glass"],
    "stones": ["minecraft:stone", "minecraft:andesite", "minecraft:diorite", "minecraft:granite", "minecraft:tuff", "minecraft:deepslate"],
    "dusts/redstone": ["minecraft:redstone"],
    "obsidians": ["minecraft:obsidian", "minecraft:crying_obsidian"],
    "storage_blocks/gold": ["minecraft:gold_block"],
    "storage_blocks/redstone": ["minecraft:redstone_block"],
    "storage_blocks/allthemodium": [],
    "storage_blocks/vibranium": [],
    "storage_blocks/unobtainium": [],
    "rods/blaze": ["minecraft:blaze_rod"],
    "nether_stars": ["minecraft:nether_star"],
    "dusts/prismarine": ["minecraft:prismarine_shard"],
}


def furnace_block(name):
    resistance, sound, tool = FURNACES[name]
    props = {
        "lit": {"type": "boolean"},
        "facing": {"type": "enum", "values": ["north", "south", "west", "east"]},
        "jovial": {"type": "enum", "values": ["0", "1", "2"]},
        "type": {"type": "enum", "values": ["0", "1", "2"]},
    }
    default = {"facing": "north", "jovial": "0", "lit": "false", "type": "0"}
    if name == "million_furnace":
        props["rainbow"] = {"type": "boolean"}
        default["rainbow"] = "false"
    return {
        "template": "minecraft:beacon",
        "properties": {"hardness": 3.0, "explosion_resistance": resistance, "sounds": sound, "requires_tool": tool},
        "state_properties": props,
        "default_state": default,
    }


def heater_block():
    return {
        "template": "minecraft:beacon",
        "properties": {"hardness": 3.0, "explosion_resistance": 6.0, "sounds": "minecraft:iron_block", "requires_tool": True},
    }


def fix_conditions(value):
    if isinstance(value, dict):
        return {("type" if k == "condition" else k): fix_conditions(v) for k, v in value.items()}
    if isinstance(value, list):
        return [fix_conditions(v) for v in value]
    return value


def strip_required(value):
    """A tag entry `{id, required: false}` of another mod's item goes: Horizon reads plain lists."""
    return [v if isinstance(v, str) else v["id"] for v in value if isinstance(v, str) or v.get("required", True)]


def guide(files):
    pages = {
        "furnaces.md": """---
navigation:
  title: Iron Furnaces
item_ids:
- ironfurnaces:iron_furnace
- ironfurnaces:copper_furnace
- ironfurnaces:silver_furnace
- ironfurnaces:gold_furnace
- ironfurnaces:diamond_furnace
- ironfurnaces:emerald_furnace
- ironfurnaces:crystal_furnace
- ironfurnaces:obsidian_furnace
- ironfurnaces:netherite_furnace
- ironfurnaces:allthemodium_furnace
- ironfurnaces:vibranium_furnace
- ironfurnaces:unobtainium_furnace
---

# Iron Furnaces

Furnaces of every metal, each faster than the last. A furnace's side buttons say what each face does (input, output, fuel), and
hoppers and pipes follow them. Sneak and right-click a furnace with an upgrade to change its tier and keep what's inside.
""",
        "augments.md": """---
navigation:
  title: Augments
item_ids:
- ironfurnaces:augment_blasting
- ironfurnaces:augment_smoking
- ironfurnaces:augment_speed
- ironfurnaces:augment_fuel
- ironfurnaces:augment_factory
- ironfurnaces:augment_generator
---

# Augments

Three slots, one of each colour, opened with the button at the furnace's corner. **Blasting** and **Smoking** (red) limit it to those
recipes. **Speed** and **Fuel Efficiency** (green) trade fuel and cook time. **Factory** (blue) makes it a many-slot machine that
runs on power; **Generator** (blue) makes power from fuel.
""",
        "rainbow.md": """---
navigation:
  title: Rainbow Furnace
item_ids:
- ironfurnaces:million_furnace
- ironfurnaces:rainbow_core
- ironfurnaces:rainbow_plating
- ironfurnaces:rainbow_coal
- ironfurnaces:heater
- ironfurnaces:item_heater
---

# The Rainbow Furnace and wireless heat

The Rainbow Furnace smelts a stack at a time. As a generator it needs one of each tier of furnace, placed by the same player, running
as generators with room in their buffers. A blast takes it apart and leaves its core behind.

A **Wireless Heat Transmitter** takes power. Put a **Receiver** in it to bind them, then in a furnace's fuel slot, and the furnace
burns the transmitter's power.
""",
    }
    import guide_index
    # 3D scenes (drag to turn): every tier side by side, furnaces with their hoppers and AE2, and the heater.
    north = {"facing": "north"}
    tiers = ["copper", "iron", "silver", "gold", "diamond", "emerald", "crystal", "obsidian", "netherite", "million"]
    pages["furnaces.md"] += "\n" + guide_index.scene(files, ID, "tiers", [(i, 0, 0, f"{ID}:{t}_furnace", north) for i, t in enumerate(tiers)])
    pages["furnaces.md"] += (
        "\nHoppers follow the sides: one on top feeds the input, one under it takes the result.\n\n"
        + guide_index.scene(files, ID, "automation", [
            (0, 1, 0, "minecraft:hopper", {"facing": "down"}), (0, 0, 0, f"{ID}:iron_furnace", north), (0, -1, 0, "minecraft:hopper", {"facing": "down"})])
        # How it works with AE2: shown only when AE2 is installed too.
        + guide_index.if_mods(["ae2:pattern_provider"],
            "\nWith AE2, a pattern provider on top fills it, and a face set to Input/Output with Auto Output sends the "
            "result back:\n\n"
            + guide_index.scene(files, ID, "automation_ae2", [
                (0, 0, 0, f"{ID}:diamond_furnace", north), (0, 1, 0, "ae2:pattern_provider"), (0, 1, 1, "ae2:fluix_glass_cable"), (0, 1, 2, "ae2:controller")])))
    pages["rainbow.md"] += "\n" + guide_index.scene(files, ID, "heater", [(0, 0, 0, f"{ID}:heater", north), (2, 0, 0, f"{ID}:iron_furnace", north)])
    guide_index.write(files, ID, pages)


def main(jar, out):
    source = zipfile.ZipFile(jar)
    files = {}
    for name in source.namelist():
        if name.endswith("/") or not name.startswith(("assets/", "data/")):
            continue
        if name.startswith(f"data/{ID}/loot_table/"):
            files[name] = json.dumps(fix_conditions(json.loads(source.read(name))), indent=2).encode()
            continue
        if name.startswith(f"data/{ID}/tags/item/") and name.endswith(".json"):
            doc = json.loads(source.read(name))
            doc["values"] = strip_required(doc.get("values", []))
            files[name] = json.dumps(doc, indent=2).encode()
            continue
        if name.startswith("data/c/tags/item/") and name.endswith(".json"):
            doc = json.loads(source.read(name))
            doc["values"] = strip_required(doc.get("values", []))
            files[name] = json.dumps(doc, indent=2).encode()
            continue
        if name.startswith(f"data/{ID}/recipe/") and name.endswith(".json"):
            # (NeoForge's `neoforge:conditions` say "if the tag isn't empty": an empty tag already matches nothing.)
            doc = json.loads(source.read(name))
            doc.pop("neoforge:conditions", None)
            files[name] = json.dumps(doc, indent=2).encode()
            continue
        files[name] = source.read(name)
    for name in FURNACES:
        files[f"data/{ID}/horizon/block/{name}.json"] = json.dumps(furnace_block(name), indent=2).encode()
    files[f"data/{ID}/horizon/block/heater.json"] = json.dumps(heater_block(), indent=2).encode()
    # Items that aren't blocks. Registration's stacksTo(1): the heater receiver, the settings linker, rainbow coal (and its 5120 uses).
    block_items = set(FURNACES) | {"heater"}
    for name in ORDER:
        if name in block_items:
            continue
        declaration = {"template": "minecraft:paper"}
        if name in ("item_heater", "item_copy", "rainbow_coal"):
            declaration["max_stack_size"] = 1
        if name == "rainbow_coal":
            declaration["components"] = {"minecraft:max_damage": 5120, "minecraft:damage": 0}
        files[f"data/{ID}/horizon/item/{name}.json"] = json.dumps(declaration, indent=2).encode()
    for tag, values in C_TAGS.items():
        files[f"data/c/tags/item/{tag}.json"] = json.dumps({"replace": False, "values": values}, indent=2).encode()
    # Vanilla's own tag for a soul fire block, in case the base game's list isn't under the same name.
    # The menus: all 19 slots of BlockIronFurnaceTileBase; the heater's one.
    files[f"data/{ID}/horizon/menu/furnace.json"] = json.dumps({"slots": 19, "data_slots": 36}, indent=2).encode()
    files[f"data/{ID}/horizon/menu/heater.json"] = json.dumps({"slots": 1, "data_slots": 2}, indent=2).encode()
    files[f"data/{ID}/horizon/creative_tab/main.json"] = json.dumps({
        "title": "itemGroup.ironfurnaces",
        "icon": f"{ID}:iron_furnace",
        "items": [f"{ID}:{name}" for name in ORDER],
    }, indent=2).encode()
    guide(files)
    files["horizon-mod.toml"] = "\n".join([
        "# Iron Furnaces' blocks and items, from ironfurnaces-neoforge-26.1.2-4.4.2.jar by ironfurnaces-horizon/tools/make_content.py.",
        f'id = "{ID}"',
        'name = "Iron Furnaces (blocks and items)"',
        'version = "4.4.4"',
        'description = "Iron Furnaces\' blocks, recipes and textures."',
        'authors = ["Qelifern (pizzaatime)", "XenoMustache", "Astryxion"]',
        'api = "0.2"',
        'side = "both"',
        'capabilities = ["registries"]',
        "",
    ]).encode()
    buffer = io.BytesIO()
    with zipfile.ZipFile(buffer, "w") as z:
        for name in sorted(files):
            info = zipfile.ZipInfo(name, date_time=(2026, 1, 1, 0, 0, 0))
            info.compress_type = zipfile.ZIP_DEFLATED
            info.external_attr = 0o100644 << 16
            z.writestr(info, files[name])
    with open(out, "wb") as f:
        f.write(buffer.getvalue())
    print(len(files), "files,", len(buffer.getvalue()), "bytes")


if __name__ == "__main__":
    main(sys.argv[1], sys.argv[2])
