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

    pub fn n(&self) -> u32 {
        match *self {
            Terrain::Wall => 12,
            Terrain::Floor => 4,
            Terrain::Important => 29,
            Terrain::Nothing => 31,
        }
    }

    pub fn is_blocking(&self) -> bool {
        match *self {
            Terrain::Wall | Terrain::Nothing => true,
            _             => false,
        }
    }
}
