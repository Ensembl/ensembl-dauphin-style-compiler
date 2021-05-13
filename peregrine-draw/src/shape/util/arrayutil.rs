use peregrine_data::{ ShipEnd, ScreenEdge };
use super::super::layers::layer::Layer;
use crate::{shape::layers::patina::PatinaProcessName, webgl::ProcessBuilder};
use crate::shape::layers::geometry::GeometryProgramName;
use crate::webgl::ProcessStanzaElements;
use crate::util::message::Message;
use web_sys::WebGlRenderingContext;
use crate::webgl::GPUSpec;

pub fn plain_rectangle(data: &mut Vec<f64>, left: f64, top: f64, right: f64, bottom: f64) {
    data.push(left);
    data.push(top);
    data.push(left);
    data.push(bottom);
    data.push(right);
    data.push(top);
    data.push(right);
    data.push(bottom);
}

pub fn hollow_rectangle(data: &mut Vec<f64>, left: f64, top: f64, right: f64, bottom: f64, w: f64) {
    data.push(left+w);
    data.push(top+w);
    data.push(left);
    data.push(top);

    data.push(left+w);
    data.push(bottom-w);
    data.push(left);
    data.push(bottom);

    data.push(right-w);
    data.push(bottom-w);
    data.push(right);
    data.push(bottom);    

    data.push(right-w);
    data.push(top+w);
    data.push(right);
    data.push(top);
}

pub fn rectangle(data: &mut Vec<f64>, left: f64, top: f64, right: f64, bottom: f64, w: Option<f64>){
    match w {
        Some(w) => hollow_rectangle(data,left,top,right,bottom,w),
        None => plain_rectangle(data,left,top,right,bottom)
    }
}

/* convert 0-255 colour indices to 0.0-1.0 */
pub(crate) fn scale_colour(value: u8) -> f64 {
    (value as f64)/255.
}

pub(crate) fn make_rect_elements(process: &mut ProcessBuilder, len: usize, hollow: bool) -> Result<ProcessStanzaElements,Message> {
    if hollow {
        Ok(process.get_stanza_builder().make_elements(len,&[0,1,2, 1,2,3, 2,3,4, 3,4,5, 4,5,6, 5,6,7, 6,7,0, 7,0,1])?)
    } else {
        Ok(process.get_stanza_builder().make_elements(len,&[0,3,1,2,0,3])?)
    }
}

/* interleaves pairs (eg interleaving x and y when drawing wiggles) */
pub(crate) fn interleave_pair(xx: &[f64], yy: &[f64]) -> Vec<f64> {
    let mut out = vec![];
    let mut yy_iter = if yy.len()!=0 { yy.iter() } else { [0.].iter() }.cycle();
    for x in xx {
        out.push(*x);
        out.push(*yy_iter.next().unwrap());
    }
    return out;
}

/* see geometry.txt section on one-anchor shapes for explanation */
pub(super) fn calculate_vertex(sea: &ScreenEdge, ship: &ShipEnd, size: &Vec<f64>, max: bool) -> Vec<f64> {
    let (sea,fp) = match sea {
        ScreenEdge::Min(x) => (x,-1.),
        ScreenEdge::Max(x) => (x, 1.)
    };
    let (ship,fsm) = match ship {
        ShipEnd::Min(x) => (x,0.),
        ShipEnd::Centre(x) => (x,-0.5),
        ShipEnd::Max(x) => (x,-1.)
    };
    let mut fsm = -fsm*fp;
    if max { fsm -= fp }; 
    let mut ship_iter = (if ship.len() > 0 { ship.iter() } else { [0.].iter() }).cycle();
    let mut size_iter = (if size.len() > 0 { size.iter() } else { [0.].iter() }).cycle();
    sea.iter().map(|x|
        x + fp * ship_iter.next().unwrap() + fsm * size_iter.next().unwrap()
    ).collect()
}

/* see geometry.txt section on one-anchor shapes for explanation */
pub(super) fn calculate_vertex_delta(count: usize, ship: &ShipEnd, size: &Vec<f64>, max: bool) -> Vec<f64> {
    let (ship,mut fsm) = match ship {
        ShipEnd::Min(x) => (x,0.),
        ShipEnd::Centre(x) => (x,-0.5),
        ShipEnd::Max(x) => (x,-1.)
    };
    if max { fsm += 1. }; 
    let mut ship_iter = (if ship.len() > 0 { ship.iter() } else { [0.].iter() }).cycle();
    let mut size_iter = (if size.len() > 0 { size.iter() } else { [0.].iter() }).cycle();
    (0..count).map(|_|
        - ship_iter.next().unwrap() + fsm * size_iter.next().unwrap()
    ).collect()
}

pub(super) fn sea_sign(sea: &ScreenEdge) -> f64 {
    match sea {
        ScreenEdge::Min(_) => 1.,
        ScreenEdge::Max(_) => -1.
    }
}

/* see geometry.txt section on one-anchor shapes for explanation */
pub(super) fn calculate_stretch_vertex(sea: &ScreenEdge, ship: &ShipEnd) -> Vec<f64> {
    let (sea,fp) = match sea {
        ScreenEdge::Min(x) => (x,-1.),
        ScreenEdge::Max(x) => (x, 1.)
    };
    let ship = match ship {
        ShipEnd::Min(x) => x,
        ShipEnd::Centre(x) => x,
        ShipEnd::Max(x) => x
    };
    let mut ship_iter = (if ship.len() > 0 { ship.iter() } else { [0.].iter() }).cycle();
    sea.iter().map(|x|
        x + fp * ship_iter.next().unwrap()
    ).collect()
}

/* see geometry.txt section on one-anchor shapes for explanation */
pub(super) fn calculate_stretch_vertex_delta(count: usize, ship: &ShipEnd) -> Vec<f64> {
    let ship = match ship {
        ShipEnd::Min(x) => x,
        ShipEnd::Centre(x) => x,
        ShipEnd::Max(x) => x
    };
    let mut ship_iter = (if ship.len() > 0 { ship.iter() } else { [0.].iter() }).cycle();
    (0..count).map(|_|
        - ship_iter.next().unwrap()
    ).collect()
}

pub(crate) fn apply_left(coord: &mut [f64], left: f64) {
    for x in coord.iter_mut() {
        *x -= left;
    }
}

pub(crate) fn empty_is<T>(value: Vec<T>, default: T) -> Vec<T> {
    if value.len() == 0 { vec![default] } else { value }
}
