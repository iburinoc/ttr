mod map;
mod rand;
mod trains;

use map::Map;
use rand::Rand;
use trains::TrainDeck;

pub struct Engine {
    rand: Rand,
    map: Box<dyn Map>,
    trains: TrainDeck,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct City {
    pub id: u32,
    pub name: &'static str,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Ticket {
    pub id: u32,
    pub city0: &'static City,
    pub city1: &'static City,
    pub value: u32,
}

impl Engine {
    pub fn new<M: Map + 'static>(seed: u32, players: u32) -> Self {
        let mut rand = Rand::new(seed);
        let map = Box::new(M::new(&mut rand));
        let trains = TrainDeck::new();
        Engine { rand, map, trains }
    }
}
