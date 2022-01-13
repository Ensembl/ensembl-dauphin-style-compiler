pub struct LinearOffsetBuilder {
    fwd: i64,
    rev: i64,
    dud: bool
}

impl LinearOffsetBuilder {
    pub fn new() -> LinearOffsetBuilder {
        LinearOffsetBuilder {
            fwd: 0,
            rev: 0,
            dud: false
        }
    }

    pub fn dud(offset: i64) -> LinearOffsetBuilder {
        LinearOffsetBuilder {
            fwd: offset,
            rev: offset,
            dud: true
        }
    }

    pub fn advance_fwd(&mut self, amt: i64) {
        if !self.dud {
            self.fwd += amt;
        }
    }

    pub fn advance_rev(&mut self, amt: i64) {
        if !self.dud {
            self.rev += amt;
        }
    }

    pub fn fwd(&self) -> i64 { self.fwd }
    pub fn rev(&self) -> i64 { self.rev }
}
