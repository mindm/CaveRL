use rand;
use rand::Rng;
use std::collections::VecDeque;
use pathfinding::Matrix;
use tcod::colors;
use std::collections::HashMap;
use std::hash::Hash;
use std::fmt::Debug;

pub struct Mapplus {
    pub mat: Matrix<char>,
    pub col : Matrix<colors::Color>
}

fn flood_fill(start :(usize, usize), m: &Matrix<i32>, color: i32) -> Matrix<i32>{
    let (x,y) = start;

    assert_eq!(m[&(x, y)], 0);

    let mut m2 = m.clone();

    let mut queue = VecDeque::new();
    queue.push_back((x,y));

    while !queue.is_empty(){
        let (x1,y1) = queue.pop_front().unwrap();
        if m2[&(x1, y1)] != 0{
            continue
        }
        m2[&(x1, y1)] = color;

        if m2[&(x1, y1-1)] == 0 { queue.push_back((x1, y1-1)) }
        if m2[&(x1, y1+1)] == 0 { queue.push_back((x1, y1+1)) }
        if m2[&(x1-1, y1)] == 0 { queue.push_back((x1-1, y1)) }
        if m2[&(x1+1, y1)] == 0 { queue.push_back((x1+1, y1)) }
    }
    m2
}

fn fill_map(m: &Matrix<i32>) -> (Matrix<i32>, usize){
    let height = m.rows;
    let width = m.columns;
    let mut m2 = m.clone();

    let mut colors = 2..99;
    let mut count = 0;

    for x in 0..width {
        for y in 0..height {
            if m2[&(x, y)] == 0 {
                m2 = flood_fill((x, y), &m2, colors.next().unwrap());
                count += 1;
            }
        }
    }
    (m2, count)
}


fn room_sizes<T: Ord + Hash + Eq + Clone>(m: &Matrix<T>, exclude: &Vec<T>) -> Vec<(T, usize)>{
    let mut size = HashMap::new();

    let height = m.rows;
    let width = m.columns;

    for x in 0..width {
        for y in 0..height {
            let tile = m[&(x,y)].clone();
            if !exclude.contains(&tile) && !size.contains_key(&tile) {
                size.insert(tile, 1);
            } else if !exclude.contains(&tile) {
                let c = size.get(&tile).unwrap() + 1;
                size.insert(tile, c );
            }
        }
    }
    let mut ret = size.into_iter().collect::<Vec<(T, usize)>>();
    ret.sort_by(|a, b| a.1.cmp(&b.1));
    ret
}

fn create_random_binary_matrix(
    width: usize,
    height: usize,
    fill_percent: usize) -> Matrix<i32>{
    let mut vector: Vec<i32> = vec![];
    let mut rng = rand::thread_rng();
    for _ in 0..width {
        for _ in 0..height {
            let randint = rng.gen_range(0, 100);

            if randint < fill_percent {
                vector.push(1);
            } else {
                vector.push(0);
            }
        }
    }
    let matrix: Matrix<i32> = Matrix::from_vec(height, width, vector);
    matrix
}

fn create_vector(size: usize) -> Vec<i32> {
    let mut v = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..size {
        v.push(rng.gen_range(1, 100))
    }

    v
}

fn create_matrix(width: usize, height: usize) -> Vec<Vec<i32>> {
    let mut v: Vec<Vec<i32>> = Vec::new();

    for _ in 0..height {
        v.push(create_vector(width))
    }
    v
}

fn fill_edges(m: &mut Matrix<i32>) {
    let height = m.rows;
    let width = m.columns;
    println!("height: {}, width: {}", width, height);
    for y in 0..height {
        for x in 0..width {
            if x == 0 || x == width - 1 || y == 0 || y == height - 1 {
                m[&(x, y)] = 1;
            }
        }
    }
}

fn matrix_to_bin(m: &Vec<Vec<i32>>, chance: usize) -> Vec<Vec<i32>> {
    let mut m2: Vec<Vec<i32>> = Vec::new();

    for vec in m {
        m2.push(vector_to_bin(vec, chance))
    }

    m2
}

