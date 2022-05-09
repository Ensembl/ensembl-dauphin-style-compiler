use crate::{ShapeRequestGroup};

use super::rangeused::RangeUsed;

pub struct BpPxConverter {
    bp_per_carriage: f64,
    min_px_per_carriage: f64,
    bp_start: f64
}

impl BpPxConverter {
    fn calc_bp_per_carriage(request: &ShapeRequestGroup) -> f64 {
        request.region().scale().bp_in_carriage() as f64
    }

    pub(crate) fn new(extent: Option<&ShapeRequestGroup>) -> BpPxConverter {
        let bp_per_carriage = extent.map(|x| Self::calc_bp_per_carriage(x)).unwrap_or(0.);
        let min_px_per_carriage = extent.map(|x| {
            x.pixel_size().min_px_per_carriage() as f64
        }).unwrap_or(1.);
        BpPxConverter {
            bp_per_carriage, min_px_per_carriage,
            bp_start: extent.map(|x| x.region().min_value() as f64).unwrap_or(0.)
        }
    }

    #[cfg(test)]
    pub(crate) fn new_test() -> BpPxConverter {
        BpPxConverter {
            min_px_per_carriage: 1.,
            bp_per_carriage: 1.,
            bp_start: 0.
        }
    }

    pub fn full_carriage_range(&self, base_range: &RangeUsed<f64>, pixel_range: &RangeUsed<f64>) -> RangeUsed<f64> {
        base_range.plus_scalar(-self.bp_start).carriage_range(pixel_range,self.min_px_per_carriage,self.bp_per_carriage)
    }
}
