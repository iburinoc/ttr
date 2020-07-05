use std::fmt;

use super::rand::Rand;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Colour {
    Pink,
    White,
    Blue,
    Yellow,
    Orange,
    Black,
    Red,
    Green,
    Rainbow,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Train(u8);

impl fmt::Debug for Train {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}({})", self.colour(), self.0)
    }
}

pub struct TrainDeck {
    deck: Vec<Train>,
    discard: Vec<Train>,
}

impl From<u8> for Train {
    fn from(v: u8) -> Train {
        assert!(v < 110);
        Train(v)
    }
}

impl Train {
    pub fn id(self) -> u8 {
        self.0
    }

    pub fn colour(self) -> Colour {
        use Colour::*;

        match self.0 {
            0..=11 => Pink,
            12..=23 => White,
            24..=35 => Blue,
            36..=47 => Yellow,
            48..=59 => Orange,
            60..=71 => Black,
            72..=83 => Red,
            84..=95 => Green,
            96..=109 => Rainbow,
            _ => unreachable!(),
        }
    }
}

impl TrainDeck {
    pub fn new() -> Self {
        let deck = (0..110).map(Train).collect();
        let discard = Vec::new();

        TrainDeck { deck, discard }
    }

    pub fn deal_one(&mut self, rand: &mut Rand) -> Train {
        let idx = rand.uniform(0, self.deck.len() as u32) as usize;
        self.deck.remove(idx)
    }

    pub fn deal(&mut self, rand: &mut Rand, num: usize) -> Vec<Train> {
        (0..num).map(|_| self.deal_one(rand)).collect()
    }

    pub fn discard<I>(&mut self, cards: I)
    where
        I: IntoIterator<Item = Train>,
    {
        self.discard.extend(cards)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deal() {
        let mut rand = Rand::new(18446744073515119986u64 as u32);
        let mut deck = TrainDeck::new();

        rand.discard(44);

        let deal: Vec<_> = deck.deal(&mut rand, 13).iter().map(|t| t.id()).collect();
        assert_eq!(vec![88, 90, 107, 7, 19, 3, 8, 9, 39, 51, 41, 34, 40], deal);
    }
}
