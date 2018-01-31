use rand;
use rand::Rng;
use std::collections::VecDeque;


fn flood_fill(start :(i32, i32), m: &Vec<Vec<i32>>, color: i32) -> Vec<Vec<i32>>{
    let (x,y) = start;

    assert_eq!(m[y as usize][x as usize], 0);

//    let height = m.len();
//    let width = m[0].len();

    let mut m2 = m.clone();

//    m2[y as usize][x as usize] = color;

    let mut queue = VecDeque::new();
    queue.push_back((x,y));

    while !queue.is_empty(){
        let (x1,y1) = queue.pop_front().unwrap();
        if m2[y1 as usize][x1 as usize] != 0{
            continue
        }
        m2[y1 as usize][x1 as usize] = color;

        if m2[(y1-1i32) as usize][x1 as usize] == 0 { queue.push_back((x1, y1-1)) }
        if m2[(y1+1i32) as usize][x1 as usize] == 0 { queue.push_back((x1, y1+1)) }
        if m2[y1 as usize][(x1-1i32) as usize] == 0 { queue.push_back((x1-1, y1)) }
        if m2[y1 as usize][(x1+1i32) as usize] == 0 { queue.push_back((x1+1, y1)) }
    }
    m2
}

fn fill_map(m: &Vec<Vec<i32>>) -> Vec<Vec<i32>>{
    let height = m.len();
    let width = m[0].len();
    let mut m2 = m.clone();

    let mut colors = 11..99;

    for x in 0..width {
        for y in 0..height {
            if m2[y][x] == 0 {
                m2 = flood_fill((x as i32, y as i32), &m2, colors.next().unwrap());
            }
        }
    }
    m2
}

fn create_vector(size: i32) -> Vec<i32> {
    let mut v = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        v.push(rng.gen_range(1, 100))
    }

    v
}

fn create_matrix(width: i32, height: i32) -> Vec<Vec<i32>> {
    let mut v: Vec<Vec<i32>> = Vec::new();

    for _ in 0..height {
        v.push(create_vector(width))
    }
    v
}

fn fill_edges(m: &mut Vec<Vec<i32>>) {
    let height = m.len();
    let width = m[0].len();
    for y in 0..height {
        for x in 0..width {
            if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                m[y][x] = 1;
            }
        }
    }
}

fn matrix_to_bin(m: &Vec<Vec<i32>>, chance: i32) -> Vec<Vec<i32>> {
    let mut m2: Vec<Vec<i32>> = Vec::new();

    for vec in m {
        m2.push(vector_to_bin(vec, chance))
    }

    m2
}

fn vector_to_bin(v: &Vec<i32>, chance: i32) -> Vec<i32> {
    let mut v2: Vec<i32> = Vec::new();

    for val in v {
        match val {
            _ if *val < chance => v2.push(1),
            _ => v2.push(0),
        }
    }
    v2
}

#[allow(dead_code)]
fn matrix_print(v: Vec<Vec<i32>>) {
    for vec in &v {
        println!("{:?}", vec)
    }
}

fn automaton(original: &Vec<Vec<i32>>) -> Vec<Vec<i32>> {
    let mut m2: Vec<Vec<i32>> = original.clone();

    let height = (*original).len();
    let width = (*original)[0].len();

    let death_limit = 3;
    let birth_limit = 4;

    for y in 0..height {
        for x in 0..width {
            let alive = count_alive_neighbours(original, x as i32, y as i32);

            if (*original)[y][x] == 1 {
                if alive < death_limit {
                    m2[y][x] = 0;
                } else {
                    m2[y][x] = 1;
                }
            } else {
                if alive > birth_limit {
                    m2[y][x] = 1;
                } else {
                    m2[y][x] = 0;
                }
            }
        }
    }

    m2
}

fn print_m_as_map(m: &Vec<Vec<i32>>) {
    println!();
    for v in m {
        for val in v {
            match *val{
                1 => print!("X"),
                0 => print!(" "),
                _ =>  print!("{}", val % 10)
            }
        }
        println!()
    }
}

