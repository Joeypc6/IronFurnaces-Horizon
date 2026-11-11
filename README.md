# Iron Furnaces for Horizon

An unofficial port of [Iron Furnaces](https://github.com/Qelifern/IronFurnaces) 4.4.2 (NeoForge, by Qelifern, XenoMustache and Astryxion;
Apache License 2.0 in its jar) to [Horizon](https://github.com/horizon-rust/horizon), as one native mod. The Java mod was rewritten in Rust for
Horizon's hooks, so no original code runs. The textures, models, recipes and language files are the original mod's.

- `mixins/ironfurnaces`: the mod. `ironfurnaces.rs` is the logic, `data/ironfurnaces-content.hmod` its blocks, items, recipes and textures.
- `tools/make_content.py`: makes the content from the original jar. `tools/make_bundle.py` writes the bundle's `horizon-mixin.toml` and helpers.

## What's there

- Thirteen furnaces and their cook times, tiers and upgrades (sneak and right-click; the contents stay).
- Augments, by right click or on the augment page: blasting, smoking, speed, fuel, factory and generator.
- Factories (two to six rows by tier, auto split) and generators (fuel, food or the blasting recipes, into FE), with power out of any side set to output.
- The side config, auto input and output, and redstone modes, with hoppers and pipes following the sides.
- The Wireless Heat Transmitter and Receiver, the Settings Linker, the spooky and wrapping looks, Rainbow Coal, and the Rainbow Furnace with its core.
- JEI: furnaces as catalysts of smelting, and the generator's recipes. The W guide has pages written for this port.
- `ironfurnaces.toml` in the config folder holds the Java mod's speeds, tiers, energy and generation.
- Smelting, blasting and smoking recipes come from the world, its data packs included, as a vanilla furnace finds them, and a result never goes past its own stack limit (a Million Furnace's 64 at once included).
- Jade shows a furnace's contents and a factory's or generator's energy, as on NeoForge.

## With AE2

A pattern provider fills a furnace through the face it touches, and the result has to come back to the network, as
in AE2 on NeoForge:

- Put the provider on top (the default top face is Input), or set the face it touches to Input.
- Send the result back: set that face to Input/Output and turn Auto Output on, so the furnace pushes it into the
  provider; or put an import bus on a face set to Output. Without either, the job waits for its results forever.
- Fuel goes in from the sides (Fuel), by hand, a hopper or an export bus. A factory takes power instead.

The AE2 port's crafting status knows each furnace's time from its tier, the recipe, the speed or fuel augment and a
factory's Auto Split, so a job's time left is right from its first craft.

## What differs from the Java mod

- **Experience** from breaking a furnace goes straight to whoever broke it, and from taking a result to whoever took it. The Java mod drops orbs. A blast loses it.
- **Unlocking recipes** by taking a result doesn't happen.
- **PMMO's hook and the update checker** are left out. Allthemodium, Vibranium and Unobtainium furnaces are there, but need those metals from another mod.
- **The comparator subtraction mode** the language file mentions isn't in the Java mod's screens either.

## Build

This is a Horizon native mod bundle: Rust mixins compiled into the game. Native mods are unsandboxed code that runs with your rights, so
read what you build.

```bash
horizon-launcher native-build "path/to/folder/with/the/bundle" --trust-native-code
```
