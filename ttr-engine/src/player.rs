use super::{Ticket, Train};

#[derive(Debug)]
pub struct Player {
    pub id: u32,
    pub hand: Vec<Train>,
    pub tickets: Vec<&'static Ticket>,
    pub trains: u32,
}

impl Player {
    pub fn new(id: u32) -> Self {
        Player {
            id,
            hand: Vec::new(),
            tickets: Vec::new(),
            trains: 45, // TODO: This may be map dependent
        }
    }
}
