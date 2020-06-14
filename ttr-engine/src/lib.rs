mod rand;
mod trains;

use rand::Rand;
use trains::TrainDeck;

pub struct Engine {
    rand: Rand,
    trains: TrainDeck,
}
