use peregrine_data::{SingleHotspotEntry, CoordinateSystem, SpaceBasePointRef, AuxLeaf};
use peregrine_toolkit::{hotspots::hotspotstore::{HotspotStoreProfile, HotspotPosition}};
use crate::stage::axis::UnitConverter;
use super::{drawhotspotstore::PointPair, coordconverter::CoordToPxConverter};

const STRIPE_SIZE : f64 = 50.;

pub(crate) struct WindowHotspotContext {
    pub(crate) converter: UnitConverter,
    pub(crate) x_px: f64, /* width of viewport (px) */
    pub(crate) y_px: f64, /* height of viewport (px) */
    pub(crate) y_offset: f64 /* scroll down */
}

/* (0,0) whole screen, always check
 * (1,y) positive y
 * (2,y) negative y
 */

fn order(a: f64, b: f64) -> (f64,f64) { (a.min(b),a.max(b)) }

fn round(y: f64) -> usize { (y.abs()/STRIPE_SIZE).floor() as usize }

fn wrap(coord: f64, size: f64) -> f64 { if coord < 0. { coord+size+1. } else { coord } }

fn y_intersect(context: &WindowHotspotContext, c1: &SpaceBasePointRef<f64,AuxLeaf>, c2: &SpaceBasePointRef<f64,AuxLeaf>) -> Option<(f64,f64)> {
    match c1.allotment.coord_system {
        CoordinateSystem::TrackingWindow | CoordinateSystem::Window => {
            let y1 = wrap(*c1.normal,context.y_px);
            let y2 = wrap(*c2.normal,context.y_px);
            let (y1,y2) = order(y1,y2);
            Some((y1.min(y2)+context.y_offset,y1.max(y2)+context.y_offset))        
        },
        CoordinateSystem::Content => {
            let y1 = wrap(*c1.normal,context.y_px);
            let y2 = wrap(*c2.normal,context.y_px);
            let (y1,y2) = order(y1,y2);
            Some((y1.min(y2),y1.max(y2)))
        },
        CoordinateSystem::SidewaysLeft | CoordinateSystem::SidewaysRight => {
            Some((
                (context.y_px * c1.base) + c1.tangent,
                (context.y_px * c2.base) + c2.tangent
            ))
        },
        _ => { return None; }
    }
}

fn x_intersect(context: &WindowHotspotContext, coord_to_px: &CoordToPxConverter, c1: &SpaceBasePointRef<f64,AuxLeaf>, c2: &SpaceBasePointRef<f64,AuxLeaf>) -> Option<(f64,f64)> {
    let (px1,px2) = match c1.allotment.coord_system {
        CoordinateSystem::TrackingWindow => {
            (
                coord_to_px.tracking_coord_to_px(&c1),
                coord_to_px.tracking_coord_to_px(&c2)
            )
        },
        CoordinateSystem::Window | CoordinateSystem::Content => {
            (
                (context.x_px * c1.base) + c1.tangent,
                (context.x_px * c2.base) + c2.tangent
            )
        },
        CoordinateSystem::SidewaysLeft => {
            let x1 = wrap(*c1.normal,context.x_px);
            let x2 = wrap(*c2.normal,context.x_px);
            let (x1,x2) = order(x1,x2);
            (x1,x2)
        },
        CoordinateSystem::SidewaysRight => {
            let x1 = wrap(-1.-*c1.normal,context.x_px);
            let x2 = wrap(-1.-*c2.normal,context.x_px);
            let (x1,x2) = order(x1,x2);
            (x1,x2)
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
    type Context = WindowHotspotContext;

    fn diagonalise(&self, x: usize, y: usize) -> usize { 
        if x == 0 { 0 } else if x == 1 { 2*y+1 } else { 2*y+2 }
    }

    fn get_zones(&self, context: &WindowHotspotContext, coords: &(f64,f64)) -> Vec<(usize,usize)> {
        vec![
            (0,0),
            (1,round(coords.1)),
            (2,round(context.y_px-coords.1))
        ]
    }

    fn bounds(&self, context: &WindowHotspotContext, value: &SingleHotspotEntry) -> Option<HotspotPosition> {
        let coord_to_px = self.converter(&context.converter)?;
        let (at_coords,_) = value.coordinates();
        at_coords.map(|(c1,c2)| {
            let (left,right) = x_intersect(&context,&coord_to_px,&c1,&c2)?;
            let (top,bottom) = y_intersect(&context,&c1,&c2)?;
            Some(HotspotPosition { top, bottom, left, right })
        }).unwrap_or(None)
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
            CoordinateSystem::Content | CoordinateSystem::SidewaysLeft | 
                CoordinateSystem::SidewaysRight => { (0,0,0) },
            CoordinateSystem::Tracking | CoordinateSystem::TrackingSpecial |
                CoordinateSystem::Dustbin => { return None; }
        };
        Some((x..(x+1),(y0..(y1+1))))
    }
}
