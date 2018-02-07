use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

use grid::NodeMap;

use rand;
use rand::Rng;

use tcod::colors;

use pathfinding::bfs;
use pathfinding::Grid;

pub struct MapInfo {
    pub walls: NodeMap<char>,
    pub colors: NodeMap<colors::Color>,
    pub blocked: NodeMap<bool>,
    pub visible: NodeMap<bool>,
    pub start: (usize, usize),
    pub end: (usize, usize),
}

fn flood_fill(start: (i32, i32), m: &NodeMap<i32>, color: i32) -> NodeMap<i32> {
    let (x, y) = start;

    assert_eq!(m.get(&(x as usize, y as usize)), 0);

    let mut m2 = m.clone();

    let mut queue = VecDeque::new();
    queue.push_back((x, y));

    while !queue.is_empty() {
        let (x1, y1) = queue.pop_front().unwrap();
        if m2.get(&(x1 as usize, y1 as usize)) != 0 {
            continue;
        }
        m2.set(&(x1 as usize, y1 as usize), color);

        if m2.get(&(x1 as usize, (y1 - 1) as usize)) == 0 {
            queue.push_back((x1, y1 - 1))
        }
        if m2.get(&(x1 as usize, (y1 + 1) as usize)) == 0 {
            queue.push_back((x1, y1 + 1))
        }
        if m2.get(&((x1 - 1) as usize, y1 as usize)) == 0 {
            queue.push_back((x1 - 1, y1))
        }
        if m2.get(&((x1 + 1) as usize, y1 as usize)) == 0 {
            queue.push_back((x1 + 1, y1))
        }
    }
    m2
}

fn fill_map(nm: &NodeMap<i32>) -> (NodeMap<i32>, usize) {
    let height = nm.height;
    let width = nm.width;
    let mut m2 = nm.clone();

    let mut colors = 2..999;
    let mut count = 0;

    for x in 0..width {
        for y in 0..height {
            if m2.get(&(x, y)) == 0 {
                m2 = flood_fill((x as i32, y as i32), &m2, colors.next().unwrap());
                count += 1;
            }
        }
    }
    (m2, count)
}

fn room_sizes<T: Ord + Hash + Eq + Clone>(m: &NodeMap<T>, exclude: &[T]) -> Vec<(T, usize)> {
    let mut size = HashMap::new();

    let height = m.height;
    let width = m.width;

    for x in 0..width {
        for y in 0..height {
            let tile = m.get(&(x, y)).clone();
            if !exclude.contains(&tile) && !size.contains_key(&tile) {
                size.insert(tile, 1);
            } else if !exclude.contains(&tile) {
                let c = &size[&tile] + 1;
                size.insert(tile, c);
            }
        }
    }
    let mut ret = size.into_iter().collect::<Vec<(T, usize)>>();
    ret.sort_by(|a, b| a.1.cmp(&b.1));
    ret
}

fn new_binary_nodemap(width: usize, height: usize, probability: usize) -> NodeMap<i32> {
    let mut v: Vec<i32> = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..(width * height) {
        let roll = rng.gen_range(0, 100);
        if roll < probability {
            v.push(1);
        } else {
            v.push(0);
        }
    }

    NodeMap::from_vec(width, height, v)
}

fn fill_edges_with<C: Clone>(nm: &mut NodeMap<C>, c: C) {
    for y in 0..nm.height {
        for x in 0..nm.width {
            if x == 0 || x == nm.width - 1 || y == 0 || y == nm.height - 1 {
                nm.set(&(x, y), c.clone());
            }
        }
    }
}

fn automaton(original: &NodeMap<i32>) -> NodeMap<i32> {
    let mut nm: NodeMap<i32> = original.clone();

    let death_limit = 3;
    let birth_limit = 4;

    for y in 0..nm.height {
        for x in 0..nm.width {
            let alive = count_alive_neighbours(original, &(x as i32, y as i32));

            if original.get(&(x, y)) == 1 {
                if alive < death_limit {
                    nm.set(&(x, y), 0);
                } else {
                    nm.set(&(x, y), 1);
                }
            } else if alive > birth_limit {
                nm.set(&(x, y), 1);
            } else {
                nm.set(&(x, y), 0);
            }
        }
    }

    nm
}

fn count_alive_neighbours(nm: &NodeMap<i32>, p: &(i32, i32)) -> i32 {
    let height = nm.height as i32;
    let width = nm.width as i32;

    let mut alive = 0;

    for i in (p.0 - 1)..(p.0 + 2) {
        for j in (p.1 - 1)..(p.1 + 2) {
            if i < 0 || j < 0 || i >= width || j >= height {
                alive += 1
            } else if i == p.0 && j == p.1 {
                continue;
            } else {
                alive += nm.get(&(i as usize, j as usize))
            }
        }
    }
    alive
}

