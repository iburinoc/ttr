mod map;
mod rand;
mod trains;

use rand::Rand;
use trains::TrainDeck;

pub struct Engine {
    rand: Rand,
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
