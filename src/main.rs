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
    map_info: &mut MapInfo,
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

        Key { code: Spacebar, .. } => *map_info = new_map(),

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
    root.put_char(mapp.start.0 as i32, mapp.start.1 as i32, '<', BackgroundFlag::None);
    root.put_char(mapp.end.0 as i32, mapp.start.1 as i32, '>', BackgroundFlag::None);
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

    let mut map_info = mapgen::generate_cave(SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize, 3, 40);

    while !root.window_closed() {
        root.set_default_foreground(colors::WHITE);

        draw(&mut root, &map_info);
        root.flush();

        // handle keys and exit game if needed
        let exit = handle_keys(&mut root, &mut map_info);
        if exit {
            break;
        }
    }
}