fn search_closest_value<C: Eq + Hash + Clone>(
    haystack: &NodeMap<C>,
    start: &(usize, usize),
    needles: &[C],
) -> Option<Vec<(usize, usize)>> {
    let mut gridmap = Grid::new(haystack.width, haystack.height);
    gridmap.fill();

    bfs(
        start,
        |x| gridmap.neighbours(x),
        |x| needles.contains(&haystack.get(x)),
    )
}

fn connect_rooms(nm: &NodeMap<i32>, number_rooms: usize) -> NodeMap<i32> {
    let mut rooms: Vec<i32> = (2..).take(number_rooms).collect();

    let mut nm_connected = nm.clone();

    'outer: while rooms.len() > 1 {
        let points = randomize_points(nm.width, nm.height);
        for point in points {
            let room = nm_connected.get(&point);
            if room != 1 && rooms.contains(&room) {
                let goal = rooms
                    .clone()
                    .into_iter()
                    .filter(|x| *x != room)
                    .collect::<Vec<i32>>();

                let mut closest = search_closest_value(&nm_connected, &point, &goal);

                let target = closest.unwrap().pop().unwrap();
                let old_room = nm_connected.get(&target);

                let mut edge = search_closest_value(&nm_connected, &target, &[room]);

                for p in edge.unwrap() {
                    nm_connected.set(&p, room);
                }

                change_nodes(&mut nm_connected, old_room, room);

                let index = rooms.iter().position(|&r| r == old_room).unwrap();

                rooms.remove(index);
                continue 'outer;
            }
        }
    }
    nm_connected
}

fn change_nodes(nm: &mut NodeMap<i32>, from: i32, to: i32) {
    for x in 0..nm.width {
        for y in 0..nm.height {
            if nm.get(&(x, y)) == from {
                nm.set(&(x, y), to);
            }
        }
    }
}

pub fn generate_cave(
    width: usize,
    height: usize,
    generations: usize,
    fill_percentage: usize,
) -> MapInfo {
    let mut nm = new_binary_nodemap(width, height, fill_percentage);

    fill_edges_with(&mut nm, 1);

    //    let start: (usize, usize);
    //    let end: (usize, usize);

    for _ in 0..generations {
        nm = automaton(&nm)
    }

    let (start, end) = find_start_and_exit(&nm);

    let _rooms: usize;
    let res = fill_map(&nm);
    nm = res.0;
    _rooms = res.1;

    let mut mat: NodeMap<char> = NodeMap::new(width, height, '.');
    let mut colormat: NodeMap<colors::Color> = NodeMap::new(width, height, colors::WHITE);

    let colvec = vec![
        colors::LIGHT_BLUE,
        colors::RED,
        colors::GREEN,
        colors::CYAN,
        colors::FUCHSIA,
        colors::AMBER,
        colors::HAN,
        colors::PURPLE,
    ];

    for y in 0..height {
        for x in 0..width {
            let color = match nm.get(&(x, y)) {
                1 => colors::WHITE,
                z => colvec[((z + 5) % 8) as usize],
            };
            colormat.set(&(x, y), color)
        }
    }

    nm = connect_rooms(&nm, _rooms);

    for y in 0..height {
        for x in 0..width {
            let c = match nm.get(&(x, y)) {
                1 => '#',
                _ => '.',
            };
            mat.set(&(x, y), c);
        }
    }

    let mp: MapInfo = MapInfo {
        walls: mat,
        colors: colormat,
        blocked: NodeMap::new(width, height, false),
        visible: NodeMap::new(width, height, true),
        start,
        end,
    };
    mp
}

fn randomize_points(x: usize, y: usize) -> Vec<(usize, usize)> {
    let mut points: Vec<(usize, usize)> = vec![];

    for _x in 0..x {
        for _y in 0..y {
            points.push((_x, _y));
        }
    }

    let mut rng = rand::thread_rng();
    let slice = points.as_mut_slice();
    rng.shuffle(slice);

    slice.to_vec()
}

