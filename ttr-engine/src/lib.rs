mod map;
mod player;
mod rand;
mod trains;

use map::Map;
use player::Player;
use rand::Rand;
use trains::{Colour, Train, TrainDeck};

pub struct Engine {
    rand: Rand,
    map: Box<dyn Map>,
    trains: TrainDeck,
    face_up: FaceUp,
    players: Vec<Player>,

    state: GameState,
}

#[derive(Debug)]
pub enum GameState {
    InitialTickets(Vec<InitialTicketState>),
    Turn { player: u32, state: TurnState },
}

#[derive(Debug)]
pub struct InitialTicketState {
    options: Vec<&'static Ticket>,
    selected: Option<Vec<&'static Ticket>>,
}

#[derive(Debug)]
pub enum TurnState {
    Start,
    PickAnotherTicket,
    SelectingTickets(Vec<&'static Ticket>),
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
    pub fn new<M: Map + 'static>(seed: u32, num_players: u32) -> Self {
        let mut rand = Rand::new(seed);
        let mut map = Box::new(M::new(&mut rand));
        let mut trains = TrainDeck::new();
        let face_up = FaceUp::new(&mut rand, &mut trains);
        let mut players = (0..num_players)
            .map(|id| {
                let mut p = Player::new(id);
                p.hand = trains.deal(&mut rand, 4);
                p
            })
            .collect();
        let state = GameState::InitialTickets(
            map.initial_tickets(num_players)
                .into_iter()
                .map(|tickets| InitialTicketState {
                    options: tickets,
                    selected: None,
                })
                .collect(),
        );
        Engine {
            rand,
            map,
            trains,
            face_up,
            players,
            state,
        }
    }

    pub fn state(&self) -> &GameState {
        &self.state
    }
}

impl GameState {
    pub fn action_required(&self, player: u32) -> bool {
        match self {
            InitialTickets(players) => players[player as usize].is_some(),
            Turn { player: turn_player, _ } => player == turn_player,
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
        let result = std::mem::replace(&mut self.0[slot], deck.deal_one(rand));
        self.check_for_rainbow(rand, deck);
        result
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

    #[test]
    fn test_hand() {
        use Colour::*;

        let engine = Engine::new::<map::Europe>(18446744071963584756u64 as u32, 2);
        let mut colours: Vec<_> = engine.players[0].hand.iter().map(|x| x.colour()).collect();
        colours.sort();
        assert_eq!(colours, vec![Orange, Red, Green, Green]);
    }
}
