use std::iter;

use lazy_static::lazy_static;

use crate::{rand::Rand, City, Ticket};

use super::Map;

macro_rules! cities {
    ($($id:expr => $name:ident ,)*) => {
        lazy_static! {
            static ref CITIES: Vec<City> = {
                vec![
                    $(City { id: $id, name: stringify!($name) },)*
                ]
            };
        }
    }
}

cities! {
    0 => Amsterdam,
    1 => Angora,
    2 => Athina,
    3 => Barcelona,
    4 => Berlin,
    5 => Brest,
    6 => Brindisi,
    7 => Bruxelles,
    8 => Bucuresti,
    9 => Budapest,
    10 => Cadiz,
    11 => Constantinople,
    12 => Danzic,
    13 => Dieppe,
    14 => Edinburgh,
    15 => Erzurum,
    16 => Essen,
    17 => Frankfurt,
    18 => Kharkov,
    19 => Kobenhavn,
    20 => Kyiv,
    21 => Lisboa,
    22 => London,
    23 => Madrid,
    24 => Marseille,
    25 => Moskva,
    26 => Munchen,
    27 => Palermo,
    28 => Pamplona,
    29 => Paris,
    30 => Petrograd,
    31 => Riga,
    32 => Roma,
    33 => Rostov,
    34 => Sarajevo,
    35 => Sevastopol,
    36 => Smolensk,
    37 => Smyrna,
    38 => Sochi,
    39 => Sofia,
    40 => Stockholm,
    41 => Venezia,
    42 => Venizia,
    43 => Warszawa,
    44 => Wien,
    45 => Wilno,
    46 => Zagrab,
    47 => Zagreb,
    48 => Zurich,
}

macro_rules! tickets {
    ($($id:expr => $c0:ident $c1:ident : $val:expr ,)*) => {
        lazy_static! {
            static ref TICKETS: Vec<Ticket> = {
                let find = |name| {
                    CITIES.iter().find(|c| c.name == name).unwrap()
                };
                vec![
                    $(Ticket {
                        id: $id,
                        city0: find(stringify!($c0)),
                        city1: find(stringify!($c1)),
                        value: $val,
                    },)*
                ]
            };
        }
    };
}

tickets! {
    0 => Amsterdam Pamplona : 7,
    1 => Amsterdam Wilno : 12,
    2 => Angora Kharkov : 10,
    3 => Athina Angora : 5,
    4 => Athina Wilno : 11,
    5 => Barcelona Bruxelles : 8,
    6 => Barcelona Munchen : 8,
    7 => Berlin Bucuresti : 8,
    8 => Berlin Moskva : 12,
    9 => Berlin Roma : 9,
    10 => Brest Marseille : 7,
    11 => Brest Petrograd : 20,
    12 => Brest Venezia : 8,
    13 => Bruxelles Danzic : 9,
    14 => Budapest Sofia : 5,
    15 => Cadiz Stockholm : 21,
    16 => Edinburgh Athina : 21,
    17 => Edinburgh Paris : 7,
    18 => Essen Kyiv : 10,
    19 => Frankfurt Kobenhavn : 5,
    20 => Frankfurt Smolensk : 13,
    21 => Kobenhavn Erzurum : 21,
    22 => Kyiv Petrograd : 6,
    23 => Kyiv Sochi : 8,
    24 => Lisboa Danzic : 20,
    25 => London Berlin : 7,
    26 => London Wien : 10,
    27 => Madrid Dieppe : 8,
    28 => Madrid Zurich : 8,
    29 => Marseille Essen : 8,
    30 => Palermo Constantinople : 8,
    31 => Palermo Moskva : 20,
    32 => Paris Wien : 8,
    33 => Paris Zagreb : 7,
    34 => Riga Bucuresti : 10,
    35 => Roma Smyrna : 8,
    36 => Rostov Erzurum : 5,
    37 => Sarajevo Sevastopol : 8,
    38 => Smolensk Rostov : 8,
    39 => Sofia Smyrna : 5,
    40 => Stockholm Wien : 11,
    41 => Venizia Constantinople : 10,
    42 => Warszawa Smolensk : 6,
    43 => Zagrab Brindisi : 6,
    44 => Zurich Brindisi : 6,
    45 => Zurich Budapest : 6,
}

pub struct Europe {
    smalls: Vec<&'static Ticket>,
    bigs: Vec<&'static Ticket>,
}

impl Map for Europe {
    fn new(rand: &mut Rand) -> Self {
        let (bigs, smalls): (Vec<_>, Vec<_>) = TICKETS.iter().partition(|t| t.value >= 20);
        let mut shuffle = |mut vec: Vec<_>| {
            let mut shuffled = Vec::new();
            for i in (2..=vec.len()).rev() {
                let idx = rand.uniform(0, i as u32);
                let val = vec.remove(idx as usize);
                shuffled.push(val);
            }
            shuffled.push(vec.remove(0));
            shuffled
        };
        let bigs = shuffle(bigs);
        let smalls = shuffle(smalls);

        Europe { smalls, bigs }
    }

    fn initial_tickets(&mut self, players: u32) -> Vec<Vec<&'static Ticket>> {
        (0..players)
            .map(|_| {
                self.smalls
                    .drain(0..3)
                    .chain(iter::once(self.bigs.remove(0)))
                    .collect()
            })
            .collect()
    }

    fn draw_ticket(&mut self) -> &'static Ticket {
        self.smalls.remove(0)
    }

    fn cities(&self) -> &'static [City] {
        CITIES.as_slice()
    }
}
