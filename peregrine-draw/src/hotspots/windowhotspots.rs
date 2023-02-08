use peregrine_data::{SingleHotspotEntry, CoordinateSystem, SpaceBasePointRef, AuxLeaf};
use peregrine_toolkit::{hotspots::hotspotstore::{HotspotStoreProfile, HotspotPosition}, ubail};
use crate::stage::axis::UnitConverter;
use super::{drawhotspotstore::PointPair, coordconverter::CoordToPxConverter};

const STRIPE_SIZE : f64 = 50.;

/* (0,0) whole screen, always check
 * (1,y) positive y
 * (2,y) negative y
 */

fn order(a: f64, b: f64) -> (f64,f64) { (a.min(b),a.max(b)) }

fn round(y: f64) -> usize { (y.abs()/STRIPE_SIZE).floor() as usize }

fn y_intersect(height: f64, offset: f64, mut y1: f64, mut y2: f64) -> Option<(f64,f64)> {
    if y1 < 0. { y1 += height; }
    if y2 < 0. { y2 += height; }
    let (y1,y2) = order(y1,y2);
    Some((y1.min(y2)+offset,y1.max(y2)+offset))
}

fn x_intersect(coord_to_px: &CoordToPxConverter, width: f64, c1: &SpaceBasePointRef<f64,AuxLeaf>, c2: &SpaceBasePointRef<f64,AuxLeaf>) -> Option<(f64,f64)> {
    let (px1,px2) = match c1.allotment.coord_system {
        CoordinateSystem::TrackingWindow => {
            (
                coord_to_px.tracking_coord_to_px(&c1),
                coord_to_px.tracking_coord_to_px(&c2)
            )
        },
        CoordinateSystem::Window | CoordinateSystem::Content => {
            (
                (width * c1.base) + c1.tangent,
                (width * c2.base) + c2.tangent
            )
        },
        _ => { return None; }
    };
    Some(order(px1,px2))
}

pub(super) struct WindowHotspotProfile {
    left: f64,
    bp_per_carriage: f64
}

impl WindowHotspotProfile {
    pub(crate) fn new(left: f64, bp_per_carriage: f64) -> WindowHotspotProfile { 
        WindowHotspotProfile { left, bp_per_carriage }
    }

    fn converter(&self, converter: &UnitConverter) -> Option<CoordToPxConverter> {
        CoordToPxConverter::new(converter,self.left,self.bp_per_carriage)
    }
}

impl HotspotStoreProfile<SingleHotspotEntry> for WindowHotspotProfile {
    type Area = PointPair;
    type Context = (UnitConverter,f64,f64,f64);

    fn diagonalise(&self, x: usize, y: usize) -> usize { 
        if x == 0 { 0 } else if x == 1 { 2*y+1 } else { 2*y+2 }
    }

    fn get_zones(&self, context: &(UnitConverter,f64,f64,f64), coords: &(f64,f64)) -> Vec<(usize,usize)> {
        vec![
            (0,0),
            (1,round(coords.1)),
            (2,round(context.2-coords.1))
        ]
    }

    fn bounds(&self, context: &(UnitConverter,f64,f64,f64), value: &SingleHotspotEntry) -> Option<HotspotPosition> {
        let coord_to_px = ubail!(self.converter(&context.0),None);
        let (at_coords,_) = value.coordinates();
        let out = at_coords.map(|(c1,c2)| {
            match c1.allotment.coord_system {
                CoordinateSystem::TrackingWindow |
                CoordinateSystem::Window => {
                    Some((
                        x_intersect(&coord_to_px,context.1,&c1,&c2),
                        y_intersect(context.2,context.3,*c1.normal,*c2.normal)
                    ))
                },
                CoordinateSystem::Content => {
                    Some((
                        x_intersect(&coord_to_px,context.1,&c1,&c2),
                        y_intersect(context.2,0.,*c1.normal,*c2.normal)
                    ))
                },
                _ => None
            }
        }).unwrap_or(None);
        out.and_then(|(a,b)| a.zip(b).map(|((left,right),(top,bottom))|
            HotspotPosition { top, bottom, left, right }
        ))
    }

    fn add_zones(&self, a: &PointPair) -> Option<(std::ops::Range<usize>,std::ops::Range<usize>)> {
        let s0 = round(a.0.normal);
        let s1 = round(a.1.normal);
        let (mut s0,mut s1) = (s0.min(s1),s0.max(s1));
        let (x,y0,y1) = match &a.0.allotment.coord_system {
            CoordinateSystem::TrackingWindow | CoordinateSystem::Window => {
                let x = match (a.0.normal.is_sign_positive(),a.1.normal.is_sign_positive()) {
                    (true, true) =>   { 1 },
                    (false, false) => { 2 },
                    _ => { s0 = 0; s1 = 0; 0 }
                };
                (x,s0,s1)
            },
            CoordinateSystem::Content => { (0,0,0) },
            _ => { return None; }
        };
        Some((x..(x+1),(y0..(y1+1))))
    }
}