fn count_alive_neighbours(m: &Vec<Vec<i32>>, x: i32, y: i32) -> i32 {
    let height = (*m).len() as i32;
    let width = (*m)[0].len() as i32;

    let mut alive = 0;

    for i in (x - 1)..(x + 2) {
        for j in (y - 1)..(y + 2) {
            if i < 0 || j < 0 || i >= width || j >= height {
                alive += 1
            } else if i == x && j == y {
                continue;
            } else {
                alive += m[j as usize][i as usize]
            }
        }
    }
    alive
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::time::{Instant};
    use pathfinding::astar;

    #[test]
    #[ignore]
    fn test_auto() {
            let m = create_matrix(60, 35);
            let mut m2 = matrix_to_bin(&m, 40);
            fill_edges(&mut m2);

            print_m_as_map(&m2);

            let m3 = automaton(&m2);

            print_m_as_map(&m3);

            let m4 = automaton(&m3);

            print_m_as_map(&m4);

            let mut m5 = automaton(&m4);
            m5 = automaton(&m5);
            m5 = automaton(&m5);
            m5 = automaton(&m5);

            print_m_as_map(&m5);
    }

    #[test]
    #[ignore]
    fn test_count_alive() {
        let m = vec![vec![1, 0, 1], vec![1, 0, 1], vec![1, 0, 1]];

        assert_eq!(count_alive_neighbours(&m, 0, 0), 6);
    }


    #[test]
    #[ignore]
    fn test_flood_fill() {
        let mut m = vec![vec![0;100]; 100];

//        assert_eq!(count_alive_neighbours(&m, 0, 0), 6);

        fill_edges(&mut m);

        let instant = Instant::now();
        let m2 = flood_fill((5,5), &m, 11);
        let elapsed = instant.elapsed();

        print_m_as_map(&m2);
        println!("{:?}", elapsed);
    }

    #[test]
    #[ignore]
    fn test_fill_dungeon(){
        let m = create_matrix(60, 35);
        let mut m2 = matrix_to_bin(&m, 40);
        fill_edges(&mut m2);

        let mut m3 = automaton(&m2);
        m3 = automaton(&m3);
        m3 = automaton(&m3);

        m3 = fill_map(&m3);

        print_m_as_map(&m3);
    }

    #[derive(Clone, Debug)]
    pub struct Map {
        map: Vec<char>,
        height: usize,
        width: usize,
    }

    impl Map {
        fn get(&self, x: i32, y: i32) -> Option<char>{
            if x >= self.width as i32 || x < 0 || y >= self.height as i32 || y < 0 {
                return None;
            }
            Some(self.map[(y as usize* self.height) + x as usize])
        }

        fn set(&mut self, x: i32, y: i32, c: char) -> Result<(),()>{
            if x >= self.width as i32 || x < 0 || y >= self.height as i32 || y < 0 {
                return Err(());
            }
            self.map[(y as usize* self.height) + x as usize] = c;
            Ok(())
        }

        fn new(c: char, width: usize, height: usize) -> Map{
            Map {map: vec![c; width * height], height, width }
        }

        fn in_map(&self, x: i32, y : i32)-> bool{
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
            vec![Pos(x+1,y), Pos(x,y-1), Pos(x-1, y), Pos(x, y+1)]
                .into_iter().filter(|p| map.in_map(p.0, p.1))
                .filter(|p| map.get(p.0, p.1).unwrap() != '0')
                .map(|p| (p, 1)).collect()
        }
    }

    #[test]
    #[ignore]
    fn test_pos(){
        let p = Pos(0,0);
        let mut map = Map::new('1', 10, 10);

        for i in 0..6 {
            map.set(i, 1, '0').unwrap();
        }

        for i in 0..100 {
            print!("{}", map.get(i % 10, i / 10).unwrap());
            if i%10 == 9 {
                println!();
            }
        }

        println!("Pos: {:?}", p.neighbours(&map));
    }

    #[test]
    fn test_astar(){
        let mut map = Map::new('1', 10, 10);
        for i in 0..6 {
            map.set(i, 3, '0').unwrap();
        }




        static GOAL: Pos = Pos(1, 7);
        let result = astar(&Pos(1, 1), |p| p.neighbours(&map), |p| p.distance(&GOAL),
                           |p| *p == GOAL);
        println!("{:?}", result);

        for r in result.unwrap().0 {
            map.set(r.0, r.1, '2');
        }

        for i in 0..100 {

            print!("{}", map.get(i % 10, i / 10).unwrap());
            if i%10 == 9 {
                println!();
            }
        }

//        assert_eq!(result.expect("no path found").1,16);
    }
}