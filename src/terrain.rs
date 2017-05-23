use rand::{self, Rng};

#[derive(Eq, PartialEq, Clone, Debug)]
pub enum Feature {
    Door,
    UpStair,
    DownStair,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum Terrain {
    Wall,
    Floor,
    Important,
    Nothing,
}

impl Terrain {
    pub fn to_char(&self) -> char {
        match *self {
            Terrain::Wall => '#',
            Terrain::Floor => '.',
            Terrain::Important => '$',
            Terrain::Nothing => ' ',
        }
    }

    pub fn n(&self) -> usize {
        match *self {
            Terrain::Wall => 0,
            Terrain::Floor => 1,
            Terrain::Important => 0,
            Terrain::Nothing => 0,
        }
    }

    pub fn is_blocking(&self) -> bool {
        match *self {
            Terrain::Wall | Terrain::Nothing => true,
            _             => false,
        }
    }
}
