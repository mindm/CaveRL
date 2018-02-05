use std::fmt::Debug;

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct NodeMap<C> {
    pub width: usize,
    pub height: usize,
    grid: Vec<C>
}

impl<C> NodeMap<C> {
    pub fn from_vec(width: usize, height: usize, values: Vec<C>) -> Self {
        assert_eq!(
            width * height,
            values.len(),
            "length of vector does not correspond to announced dimensions"
        );
        NodeMap{
            width,
            height,
            grid: values,
        }
    }
}

impl<C: Clone> NodeMap<C> {
    pub fn new(width: usize, height: usize, init: C) -> NodeMap<C>{
        let mut v = Vec::with_capacity(width * height);
        v.resize(width * height, init);
        NodeMap {
            width,
            height,
            grid: v,
        }
    }

    pub fn get(&self, p: &( usize, usize))-> C {
        self.grid[p.1 * self.width + p.0].clone()
    }

    pub fn set(&mut self, p: &( usize, usize), value: C) {
        self.grid[p.1 * self.width + p.0] = value;
    }
}

#[allow(dead_code)]
impl<C: Debug + Clone> NodeMap<C> {
    pub fn print(&self) {
        for j in 0..self.height {
            for i in 0..self.width {
                print!("{:?}", self.get(&(i,j)))
            }
            println!();
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grid_new() {
        let new = NodeMap::new(3, 5, 0);
        println!("{:?}", new);
    }

    #[test]
    fn test_set_get() {
        let mut new = NodeMap::new(3, 2, 0);
        new.set(&(1, 0), 2);
        println!("{:?}", new.get(&(0, 1)));
        println!("{:?}", new);
        new.print();
    }

    #[test]
    fn test_from_vec(){
        let new = NodeMap::from_vec(3,3, vec![0,0,1,0,0,0,0,0,0]);
        new.print();
        assert_eq!(new.get(&(2,0)), 1);
    }
}
