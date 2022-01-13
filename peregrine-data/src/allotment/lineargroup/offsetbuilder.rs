pub struct LinearOffsetBuilder {
    size: [i64;2],
    dud: bool
}

impl LinearOffsetBuilder {
    pub fn new() -> LinearOffsetBuilder {
        LinearOffsetBuilder {
            size: [0,0],
            dud: false
        }
    }

    pub fn dud(offset: i64) -> LinearOffsetBuilder {
        LinearOffsetBuilder {
            size: [offset,0],
            dud: true
        }
    }

    pub fn advance(&mut self, amt: i64, fwd: bool) {
        if !self.dud {
            self.size[if fwd {1} else {0}] += amt;
        }
    }


    pub fn size(&self, fwd: bool) -> i64 { self.size[if fwd { 1 } else { 0 }] }
    pub fn total_size(&self) -> i64 { self.size[0] + self.size[1] }
}
