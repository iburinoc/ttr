const MUL: u64 = 48271;
const MOD: u64 = 2147483647;

pub struct Rand(u32);

impl Rand {
    pub fn new(seed: u32) -> Rand {
        Rand(((seed as u64) % MOD) as _)
    }

    pub fn gen(&mut self) -> u32 {
        self.0 = next(self.0);
        self.0
    }

    pub fn uniform(&mut self, low: u32, high: u32) -> u32 {
        assert!(low < high);
        let size = high - low;
        let scaling = (MOD as u32 - 1) / size;
        let max = scaling * size;
        loop {
            let val = self.gen();
            if val < max {
                return val / scaling + low;
            }
        }
    }

    pub fn discard(&mut self, num: usize) {
        (0..num).for_each(|_| {
            self.gen();
        })
    }
}

fn next(s: u32) -> u32 {
    (((s as u64) * MUL) % MOD) as u32
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_next() {
        assert_eq!(next(1), MUL as u32);
    }

    #[test]
    fn test_multiple() {
        let mut rand = Rand::new(18446744071653277558u64 as u32);
        assert_eq!(22, rand.uniform(0, 110));
        assert_eq!(98, rand.uniform(0, 109));
        assert_eq!(19, rand.uniform(0, 108));
        assert_eq!(97, rand.uniform(0, 107));
    }
}
