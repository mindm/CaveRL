# CaveRL

CaveRL is a roguelike written in Rust. It utilises tcod-rs library for graphics.
CaveRL should run on Windows, Linux and Mac. If it doesn't run for you with a simple cargo build,
take a gander at **tcod-rs** [github page][tcod] for further instructions.

# How to run
```sh
$ cargo build
$ cargo run
```

### Implemented features

- [x] Player object
- [x] Player movement
- [x] FOV
- [x] Memory of visited locations
- [x] Monsters (They're empty husks right now)
- [x] ECS
- [x] Melee combat (Just damage/HP as of now)
- [x] Mapgen
- [x] Saving/Loading

### Next features

- [ ] Map-object instead of everything is entity (performance)
- [ ] Attributes/Stats
- [ ] Character progression
- [ ] Inventory
- [ ] Monster AI
- [ ] Items
- [ ] Equipment slots
- [ ] Sub-menu
- [ ] Main menu
- [ ] Debug mode
- [ ] More GUI

### In the distant future

- [ ] More map layouts
- [ ] Spells
- [ ] Different AIs
- [ ] Music
- [ ] ...?


   [tcod]: <https://github.com/tomassedovic/tcod-rs>