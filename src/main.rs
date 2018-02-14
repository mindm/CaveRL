extern crate tcod;
extern crate serde;

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
    fov: Vec<bool>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, new)]
struct SpatialMemory {
    memory: Vec<bool>
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
struct BlockSight{}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize, new)]
struct SightRange{
    range: i32
}

fn render(world: &recs::Ecs, con: &mut RootConsole){
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

fn is_in_fov(vec: &Vec<bool>, x: i32, y: i32) -> bool {
    if x < 0 || y < 0 || x >= MAP_WIDTH || y >= MAP_HEIGHT {
        return false;
    }
    vec[(MAP_WIDTH * y + x) as usize]
}

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

fn calculate_fov(world: &mut Ecs){
    let components = component_filter!(Position, Fov, SightRange);
    let mut to_update = Vec::new();
    world.collect_with(&components, &mut to_update);

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
        if world.has::<SpatialMemory>(*id).unwrap() {
            let fov = world.get::<Fov>(*id).unwrap();
            let mut memory = world.borrow_mut::<SpatialMemory>(*id).unwrap();
            compute_memory(&mut memory.memory, &fov.fov);
        }
    }
}

fn map_to_vec(map: &Map) -> Vec<bool>{
    let mut vec :Vec<bool> = Vec::with_capacity((MAP_WIDTH * MAP_HEIGHT) as usize);

    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH {
            vec.push(map.is_in_fov(x, y));
        }
    }
    vec
}

fn compute_memory(memory: &mut Vec<bool>, fov: &Vec<bool>){
    for y in 0..MAP_HEIGHT {
        for x in 0..MAP_WIDTH{
            memory[(y * MAP_WIDTH + x) as usize] =  memory[(y * MAP_WIDTH + x) as usize] || fov[(y * MAP_WIDTH + x) as usize];
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

fn save(world: &Ecs){

    let mut vec:Vec<Vec<serde_json::value::Value>> = vec![];

    for id in world.iter(){
        let mut vec_inner : Vec<serde_json::Value> = vec![];

        match world.get::<Position>(id) {
            Ok(component) => vec_inner.push(json!({"Position": component})),
            _ => ()
        }
        match world.get::<Velocity>(id) {
            Ok(component) => vec_inner.push(json!({"Velocity": component})),
            _ => ()
        }
        match world.get::<Name>(id) {
            Ok(component) => vec_inner.push(json!({"Name": component})),
            _ => ()
        }
        match world.get::<TakeDamage>(id) {
            Ok(component) => vec_inner.push(json!({"TakeDamage": component})),
            _ => ()
        }
        match world.get::<Health>(id) {
            Ok(component) => vec_inner.push(json!({"Health": component})),
            _ => ()
        }
        match world.get::<Blocking>(id) {
            Ok(component) => vec_inner.push(json!({"Blocking": component})),
            _ => ()
        }
        match world.get::<Sprite>(id) {
            Ok(component) => vec_inner.push(json!({"Sprite": component})),
            _ => ()
        }
        match world.get::<Damage>(id) {
            Ok(component) => vec_inner.push(json!({"Damage": component})),
            _ => ()
        }
        match world.get::<Player>(id) {
            Ok(component) => vec_inner.push(json!({"Player": component})),
            _ => ()
        }
        match world.get::<Static>(id) {
            Ok(component) => vec_inner.push(json!({"Static": component})),
            _ => ()
        }
        match world.get::<Fov>(id) {
            Ok(component) => vec_inner.push(json!({"Fov": component})),
            _ => ()
        }
        match world.get::<SpatialMemory>(id) {
            Ok(component) => vec_inner.push(json!({"SpatialMemory": component})),
            _ => ()
        }
        match world.get::<BlockSight>(id) {
            Ok(component) => vec_inner.push(json!({"BlockSight": component})),
            _ => ()
        }
        match world.get::<SightRange>(id) {
            Ok(component) => vec_inner.push(json!({"SightRange": component})),
            _ => ()
        }

        vec.push(vec_inner);
    }
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
    let v = json.as_array().unwrap();
    for i in 0..v.len(){
        let new = world.create_entity();
        for val in v[i].as_array().unwrap().iter(){
            let key : &String = val.as_object().unwrap().keys().collect::<Vec<_>>()[0];

            match key.as_ref(){
                "Position" => {
                    let _ = world.set::<Position>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                },
                "Velocity" => {
                    let _ = world.set::<Velocity>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                }
                "Sprite" => {
                    let _ = world.set::<Sprite>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                }
                "Name" => {
                    let _ = world.set::<Name>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                },
                "TakeDamage" => {
                    let _ = world.set::<TakeDamage>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                },
                "Health" => {
                    let _ = world.set::<Health>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                },
                "Blocking" => {
                    let _ = world.set::<Blocking>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                },
                "Damage" => {
                    let _ = world.set::<Damage>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                },
                "Player" => {
                    let _ = world.set::<Player>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                },
                "Static" => {
                    let _ = world.set::<Static>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                },
                "Fov" => {
                    let _ = world.set::<Fov>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                },
                "SpatialMemory" => {
                    let _ = world.set::<SpatialMemory>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                },
                "BlockSight" => {
                    let _ = world.set::<BlockSight>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                },
                "SightRange" => {
                    let _ = world.set::<SightRange>(new, serde_json::from_value(val.as_object().unwrap()[key].clone()).unwrap());
                }
                _ => ()
            }
        }

    }
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
    let walls = vec![(1, 3), (2, 3), (3, 3), (4, 3), (5, 3)];

    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT{
            if !walls.contains(&(x, y)) {
                let floor = world.create_entity();
                let _ = world.set(floor, Position::new(x, y));
                let _ = world.set(floor, Static {});
                let _ = world.set(floor, Sprite { glyph: '.' });
            }
        }
    }

    let mut player = world.create_entity();
    let monster = world.create_entity();

    let _ = world.set(player, Position::new(1, 1));
    let _ = world.set(player, Player{});
    let _ = world.set(player, Damage::new(1));
    let _ = world.set(player, Velocity::new(0,0));
    let _ = world.set(player, Sprite::new('@'));
    let _ = world.set(player, Fov::new(vec![false; (MAP_HEIGHT * MAP_WIDTH) as usize]));
    let _ = world.set(player, SpatialMemory::new(vec![false; (MAP_HEIGHT * MAP_WIDTH) as usize]));
    let _ = world.set(player, SightRange::new(5));

    let _ = world.set(monster, Velocity::new(0,0));
    let _ = world.set(monster, Position::new(10, 10));
    let _ = world.set(monster, Sprite::new('m'));
    let _ = world.set(monster, Health::new(2, 5));
    let _ = world.set(monster, TakeDamage::new());
    let _ = world.set(monster, Name::new("Gorok".to_string()));

    // let walls = vec![(1, 3), (2, 3), (3, 3), (4, 3), (5, 3)];
    for point in walls {
        let rock = world.create_entity();
        let (x, y) = point;
        let _ = world.set(rock, Position { x: x, y: y });
        let _ = world.set(rock, Blocking {});
        let _ = world.set(rock, Name::new("rock".to_string()));
        let _ = world.set(rock, Sprite { glyph: '#' });
        let _ = world.set(rock, BlockSight{} );
        let _ = world.set(rock, Static {});
    }

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