fn find_start_and_exit(nm: &NodeMap<i32>) -> ((usize, usize), (usize, usize)) {
    let mut start_points = randomize_points(nm.width, nm.height);

    start_points = start_points
        .into_iter()
        .filter(|z| nm.get(z) != 1)
        .collect();

    let start = start_points.pop().unwrap();
    let end = start_points.pop().unwrap();

    (start, end)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Instant;
    use pathfinding::astar;

    //    #[test]
    //    fn test_cavegen(){
    //        let mat = generate_cave(70,42,3,40 );
    //    }
    //

    #[test]
    fn test_count_alive() {
        let m = NodeMap::from_vec(3, 3, vec![1, 0, 1, 1, 0, 1, 1, 0, 1]);

        assert_eq!(count_alive_neighbours(&m, &(1, 1)), 6);
    }

    //    #[test]
    //    fn test_flood_fill() {
    //        let mut m = vec![vec![0;100]; 100];
    //
    ////        assert_eq!(count_alive_neighbours(&m, 0, 0), 6);
    //
    //        fill_edges(&mut m);
    //
    //        let instant = Instant::now();
    //        let m2 = flood_fill((5,5), &m, 11);
    //        let elapsed = instant.elapsed();
    //
    //        print_m_as_map(&m2);
    //        println!("{:?}", elapsed);
    //    }
    //
    #[test]
    fn test_fill_dungeon() {
        let mut m2 = new_binary_nodemap(60, 35, 40);
        fill_edges_with(&mut m2, 1);

        m2.print();
        let mut m3 = automaton(&m2);
        m3 = automaton(&m3);
        m3 = automaton(&m3);

        m3.print();

        let rooms: usize;
        let res = fill_map(&m3);
        m3 = res.0;
        rooms = res.1;

        println!();
        m3.print();
    }

    #[derive(Clone, Debug)]
    pub struct Map {
        map: Vec<char>,
        height: usize,
        width: usize,
    }

    impl Map {
        fn get(&self, x: i32, y: i32) -> Option<char> {
            if x >= self.width as i32 || x < 0 || y >= self.height as i32 || y < 0 {
                return None;
            }
            Some(self.map[(y as usize * self.height) + x as usize])
        }

        fn set(&mut self, x: i32, y: i32, c: char) -> Result<(), ()> {
            if x >= self.width as i32 || x < 0 || y >= self.height as i32 || y < 0 {
                return Err(());
            }
            self.map[(y as usize * self.height) + x as usize] = c;
            Ok(())
        }

        fn new(c: char, width: usize, height: usize) -> Map {
            Map {
                map: vec![c; width * height],
                height,
                width,
            }
        }

        fn in_map(&self, x: i32, y: i32) -> bool {
            if x >= self.width as i32 || x < 0 || y >= self.height as i32 || y < 0 {
                return false;
            }
            true
        }
    }

    #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
    struct Pos(i32, i32);

    impl Pos {
        fn distance(&self, other: &Pos) -> usize {
            ((self.0 - other.0).abs() + (self.1 - other.1).abs()) as usize
        }

        fn neighbours(&self, map: &Map) -> Vec<(Pos, usize)> {
            let &Pos(x, y) = self;
            vec![Pos(x + 1, y), Pos(x, y - 1), Pos(x - 1, y), Pos(x, y + 1)]
                .into_iter()
                .filter(|p| map.in_map(p.0, p.1))
                .filter(|p| map.get(p.0, p.1).unwrap() != '0')
                .map(|p| (p, 1))
                .collect()
        }
    }

    #[test]
    fn test_pos() {
        let p = Pos(0, 0);
        let mut map = Map::new('1', 10, 10);

        for i in 0..6 {
            map.set(i, 1, '0').unwrap();
        }

        for i in 0..100 {
            print!("{}", map.get(i % 10, i / 10).unwrap());
            if i % 10 == 9 {
                println!();
            }
        }

        println!("Pos: {:?}", p.neighbours(&map));
    }

    #[test]
    fn test_astar() {
        let mut map = Map::new('1', 10, 10);
        for i in 0..6 {
            map.set(i, 3, '0').unwrap();
        }

        static GOAL: Pos = Pos(1, 7);
        let result = astar(
            &Pos(1, 1),
            |p| p.neighbours(&map),
            |p| p.distance(&GOAL),
            |p| *p == GOAL,
        );
        println!("{:?}", result);

        for r in result.unwrap().0 {
            map.set(r.0, r.1, '2').unwrap();
        }

        for i in 0..100 {
            print!("{}", map.get(i % 10, i / 10).unwrap());
            if i % 10 == 9 {
                println!();
            }
        }

        //        assert_eq!(result.expect("no path found").1,16);
    }
    #[test]
    fn test_binary_nodemap() {
        let nm = new_binary_nodemap(20, 20, 40);
        nm.print();
    }

    #[test]
    fn test_fill_edges_with() {
        let mut nm = new_binary_nodemap(5, 7, 0);
        fill_edges_with(&mut nm, 1);
        nm.print();
    }

    #[test]
    fn test_search_closest() {
        let mut nm = NodeMap::new(10, 10, '0');
        fill_edges_with(&mut nm, '1');

        nm.set(&(1, 1), '2');
        nm.set(&(5, 9), '3');

        let res = search_closest_value(&nm, &(1, 1), &['3']).unwrap();
        println!("{:?}", res);
    }

    #[test]
    fn test_connect_rooms() {
        let mut nm = NodeMap::new(10, 10, 1);
        let room1 = &[(1, 1), (1, 2), (2, 2), (2, 1)];
        let room2 = &[(8, 8), (9, 9), (8, 9), (9, 8)];
        let room3 = &[(3, 5), (3, 6), (4, 5), (4, 6)];

        for point in room1 {
            nm.set(&point, 2);
        }

        for point in room2 {
            nm.set(&point, 3);
        }

        for point in room3 {
            nm.set(&point, 4);
        }

        let connected = connect_rooms(&nm, 3);
        //        println!("{:?}", c)
        connected.print();
    }
}
