extern crate tcod;
extern crate serde;
extern crate rand;
extern crate pathfinding;

//use time::PreciseTime;

#[macro_use]
extern crate recs;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate serde_json;

#[macro_use]
extern crate derive_new;

use std::fs::File;

use recs::*;

use tcod::{Console, RootConsole, BackgroundFlag, FontType, FontLayout};
use tcod::map::{FovAlgorithm, Map};
use tcod::input::Key;
use tcod::input::KeyCode::{Up, Down, Left, Right, Escape};
use tcod::input::KeyCode::{F9, F5};
use tcod::colors::{DARK_GREY, BLACK};

mod grid;
mod mapgen;

use grid::NodeMap;
use mapgen::MapInfo;

const MAP_HEIGHT: i32 = 50;
const MAP_WIDTH: i32 = 80;
const FOV_ALGO: FovAlgorithm = FovAlgorithm::Basic;
const FOV_LIGHT_WALLS: bool = true;


#[derive(PartialEq, Serialize, Deserialize)]
enum Action { Move, Attack, BlockedMove }

// Component Definitions
#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, new)]
struct Position {
    x: i32,
    y: i32
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, new)]
struct Velocity {
    x: i32,
    y: i32,
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, new)]
struct Sprite {
    glyph: char,
}

#[derive( Clone, PartialEq, Debug, Serialize, Deserialize, new)]
struct Name {
    name : String
}

#[derive( Clone, PartialEq, Debug, Serialize, Deserialize, new)]
struct TakeDamage{
    #[new(default)]
    dmg: Vec<i32>
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, new)]
struct Health {
    hp: i32,
    max: i32
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
struct Blocking{}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, new)]
struct Damage{
    dmg : i32
}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
struct Player {}

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
struct Static {}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, new)]
struct Fov{
    fov: NodeMap<bool>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, new)]
struct SpatialMemory {
    memory: NodeMap<bool>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
struct BlockSight{}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, new)]
struct SightRange{
    range: i32
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, new)]
struct MapObject {
    map: MapInfo
}

/// Creates a vector that holds serialized data for all entities in Ecs for defined components.
/// # Examples
///
/// ```
/// let serialized:Vec<Vec<serde_json::value::Value>> = serialize_world!(world; Health, Position, Sprite, Name)
/// ```
macro_rules! serialize_world {
    ($world: ident; $($str: ty),+) => {
        {
            let mut outer_vec = Vec::new();
            for id in $world.iter() {
                let mut vec = Vec::new();
                $(
                    match $world.get::<$str>(id) {
                        Ok(component) => vec.push(json!({stringify!($str) : component})),
                        _ => ()
                    }
                )+
                outer_vec.push(vec);
            }
            outer_vec
        }
    }
}

/// Loads components to world from serde_json::Value.
/// Note that world, json and components are differentiated with semicolon and components by comma.
/// # Examples
/// ```
/// let buffer = File::open("savegame.json").unwrap();
/// let json: serde_json::Value = serde_json::from_reader(buffer).unwrap();
///
/// load_components!(world; json; Health, Position, Sprite, Name);
/// ```
macro_rules! load_components {
($world: ident; $json_value: ident; $($component: ty),+) => {
        {
            let v = $json_value.as_array().unwrap();
            for i in 0..v.len() {
                let new = $world.create_entity();
                for val in v[i].as_array().unwrap().iter() {
                    let key: &String = val.as_object().unwrap().keys().collect::<Vec<_>>()[0];
                    match key.as_ref() {
                        $(
                            stringify!($component) => {
                                let _ = $world.set::<$component>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                            },
                        )+
                        _ => ()
                    }
                }
            }
        }
    }
}

