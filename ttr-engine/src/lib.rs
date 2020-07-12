use std::convert::{TryFrom, TryInto};
use std::fmt::Debug;
use std::ops::Index;

use log::*;
use thiserror::Error;

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

#[derive(Debug, Clone)]
pub enum GameState {
    InitialTickets(InitialTicketSelections),
    Turn { player: u32, state: TurnState },
    GameEnded,
}

#[derive(Debug, Clone)]
pub struct InitialTicketSelections(Vec<InitialTicketState>);

#[derive(Debug, Clone)]
pub struct InitialTicketState {
    options: Vec<&'static Ticket>,
    selected: Option<Vec<&'static Ticket>>,
}

#[derive(Debug, Clone)]
pub enum TurnState {
    Start,
    PickAnotherTrain,
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

#[derive(Debug, Copy, Clone)]
pub enum CardSlot {
    Slot(u8),
    Deck,
}

#[derive(Debug, Error)]
pub enum ActionError {
    #[error("Invalid player number: {0}/{0}")]
    InvalidPlayerNumber(u32, usize),
    #[error("Action is not required from player {0} for current game state: {1:?}")]
    ActionNotRequired(u32, GameState),
    #[error("Action {0} invalid for game state {1:?}")]
    WrongState(&'static str, GameState),
    #[error("Selected tickets {0:?} are not a subset of available tickets {1:?}")]
    BadTicketSelection(Vec<u32>, Vec<&'static Ticket>),
    #[error("Not enough tickets selected: {0:?}/{1}")]
    NotEnoughTickets(Vec<u32>, u32),
    #[error("Index out of range [0, {0}): {1}")]
    IndexOutOfRange(usize, String),
    #[error("Rainbow train taken as second pick")]
    RainbowTrainTakenSecond,
}

macro_rules! match_state {
    ($state:expr, $action:expr, $($pattern:pat => $val:expr),+) => {
        (match $state {
            $($pattern => Ok($val),)*
            _ => Err(ActionError::WrongState($action, $state.clone())),
        })?
    };
}

impl Engine {
    pub fn new<M: Map + 'static>(seed: u32, num_players: u32) -> Self {
        let mut rand = Rand::new(seed);
        let mut map = Box::new(M::new(&mut rand));
        let mut trains = TrainDeck::new();
        let face_up = FaceUp::new(&mut rand, &mut trains);
        let players = (0..num_players)
            .map(|id| {
                let mut p = Player::new(id);
                p.hand = trains.deal(&mut rand, 4);
                p
            })
            .collect();
        let state = GameState::InitialTickets(InitialTicketSelections(
            map.initial_tickets(num_players)
                .into_iter()
                .map(|tickets| InitialTicketState {
                    options: tickets,
                    selected: None,
                })
                .collect(),
        ));
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

    pub fn select_initial_tickets(&mut self, player: u32, ids: &[u32]) -> Result<(), ActionError> {
        self.check_player_number(player)?;
        self.check_action_required(player)?;
        let state = &mut self.state;
        let selections: Option<Vec<_>> = {
            let selections = match_state!(state,
                "initial tickets",
                GameState::InitialTickets(selections) => selections);
            selections.select_tickets(player, ids)?;
            if selections.complete() {
                Some(
                    selections
                        .0
                        .iter_mut()
                        .map(|x| x.selected.take().unwrap())
                        .collect(),
                )
            } else {
                None
            }
        };
        match selections {
            Some(mut tickets) => {
                tickets
                    .iter_mut()
                    .zip(self.players.iter_mut())
                    .for_each(|(state, player)| {
                        std::mem::swap(state, &mut player.tickets);
                    });
                *state = GameState::Turn {
                    player: 0,
                    state: TurnState::Start,
                };
                trace!("Ticket selections complete");
            }
            None => (),
        }

        Ok(())
    }

    pub fn draw_card(
        &mut self,
        player: u32,
        slot: impl TryInto<CardSlot, Error = ActionError>,
    ) -> Result<Train, ActionError> {
        let slot = slot.try_into()?;
        self.check_player_number(player)?;

        let first = match_state!(&self.state,
            "draw train card",
            GameState::Turn { state: TurnState::Start, .. } => true,
            GameState::Turn { state: TurnState::PickAnotherTrain, .. } => false
        );

        match slot {
            CardSlot::Slot(i) => {
                let rainbow = self.face_up[i].colour() == Colour::Rainbow;
                if rainbow && !first {}
            }
            _ => unimplemented!(),
        };
        unimplemented!()
    }

    fn check_player_number(&self, player: u32) -> Result<(), ActionError> {
        if player as usize >= self.players.len() {
            Err(ActionError::InvalidPlayerNumber(player, self.players.len()))
        } else {
            Ok(())
        }
    }

    fn check_action_required(&self, player: u32) -> Result<(), ActionError> {
        if !self.state.action_required(player) {
            Err(ActionError::ActionNotRequired(player, self.state.clone()))
        } else {
            Ok(())
        }
    }
}

impl GameState {
    pub fn action_required(&self, player: u32) -> bool {
        use GameState::*;

        match self {
            InitialTickets(players) => players.0[player as usize].selected.is_none(),
            Turn {
                player: turn_player,
                ..
            } => player == *turn_player,
            GameEnded => false,
        }
    }
}

impl InitialTicketSelections {
    fn select_tickets(&mut self, player: u32, ids: &[u32]) -> Result<(), ActionError> {
        let mut player_state = &mut self.0[player as usize];
        player_state.select_tickets(ids)?;
        trace!(
            "Player {} selected {:?}",
            player,
            player_state.selected.as_deref().unwrap()
        );
        Ok(())
    }

    fn complete(&self) -> bool {
        self.0.iter().all(|x| x.selected.is_some())
    }
}

impl InitialTicketState {
    fn select_tickets(&mut self, ids: &[u32]) -> Result<(), ActionError> {
        let selection: Vec<_> = self
            .options
            .iter()
            .filter(|x| ids.contains(&x.id))
            .copied()
            .collect();
        if ids.len() < 2 {
            // TODO: This may be map-specific
            return Err(ActionError::NotEnoughTickets(ids.to_vec(), 2));
        }

        if selection.len() != ids.len() {
            return Err(ActionError::BadTicketSelection(
                ids.to_vec(),
                self.options.clone(),
            ));
        }

        Ok(())
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

impl Index<u8> for FaceUp {
    type Output = Train;

    fn index(&self, idx: u8) -> &Self::Output {
        &self.0[idx as usize]
    }
}

macro_rules! impl_card_slot {
    ($t:ty) => {
        impl TryFrom<$t> for CardSlot {
            type Error = ActionError;

            fn try_from(idx: $t) -> Result<Self, ActionError> {
                let gen_error = || ActionError::IndexOutOfRange(5, format!("{:?}", idx));
                let idx: u8 = idx.try_into().map_err(|_| gen_error())?;
                if idx >= 5 {
                    Err(gen_error())
                } else {
                    Ok(CardSlot::Slot(idx))
                }
            }
        }
    };
}

impl_card_slot!(u8);
impl_card_slot!(u16);
impl_card_slot!(u32);
impl_card_slot!(u64);
impl_card_slot!(i8);
impl_card_slot!(i16);
impl_card_slot!(i32);
impl_card_slot!(i64);

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

    #[test]
    fn test_initial_tickets() {
        let engine = Engine::new::<map::Europe>(18446744071963584756u64 as u32, 2);

        let player_state = {
            let state = engine.state();
            assert!(state.action_required(0));

            use GameState::*;
            match state {
                InitialTickets(state) => state.0[0].clone(),
                _ => panic!("Unexpected state"),
            }
        };

        let option_ids: Vec<_> = player_state.options.iter().map(|x| x.id).collect();
        assert_eq!(option_ids, vec![10, 43, 5, 21]);
    }
}
