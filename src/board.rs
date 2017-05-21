use std::collections::HashMap;
use std::fmt;

use point::*;
use terrain::*;

pub struct Board {
    dimensions: Point,
    tiles: Vec<Terrain>,
    features: HashMap<Point, Feature>
}

impl Board {
    pub fn new(x: i32, y: i32, fill: Terrain) -> Self {
        let mut tiles = Vec::new();
        for _ in 0..x {
            for _ in 0..y {
                tiles.push(fill);
            }
        }
        Board {
            dimensions: Point::new(x, y),
            tiles: tiles,
            features: HashMap::new(),
        }
    }

    pub fn in_bounds(&self, pt: &Point) -> bool {
        *pt >= Point::new(0, 0) && *pt < self.dimensions
    }

    pub fn set(&mut self, pt: &Point, val: Terrain) {
        if self.in_bounds(pt) {
            let idx = (pt.y * self.dimensions.x + pt.x) as usize;
            let mut v = self.tiles.get_mut(idx).unwrap();
            *v = val;
        }
    }

    pub fn get(&self, pt: &Point) -> Terrain {
        if self.in_bounds(pt) {
            let idx = (pt.y * self.dimensions.x + pt.x) as usize;
            self.tiles.get(idx).unwrap().clone()
        } else {
            Terrain::Nothing
        }
    }

    pub fn width(&self) -> i32 {
        self.dimensions.x
    }

    pub fn height(&self) -> i32 {
        self.dimensions.y
    }
}

impl fmt::Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.dimensions.x {
            for j in 0..self.dimensions.y {
                let pos = Point::new(i, j);
                let ch = self.get(&pos).to_char();
                write!(f, "{}", ch)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}
