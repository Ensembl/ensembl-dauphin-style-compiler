pub struct LinearOffsetBuilder {
    size: i64,
    dud: bool
}

impl LinearOffsetBuilder {
    fn real_new(size: i64, dud: bool) -> LinearOffsetBuilder { LinearOffsetBuilder { size, dud } }

    pub fn new() -> LinearOffsetBuilder { LinearOffsetBuilder::real_new(0,false) }
    pub fn dud(size: i64) -> LinearOffsetBuilder { LinearOffsetBuilder::real_new(size,true) }

    pub fn advance(&mut self, amt: i64) {
        if !self.dud {
            self.size += amt;
        }
    }

    pub fn primary(&self) -> i64 { self.size }
}
