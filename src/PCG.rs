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

    pub fn random(&mut self)-> u64 {
        let oldstate = self.state;
        self.state =

    }
}