/* We care about pixel size differences in the around 5-10% range, or four binary bits.
 * eg. 
 * 1101xxxxxxx = 1792-
 * 1110xxxxxxx = 1792 - 1920
 * 1111xxxxxxx = 1920+
 * 
 * We care must about the /fewest/ pixels that might be on the screen (bumping), so we round down and store x=0
 * as our internal representation. We only care about x.
 */

const ROUNDING_BITS : u32 = 4;

fn round(input: u32, delta: u32) -> u32 {
    let shifts = (u32::BITS - input.leading_zeros() - ROUNDING_BITS).max(0);
    ((input >> shifts)+delta) << shifts
}


fn round_down(input: u32) -> u32 { round(input,0) }
fn round_up(input: u32) -> u32 { round(input,1) }

#[cfg_attr(debug_assertions,derive(Debug))]
#[derive(Clone,PartialEq,Eq,Hash,PartialOrd,Ord)]
pub struct PixelSize(u32,u32);

impl PixelSize {
    pub(crate) fn new(px: u32) -> PixelSize {
        PixelSize(round_down(px),round_up(px))
    }

    pub fn min_px_per_carriage(&self) -> u32 { self.0 }
    pub fn max_px_per_carriage(&self) -> u32 { self.1 }
}