/// Renders all entities which have Position and Sprite components.
/// The rendering is done in 3 phases:
/// 1. Render static entities
/// 2. Render non-player entities
/// 3. Render player
fn render(world: &recs::Ecs, con: &mut RootConsole){
    let map_id = get_map(&world);
    let map = world.get::<MapObject>(map_id).unwrap();
    for x in 0..MAP_WIDTH as usize{
        for y in 0..MAP_HEIGHT as usize{
            con.put_char(x as i32, y as i32, map.map.walls.get(&(x, y)), BackgroundFlag::Set);
        }
    }

    let player = get_player(&world);
    let fov = world.get::<Fov>(player).unwrap().fov;
    let memory = world.get::<SpatialMemory>(player).unwrap().memory;

    let components = component_filter!(Static, Position, Sprite);
    let mut statics = vec![];
    world.collect_with(&components, &mut statics);

    for id in statics.iter(){
        let pos: Position = world.get(*id).unwrap();
        let sprite: Sprite = world.get(*id).unwrap();

        if is_in_fov(&fov, pos.x, pos.y, ){
            con.put_char(pos.x, pos.y, sprite.glyph, BackgroundFlag::Set);
        } else if is_in_fov(&memory, pos.x, pos.y) {
            con.put_char_ex(pos.x, pos.y, sprite.glyph, DARK_GREY, BLACK);

        }
    }

    let components2 = component_filter!(Position, Sprite);
    let mut to_update = Vec::new();
    world.collect_with(&components2, &mut to_update);

    for id in to_update.iter(){
        if world.has::<Player>(*id).unwrap() {continue}
        if world.has::<Static>(*id).unwrap() {continue}
        let pos: Position = world.get(*id).unwrap();
        let sprite: Sprite = world.get(*id).unwrap();

        if is_in_fov(&fov, pos.x, pos.y)  {
            con.put_char(pos.x, pos.y, sprite.glyph, BackgroundFlag::Set);
        }
        //con.put_char(pos.x, pos.y, sprite.glyph, BackgroundFlag::Set);
    }

    let components3 = component_filter!(Player);
    to_update = vec![];
    world.collect_with(&components3, &mut to_update);

    for id in to_update.iter(){
        let pos: Position = world.get(*id).unwrap();
        let sprite: Sprite = world.get(*id).unwrap();

        con.put_char(pos.x, pos.y, sprite.glyph, BackgroundFlag::Set);
    }

}

/// Calculate if point is visible in fov, works also with memory or any other NodeMap<bool> that
/// has the map dimensions
fn is_in_fov(nm: &NodeMap<bool>, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= MAP_WIDTH || y >= MAP_HEIGHT {
        return false;
    }
    nm.get(&(x as usize, y as usize))
}

/// If the entity has taken damage, calculate how much to take and reduce current hp.
/// Kills entity if hp drops below zero.
fn take_dmg(world: &mut Ecs) {
    let components = component_filter!(Health, TakeDamage);
    let mut to_update = Vec::new();
    world.collect_with(&components, &mut to_update);


    for id in to_update.iter() {

        let new_health;
        {
            let health: i32 = world.get::<Health>(*id).unwrap().hp;
            let takedamage: i32 = world.get::<TakeDamage>(*id).unwrap().dmg.iter().sum();
            new_health = health - takedamage;
            let take_dmg = world.borrow_mut::<TakeDamage>(*id).unwrap();
            take_dmg.dmg.clear();
        }

        {
            let mut health = world.borrow_mut::<Health>(*id).unwrap();
            health.hp = new_health;
        }

        if new_health <= 0 {
            println!("{} has died.", world.get::<Name>(*id).unwrap().name.clone());
            die(world, id);
        }
    }
}

/// Kill entity and do the needful
fn die(world: &mut Ecs, id: &EntityId){
    let pos : Position = world.get(*id).unwrap();
    let mut name : Name = world.get(*id).unwrap();
    name.name = format!("Corpse of {}", name.name);
    let _ = world.destroy_entity(*id);
    let corpse = world.create_entity();
    let _ = world.set(corpse, pos);
    let _ = world.set(corpse, name);
    let _ = world.set(corpse, Sprite{ glyph: '%'});
}

