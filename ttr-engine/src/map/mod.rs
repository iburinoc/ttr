use crate::{rand::Rand, City, Ticket};

mod europe;

pub use europe::Europe;

pub trait Map {
    fn new(rand: &mut Rand) -> Self;

    fn initial_tickets(&mut self, players: u32) -> Vec<Vec<&'static Ticket>>;

    fn draw_ticket(&mut self) -> &'static Ticket;

    fn cities(&self) -> &'static [City];
}
