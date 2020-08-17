#[derive(Clone)]
pub struct Scale(u64);

impl Scale {
    pub fn new(scale: u64) -> Scale {
        Scale::new(scale)
    }

    /* direction-agnostic next scale, eg for ranges */
    pub fn next_scale(&self) -> Scale {
        Scale(self.0+1)
    }

    /* an index for ranges. Don't compute with this! */
    pub fn get_index(&self) -> u64 {
        self.0
    }
}