/// Calculates the FOV for every entity who has the FOV and SightRange components
/// Also calulates the SpatialMemory if entity has it.
fn calculate_fov(world: &mut Ecs){
    // This are the Entities which FOV is calculated for
    let components = component_filter!(Position, Fov, SightRange);
    let mut to_update = Vec::new();
    world.collect_with(&components, &mut to_update);

    // These are the Entities which block sight
    let components2 = component_filter!(BlockSight, Position);
    let mut blocking = Vec::new();
    world.collect_with(&components2, &mut blocking);

    let mut fov_map = Map::new(MAP_WIDTH, MAP_HEIGHT);
    fov_map.clear(true, true);

    for id in blocking.iter(){
        let pos = world.get::<Position>(*id).unwrap();
        fov_map.set(pos.x, pos.y, false, false);
    }

    for id in to_update.iter(){
        let pos = world.get::<Position>(*id).unwrap();
        let range = world.get::<SightRange>(*id).unwrap().range;
        fov_map.compute_fov(pos.x, pos.y, range, FOV_LIGHT_WALLS, FOV_ALGO);
        {
            let fov = world.borrow_mut::<Fov>(*id).unwrap();
            fov.fov = map_to_vec(&fov_map);
        }
        // Also update memory if entity has one (mainly the player)
        if world.has::<SpatialMemory>(*id).unwrap() {
            let fov = world.get::<Fov>(*id).unwrap();
            let mut memory = world.borrow_mut::<SpatialMemory>(*id).unwrap();
            compute_memory(&mut memory.memory, &fov.fov);
        }
    }
}

/// Converts tcod's Map into NodeMap.
/// Tcod Map is used because it handles FOV computing.
fn map_to_vec(map: &Map) -> NodeMap<bool>{
    let mut vec :Vec<bool> = Vec::with_capacity((MAP_WIDTH * MAP_HEIGHT) as usize);

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            vec.push(map.is_in_fov(x, y));
        }
    }

    NodeMap::from_vec(MAP_WIDTH as usize, MAP_HEIGHT as usize, vec)
}

/// Updates memory
fn compute_memory(memory: &mut NodeMap<bool>, fov: &NodeMap<bool>){
    for x in 0..MAP_WIDTH as usize{
        for y in 0..MAP_HEIGHT as usize{
            if fov.get(&(x, y)) == true {
                memory.set(&(x, y), true);
            }
        }
    }
}

fn move_or_attack(world: &mut Ecs){

    // Get entities which can move
    let components = component_filter!(Position, Velocity);
    let mut to_update = Vec::new();
    world.collect_with(&components, &mut to_update);

    // Get entities with position that the moving
    // entity can interact with
    let mut to_go = Vec::new();
    let positions = component_filter!(Position);
    world.collect_with(&positions, &mut to_go);


    for id in to_update.iter(){

        let new_x;
        let new_y;

        let vel: Velocity = world.get(*id).unwrap();
        let pos: Position = world.get(*id).unwrap();

        if pos.x + vel.x >= 0 && pos.x + vel.x < MAP_WIDTH &&
            pos.y + vel.y >= 0 && pos.y + vel.y < MAP_HEIGHT {
            new_x = vel.x + pos.x;
            new_y = vel.y + pos.y;
        }
            else {
                new_x = pos.x;
                new_y = pos.y;
            }

        // Default action if no-one is on the way
        let mut action = Action::Move;

        for id_other in to_go.iter(){
            let pos_other : Position = world.get(*id_other).unwrap();
            if pos_other.x == new_x && pos_other.y == new_y && *id != *id_other {
                if world.has::<TakeDamage>(*id_other).unwrap() {
                    action = Action::Attack;

                    println!("You hit {}", world.get::<Name>(*id_other).unwrap().name);
                    let damage = world.get::<Damage>(*id).unwrap().dmg;
                    world.borrow_mut::<TakeDamage>(*id_other).unwrap().dmg.push(damage);
                } else if world.has::<Blocking>(*id_other).unwrap(){
                    action = Action::BlockedMove;
                }
            }
        }

        if action == Action::Move {
            let _ = world.set(*id, Position { x: new_x, y: new_y });
        }

        let _ = world.set(*id, Velocity{x: 0, y: 0});


    }
}

