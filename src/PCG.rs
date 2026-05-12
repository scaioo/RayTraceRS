pub struct PCG {
    state: u64,
    inc: u64,
}

impl PCG {
    pub fn __init__(&mut self) -> &mut PCG {
        let init_state: u64 = 42;
        let init_seq: u64 = 54;
        self.state=0;
        self.inc = (init_seq>>1) | 1;
        self.random();
        self.state += init_state;
        self.random();
        self
    }

    pub fn random(&mut self)-> u32 {
        let oldstate = self.state;
        self.state = oldstate.wrapping_mul(6364136223846793005).wrapping_add(self.inc);
        let xorshifted = (((oldstate >> 18)^oldstate) >> 27) as u32;
        let rot = (oldstate >> 59) as u32;
        xorshifted.rotate_right(rot)
    }
}