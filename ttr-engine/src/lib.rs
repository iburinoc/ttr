mod map;
mod rand;
mod trains;

use map::Map;
use rand::Rand;
use trains::{Colour, Train, TrainDeck};

pub struct Engine {
    rand: Rand,
    map: Box<dyn Map>,
    trains: TrainDeck,
    face_up: FaceUp,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct City {
    pub id: u32,
    pub name: &'static str,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ticket {
    pub id: u32,
    pub city0: &'static City,
    pub city1: &'static City,
    pub value: u32,
}

#[derive(Debug, Copy, Clone)]
struct FaceUp([Train; 5]);

impl Engine {
    pub fn new<M: Map + 'static>(seed: u32, players: u32) -> Self {
        let mut rand = Rand::new(seed);
        let map = Box::new(M::new(&mut rand));
        let mut trains = TrainDeck::new();
        let face_up = FaceUp::new(&mut rand, &mut trains);
        Engine {
            rand,
            map,
            trains,
            face_up,
        }
    }
}

impl FaceUp {
    fn new(rand: &mut Rand, deck: &mut TrainDeck) -> Self {
        use std::convert::TryInto as _;
        let mut cards = FaceUp(deck.deal(rand, 5).as_slice().try_into().unwrap());
        cards.check_for_rainbow(rand, deck);

        cards
    }

    fn check_for_rainbow(&mut self, rand: &mut Rand, deck: &mut TrainDeck) {
        while self.num_rainbow() >= 3 {
            deck.discard(self.0.as_ref().iter().copied());
            self.0.copy_from_slice(deck.deal(rand, 5).as_slice());
        }
    }

    fn num_rainbow(&self) -> usize {
        self.0
            .iter()
            .filter(|x| x.colour() == Colour::Rainbow)
            .count()
    }

    fn draw(&mut self, rand: &mut Rand, deck: &mut TrainDeck, slot: usize) -> Train {
        std::mem::replace(&mut self.0[slot], deck.deal_one(rand))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_face_up() {
        use Colour::*;

        let engine = Engine::new::<map::Europe>(27683789, 2);
        let colours: Vec<_> = engine.face_up.0.iter().map(|x| x.colour()).collect();
        assert_eq!(colours, vec![White, Orange, Pink, Green, Rainbow]);
    }
}
