use std::{ops::Range};
use peregrine_data::{SingleHotspotEntry, Scale, HotspotGroupEntry, SpaceBasePoint, AuxLeaf, SingleHotspotResult };
use peregrine_toolkit::{hotspots::hotspotstore::{HotspotStoreProfile, HotspotPosition}, ubail, error::Error};
use crate::{Message, stage::{stage::ReadStage, axis::UnitConverter}};
use super::{drawhotspotstore::{PointPair, DrawHotspotStore}, coordconverter::CoordToPxConverter};

const HORIZ_ZONES : usize = 10;
const VERT_ZONE_HEIGHT : usize = 200;

fn order(a: f64, b: f64) -> (f64,f64) { (a.min(b),a.max(b)) }

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

    fn max_bp_pos(&self, c: &SpaceBasePoint<f64,AuxLeaf>, neg_bias: bool) -> f64 {
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
    type Area = PointPair;
    type Context = UnitConverter;

    fn diagonalise(&self, x: usize, y: usize) -> usize { x + y*HORIZ_ZONES }

    fn bounds(&self, context: &UnitConverter, entry: &SingleHotspotEntry) -> Option<HotspotPosition> {
        let coord_to_px = ubail!(self.converter(context),None);
        let (at_coords,run) = entry.coordinates();
        at_coords.map(|(c1,c2)| {
            let mut obj_left = coord_to_px.tracking_coord_to_px(&c1);
            let mut obj_right = coord_to_px.tracking_coord_to_px(&c2);
            let left_rail = coord_to_px.left_rail();
            if let Some(run) = run {
                if obj_left < left_rail && run >= left_rail {
                    obj_right = (obj_right-obj_left)+left_rail;
                    obj_left = left_rail;
                }
            }
            Some(HotspotPosition {
                top: *c1.normal,
                bottom: *c2.normal,
                left: obj_left,
                right: obj_right
            })
        }).unwrap_or(None)
    }

    fn get_zones(&self, context: &UnitConverter, position: &(f64,f64)) -> Vec<(usize,usize)> {
        let coord_to_px = ubail!(self.converter(context),vec![]);
        let carriage_prop = coord_to_px.px_to_car_prop(position.0);
        if carriage_prop < 0. || carriage_prop >= 1. { return vec![]; }
        vec![(
            (carriage_prop * HORIZ_ZONES as f64).floor() as usize,
            (position.1 / VERT_ZONE_HEIGHT as f64).floor() as usize
        )]
    }

    fn add_zones(&self, a: &PointPair) -> Option<(Range<usize>,Range<usize>)> {
        let left_scr = self.bp_to_carriage_prop(self.max_bp_pair_pos(&a,true));
        let right_scr = self.bp_to_carriage_prop(self.max_bp_pair_pos(&a,false));
        let (top_px,bottom_px) = order(a.0.normal,a.1.normal);
        let horiz = if a.2.is_some() {
            0..HORIZ_ZONES+1
        } else {
            ((left_scr.clamp(0.,1.)*(HORIZ_ZONES as f64)).floor() as usize) ..
           (((right_scr.clamp(0.,1.)*(HORIZ_ZONES as f64)).floor() as usize)+1)
        };
        let vert =
            ((top_px/(VERT_ZONE_HEIGHT as f64)) as usize) ..
           (((bottom_px/(VERT_ZONE_HEIGHT as f64)) as usize)+1)
        ;
        Some((horiz,vert))
    }
}

pub(super) struct TrackingHotspots {
    store: DrawHotspotStore<UnitConverter>
}

impl TrackingHotspots {
    fn make_profile(min_px_per_screen: f64, scale: &Option<Scale>, left: f64) -> DrawingHotspotProfile {
        let max_bp_per_screen = scale.as_ref().map(|s| s.bp_per_screen_range().1).unwrap_or(1) as f64;
        let max_bp_per_px = max_bp_per_screen / min_px_per_screen;
        let bp_per_carriage = scale.as_ref().map(|s| s.bp_in_carriage()).unwrap_or(1) as f64;
        DrawingHotspotProfile::new(left,bp_per_carriage,max_bp_per_px)
    }

    pub(super) fn new(min_px_per_screen: f64, scale: &Option<Scale>, left: f64, entries: &[HotspotGroupEntry]) -> Result<TrackingHotspots,Error> {
        Ok(TrackingHotspots {
            store: DrawHotspotStore::new(Box::new(Self::make_profile(min_px_per_screen,scale,left)),entries)?
        })
    }

    pub(crate) fn get_hotspot(&self, stage: &ReadStage, position_px: (f64,f64)) -> Result<Vec<SingleHotspotResult>,Message> {
        let converter = stage.x().unit_converter()?;
        self.store.get_hotspot(&converter,position_px)
    }
}