fn vector_to_bin(v: &Vec<i32>, chance: usize) -> Vec<i32> {
    let mut v2: Vec<i32> = Vec::new();

    for val in v {
        match val {
            _ if *val < (chance as i32) => v2.push(1),
            _ => v2.push(0),
        }
    }
    v2
}

fn automaton(original: &Matrix<i32>) -> Matrix<i32> {
    let mut m = original.clone();

    let death_limit = 3;
    let birth_limit = 4;

    for y in 0..original.rows {
        for x in 0..original.columns {
            let alive = count_alive_neighbours(original, x as i32, y as i32);

            if original[&(x,y)] == 1 {
                if alive < death_limit {
                    m[&(x,y)] = 0;
                } else {
                    m[&(x,y)] = 1;
                }
            } else {
                if alive > birth_limit {
                    m[&(x,y)] = 1;
                } else {
                    m[&(x,y)] = 0;
                }
            }
        }
    }

    m
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

fn count_alive_neighbours(m: &Matrix<i32>, x: i32, y: i32) -> i32 {
    let height = m.rows as i32;
    let width = m.columns as i32;

    let mut alive = 0;

    for i in (x - 1)..(x + 2) {
        for j in (y - 1)..(y + 2) {
            if i < 0 || j < 0 || i >= width || j >= height {
                alive += 1
            } else if i == x && j == y {
                continue;
            } else {
                alive += m[&(i as usize,j as usize)]
            }
        }
    }
    alive
}

pub fn generate_cave(width: usize,
                 height: usize,
                 generations: usize,
                 fill_percentage: usize ) -> Mapplus {
    let mut m = create_random_binary_matrix(width,height, fill_percentage);
    fill_edges(&mut m);

    for _ in 0..generations {
        m = automaton(&m)
    }

    let _rooms: usize;
    let res =  fill_map(&m);
    m = res.0;
    _rooms = res.1;

    let room_sizes = room_sizes(&m, &vec![0,1]);
    println!("{:?}", room_sizes);

    let mut mat : Matrix<char> = Matrix::new(width, height, '.');
    let mut colormat: Matrix<colors::Color> = Matrix::new(width, height, colors::WHITE);

    let colvec = vec![colors::LIGHT_BLUE, colors::RED, colors::GREEN, colors::CYAN,
                      colors::FUCHSIA, colors::AMBER, colors::HAN, colors::PURPLE];

    for y in 0..height {
        for x in 0..width {
            mat[&(x,y)] = match m[&(x,y)] {
                1 => '#',
                _ => '.'
            };
            colormat[&(x,y)] = match m[&(x,y)] {
                1 => colors::WHITE,
                z => colvec[(z as usize) % colvec.len()]
            }
        }
    }

//    print_m_as_map(&m);

//    for y in 0..height {
//        for x in 0..width {
//            print!("{}", mat[&(x,y)]);
//        }
//        println!();
//    }


    let mut mp: Mapplus = Mapplus {mat, col: colormat};
    mp
}

fn print_matrix<T : Debug>(m: &Matrix<T>){
    for i in 0..m.columns {
        for j in 0..m.rows {
            print!("{:?}", m[&(i,j)]);
        }
        println!();
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::time::{Instant};
    use pathfinding::astar;

    #[test]
    fn test_cavegen(){
        let mat = generate_cave(70,42,3,40 );
    }

    #[test]
    fn test_bin_matrix() {
        let matrix = create_random_binary_matrix(10,10, 0);
        for i in 0..matrix.columns {
            for j in 0..matrix.rows {
                print!("{}", matrix[&(i, j)]);
            }
            println!();
        }
    }

    #[test]
    fn test_edgefill(){
        let mut matrix = Matrix::new(30, 12, 0 as i32);
        fill_edges(&mut matrix);
        print_matrix(&matrix);

    }



    #[test]
    fn test_count_alive() {
        let m = Matrix::from_vec(3,3,vec![1,0,1,1,0,1,1,0,1]);

        assert_eq!(count_alive_neighbours(&m, 0, 0), 6);
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
            map.set(r.0, r.1, '2').unwrap();
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