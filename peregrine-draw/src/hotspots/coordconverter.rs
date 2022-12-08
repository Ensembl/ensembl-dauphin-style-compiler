use peregrine_data::{SpaceBasePointRef, AuxLeaf};

use crate::stage::axis::UnitConverter;

pub(super) struct CoordToPxConverter {
    left: f64,
    bp_per_px: f64,
    car_px_left: f64,
    px_per_carriage: f64
}

impl CoordToPxConverter {
    pub(super) fn new(context: &UnitConverter, left: f64, bp_per_carriage: f64) -> Option<CoordToPxConverter> {
        let bp_per_px = context.px_delta_to_bp(1.);
        let car_px_left = context.bp_to_pos_px(left).ok();
        let car_px_left = if let Some(x) = car_px_left { x } else { return None; };
        Some(CoordToPxConverter {
            px_per_carriage: bp_per_carriage / bp_per_px,
            bp_per_px,
            car_px_left,
            left,
        })
    }

    pub(super) fn tracking_coord_to_px(&self, c: &SpaceBasePointRef<f64,AuxLeaf>) -> f64 {
        (c.base - self.left) / self.bp_per_px + self.car_px_left + c.tangent
    }

    pub(super) fn px_to_car_prop(&self, px: f64) -> f64 {
        (px - self.car_px_left) / self.px_per_carriage
    }
}
