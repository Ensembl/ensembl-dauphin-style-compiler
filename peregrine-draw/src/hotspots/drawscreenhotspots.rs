use std::{ops::Range};
use peregrine_data::{LeafStyle, SingleHotspotEntry, Scale, HotspotGroupEntry, SpaceBasePoint, SpaceBasePointRef};
use peregrine_toolkit::{hotspots::hotspotstore::{HotspotStoreProfile}, ubail};
use crate::{Message, stage::{stage::ReadStage, axis::UnitConverter}};
use super::drawhotspotstore::{PointPair, DrawHotspotStore};

const HORIZ_ZONES : u64 = 10;
const VERT_ZONE_HEIGHT : u64 = 200;

fn order(a: f64, b: f64) -> (f64,f64) { (a.min(b),a.max(b)) }

struct CoordToPxConverter {
    left: f64,
    bp_per_px: f64,
    car_px_left: f64,
    px_per_carriage: f64
}

impl CoordToPxConverter {
    fn new(context: &UnitConverter, left: f64, bp_per_carriage: f64) -> Option<CoordToPxConverter> {
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

    fn coord_to_px(&self, c: &SpaceBasePointRef<f64,LeafStyle>) -> f64 {
        (c.base - self.left) / self.bp_per_px + self.car_px_left + c.tangent
    }

    fn px_to_car_prop(&self, px: f64) -> f64 {
        (px - self.car_px_left) / self.px_per_carriage
    }
}

struct DrawingHotspotProfile {
    left: f64,
    bp_per_carriage: f64,
    max_bp_per_px: f64
}

impl DrawingHotspotProfile {
    fn new(left: f64, bp_per_carriage: f64, max_bp_per_px: f64) -> DrawingHotspotProfile { 
        DrawingHotspotProfile {
            left, bp_per_carriage, max_bp_per_px
        } 
    }

    fn bp_to_carriage_prop(&self, bp: f64) -> f64 {
        (bp - self.left) / self.bp_per_carriage
    }

    fn max_bp_pos(&self, c: &SpaceBasePoint<f64,LeafStyle>, neg_bias: bool) -> f64 {
        let px = c.tangent * self.max_bp_per_px;
        let px_extra = if neg_bias { px.min(0.) } else { px.max(0.) };
        c.base + px_extra
    }

    /* We don't know which will ultimately be on the left or right. This is annoying because
     * we need to add the tangent only if it extends the region, and it can be +ve or -ve.
     * If point A is at 1+5k and B at 2+k then A will be to the right of B when k>1/4 and to the
     * left otherwise. So the only thing to do for independence of k is to assume each can be at
     * the left or right and take the maximum extent.
     */
    fn max_bp_pair_pos(&self, p: &PointPair, neg_bias: bool) -> f64 {
        let a = self.max_bp_pos(&p.0,neg_bias);
        let b = self.max_bp_pos(&p.1,neg_bias);
        if neg_bias { a.min(b) } else { a.max(b) }
    }

    fn converter(&self, converter: &UnitConverter) -> Option<CoordToPxConverter> {
        CoordToPxConverter::new(converter,self.left,self.bp_per_carriage)
    }
}

impl HotspotStoreProfile<SingleHotspotEntry> for DrawingHotspotProfile {
    type Coords = (f64,f64);
    type Area = PointPair;
    type Context = UnitConverter;

    fn intersects(&self, context: &UnitConverter, coords: &(f64,f64), entry: &SingleHotspotEntry) -> bool {
        let coord_to_px = ubail!(self.converter(context),false);
        entry.coordinates().map(|(c1,c2)| {            
            coords.0 >= coord_to_px.coord_to_px(&c1) && 
            coords.0 <= coord_to_px.coord_to_px(&c2) && 
            coords.1 >= *c1.normal && 
            coords.1 <= *c2.normal
        }).unwrap_or(false)
    }

    fn get_zone(&self, context: &UnitConverter, position: &(f64,f64)) -> Option<(usize,usize)> {
        let coord_to_px = ubail!(self.converter(context),None);
        let carriage_prop = coord_to_px.px_to_car_prop(position.0);
        if carriage_prop < 0. || carriage_prop > 1. { return None; }
        Some((
            (carriage_prop * HORIZ_ZONES as f64).floor() as usize,
            (position.1 / VERT_ZONE_HEIGHT as f64).floor() as usize
        ))
    }

    fn add_zones(&self, a: &PointPair) -> (Range<usize>,Range<usize>) {
        let left_scr = self.bp_to_carriage_prop(self.max_bp_pair_pos(&a,true));
        let right_scr = self.bp_to_carriage_prop(self.max_bp_pair_pos(&a,false));
        let (top_px,bottom_px) = order(a.0.normal,a.1.normal);
        (
            (
                 ((left_scr*(HORIZ_ZONES as f64)).floor() as usize) ..
                (((right_scr*(HORIZ_ZONES as f64)).floor() as usize)+1)
            ),(
                 ((top_px/(VERT_ZONE_HEIGHT as f64)) as usize) ..
                (((bottom_px/(VERT_ZONE_HEIGHT as f64)) as usize)+1)
            )
        )
    }
}

pub(super) struct DrawScreenHotspots {
    store: DrawHotspotStore
}

impl DrawScreenHotspots {
    fn make_profile(min_px_per_screen: f64, scale: &Option<Scale>, left: f64) -> DrawingHotspotProfile {
        let max_bp_per_screen = scale.as_ref().map(|s| s.bp_per_screen_range().1).unwrap_or(1) as f64;
        let max_bp_per_px = max_bp_per_screen / min_px_per_screen;
        let bp_per_carriage = scale.as_ref().map(|s| s.bp_in_carriage()).unwrap_or(1) as f64;
        DrawingHotspotProfile::new(left,bp_per_carriage,max_bp_per_px)
    }

    pub(super) fn new(min_px_per_screen: f64, scale: &Option<Scale>, left: f64, entries: &[HotspotGroupEntry]) -> Result<DrawScreenHotspots,Message> {
        Ok(DrawScreenHotspots {
            store: DrawHotspotStore::new(Box::new(Self::make_profile(min_px_per_screen,scale,left)),entries)?
        })
    }

    pub(crate) fn get_hotspot(&self, stage: &ReadStage, position_px: (f64,f64)) -> Result<Vec<SingleHotspotEntry>,Message> {
        self.store.get_hotspot(stage,position_px)
    }
}
