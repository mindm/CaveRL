extern crate pathfinding;
extern crate rand;
extern crate tcod;

mod mapgen;
mod grid;

use tcod::console::*;
use tcod::colors;
use mapgen::*;

// actual size of the window
const SCREEN_WIDTH: i32 = 80;
const SCREEN_HEIGHT: i32 = 50;

const LIMIT_FPS: i32 = 20;

fn handle_keys(
    root: &mut Root,
    player_x: &mut i32,
    player_y: &mut i32,
    mapp: &mut MapInfo,
) -> bool {
    use tcod::input::Key;
    use tcod::input::KeyCode::*;

    let key = root.wait_for_keypress(true);
    match key {
        Key {
            code: Enter,
            alt: true,
            ..
        } => {
            // Alt+Enter: toggle fullscreen
            let fullscreen = root.is_fullscreen();
            root.set_fullscreen(!fullscreen);
        }
        Key { code: Escape, .. } => return true, // exit game

        // movement keys
        Key { code: Up, .. } => *player_y -= 1,
        Key { code: Down, .. } => *player_y += 1,
        Key { code: Left, .. } => *player_x -= 1,
        Key { code: Right, .. } => *player_x += 1,

        Key { code: Spacebar, .. } => *mapp = new_map(),

        _ => {}
    }

    false
}

fn draw(root: &mut Root, mapp: &MapInfo) {
    for x in 0..SCREEN_WIDTH {
        for y in 0..SCREEN_HEIGHT {
            let c = mapp.walls.get(&(x as usize, y as usize));
            let color = mapp.colors.get(&(x as usize, y as usize));
            root.put_char(x, y, c, BackgroundFlag::None);
            root.set_char_foreground(x, y, color);
        }
    }
}

fn new_map() -> MapInfo {
    mapgen::generate_cave(SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize, 3, 40)
}

fn main() {
    let mut root = Root::initializer()
        .font("arial10x10.png", FontLayout::Tcod)
        .font_type(FontType::Greyscale)
        .size(SCREEN_WIDTH, SCREEN_HEIGHT)
        .title("rogue-rs")
        .init();

    tcod::system::set_fps(LIMIT_FPS);

    let mut player_x = SCREEN_WIDTH / 2;
    let mut player_y = SCREEN_HEIGHT / 2;

    let mut mapp = mapgen::generate_cave(SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize, 3, 40);

    while !root.window_closed() {
        root.set_default_foreground(colors::WHITE);
        //        root.put_char(player_x, player_y, '@', BackgroundFlag::None);

        draw(&mut root, &mapp);
        root.flush();

        //        root.put_char(player_x, player_y, ' ', BackgroundFlag::None);

        // handle keys and exit game if needed
        let exit = handle_keys(&mut root, &mut player_x, &mut player_y, &mut mapp);
        if exit {
            break;
        }
    }
}