fn get_player(world: &Ecs) -> EntityId{
    let components = component_filter!(Player);
    let mut to_update = vec![];
    world.collect_with(&components, &mut to_update);
    to_update[0]
}

fn get_map(world: &Ecs) -> EntityId{
    let components = component_filter!(MapObject);
    let mut to_update = vec![];
    world.collect_with(&components, &mut to_update);
    to_update[0]
}

fn save(world: &Ecs){

    let vec:Vec<Vec<serde_json::value::Value>> = serialize_world!(world; Position, Velocity,
     Sprite, Name, TakeDamage, Health, Blocking, Damage, Player, Static, Fov, SpatialMemory,
     BlockSight, SightRange, MapObject);

    let buffer = File::create("foo.txt").unwrap();
    println!("Game saved!");
    let _ = serde_json::to_writer(buffer, &json!(vec));;

}

    fn load(world: &mut Ecs){
        let mut ids = vec![];
        world.collect(&mut ids);

        for id in ids.iter(){
            let _ = world.destroy_entity(*id);
        }

        let buffer = File::open("foo.txt").unwrap();
        let json: serde_json::Value = serde_json::from_reader(buffer).unwrap();

        load_components!(world; json; Position, Velocity,
            Sprite, Name, TakeDamage, Health, Blocking, Damage, Player, Static, Fov, SpatialMemory,
            BlockSight, SightRange, MapObject);

        println!("Game loaded!");
    }

fn main() {
    let mut con = RootConsole::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(MAP_WIDTH, MAP_HEIGHT)
        .title("ecs test")
        .init();

    let mut world = Ecs::new();

    let mut player = world.create_entity();
    let mut map = world.create_entity();

    let _ = world.set(player, Position::new(1, 1));
    let _ = world.set(player, Player{});
    let _ = world.set(player, Damage::new(1));
    let _ = world.set(player, Velocity::new(0,0));
    let _ = world.set(player, Sprite::new('@'));
    let _ = world.set(player, Fov::new(NodeMap::new(MAP_WIDTH as usize, MAP_HEIGHT as usize, false)));
    let _ = world.set(player, SpatialMemory::new(NodeMap::new (MAP_WIDTH as usize, MAP_HEIGHT as usize, false)));
    let _ = world.set(player, SightRange::new(5));

    let map_info = mapgen::generate_cave(MAP_WIDTH as usize,
                                    MAP_HEIGHT as usize,
                                    3,
                                    40);
    let (sx, sy) = map_info.start;
    let _ = world.set(player, Position::new(sx as i32, sy as i32));

    let _ = world.set(map, MapObject::new(map_info));

    calculate_fov(&mut world);

    while !con.window_closed(){
        con.clear();

        //let start = PreciseTime::now();
        render(&world, &mut con);
        //let end = PreciseTime::now();
        //println!("{} seconds for whatever you did.", start.to(end));

        con.flush();
        player = get_player(&world);
        let keypress = con.wait_for_keypress(true);

        if keypress.pressed {
            match keypress {
                Key { code: Escape, .. } => break,
                Key { code: Up, .. } => {
                    let v = world.get::<Velocity>(player).unwrap();
                    let _ = world.set(player, Velocity {x : v.x, y: v.y-1});
                },
                Key { code: Down, .. } => {
                    let v = world.get::<Velocity>(player).unwrap();
                    let _ = world.set(player, Velocity {x : v.x, y: v.y+1});
                },
                Key { code: Left, .. } => {
                    let v = world.get::<Velocity>(player).unwrap();
                    let _ = world.set(player, Velocity {x : v.x-1, y: v.y});
                },
                Key { code: Right, .. } => {
                    let v = world.get::<Velocity>(player).unwrap();
                    let _ = world.set(player, Velocity {x : v.x+1, y: v.y});
                },
                Key { code: F5, .. } => {
                    save(&world);
                },
                Key { code: F9, .. } => {
                    load(&mut world);
                },
                _ => {}
            }
        }

        move_or_attack(&mut world);
        take_dmg(&mut world);
        calculate_fov(&mut world);

    }

}