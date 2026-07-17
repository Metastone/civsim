use crate::algorithms::rng::random_range_int;

#[derive(Clone)]
pub enum Direction {
    North,
    NorthNorthEast,
    NorthEast,
    NorthEastEast,
    East,
    SouthEastEast,
    SouthEast,
    SouthSouthEast,
    South,
    SouthSouthWest,
    SouthWest,
    SouthWestWest,
    West,
    NorthWestWest,
    NorthWest,
    NorthNorthWest,
}

impl TryFrom<usize> for Direction {
    type Error = &'static str;
    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Direction::North),
            1 => Ok(Direction::NorthNorthEast),
            2 => Ok(Direction::NorthEast),
            3 => Ok(Direction::NorthEastEast),
            4 => Ok(Direction::East),
            5 => Ok(Direction::SouthEastEast),
            6 => Ok(Direction::SouthEast),
            7 => Ok(Direction::SouthSouthEast),
            8 => Ok(Direction::South),
            9 => Ok(Direction::SouthSouthWest),
            10 => Ok(Direction::SouthWest),
            11 => Ok(Direction::SouthWestWest),
            12 => Ok(Direction::West),
            13 => Ok(Direction::NorthWestWest),
            14 => Ok(Direction::NorthWest),
            15 => Ok(Direction::NorthNorthWest),
            _ => Err("invalid direction"),
        }
    }
}

impl Direction {
    pub fn random() -> Self {
        Direction::try_from(random_range_int(0, 16)).unwrap()
    }
}
