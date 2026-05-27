#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PCG {
    state: u64,
    inc: u64,
}

impl PCG {
    pub fn new() -> Self {
        let init_state: u64 = 42;
        let init_seq: u64 = 54;

        let mut rng = PCG {
            state: 0,
            inc: (init_seq << 1) | 1,
        };

        rng.random();
        rng.state += init_state;
        rng.random();

        rng
    }
    pub fn new_from_seed(init_state: u64, init_seq: u64) -> Self {
        let mut rng = PCG {
            state: 0,
            inc: (init_seq << 1) | 1,
        };

        rng.random();
        rng.state += init_state;
        rng.random();

        rng
    }
    pub fn random(&mut self) -> u32 {
        let oldstate = self.state;
        self.state = oldstate
            .wrapping_mul(6364136223846793005)
            .wrapping_add(self.inc);
        let xorshifted = (((oldstate >> 18) ^ oldstate) >> 27) as u32;
        let rot = (oldstate >> 59) as u32;
        xorshifted.rotate_right(rot)
    }
    pub fn random_float(&mut self) -> f32 {
        self.random() as f32 / (u32::MAX as f32 + 1.0)
    }
}
impl Default for PCG {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use crate::pcg::PCG;

    #[test]
    fn test_new_from_seed() {
        let mut pcg = PCG::new();
        pcg.state = 1753877967969059832;
        pcg.inc = 109;
        for expected in [
            2707161783, 2068313097, 3122475824, 2211639955, 3215226955, 3421331566,
        ] {
            assert_eq!(expected, pcg.random())
        }
    }

    #[test]
    /// checks that pcg is deterministic
    fn test_deterministic_new_from_seed() {
        let mut a = PCG::new_from_seed(42, 54);
        let mut b = PCG::new_from_seed(42, 54);

        for _ in 0..1000 {
            assert_eq!(a.random(), b.random());
        }
    }
}
