/* We care about pixel size differences in the around 2-5% range, or five binary bits.
 * eg. 
 * 11010xxxxxx = -1728
 * 11011xxxxxx = 1728 - 1792
 * 11100xxxxxx = 1792+
 * 
 * We care must about the /fewest/ pixels that might be on the screen (bumping), so we round down and store x=0
 * as our internal representation. We only care about x.
 */

const ROUNDING_BITS : u32 = 5;

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