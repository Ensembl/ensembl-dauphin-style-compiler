use peregrine_core::{ ShipEnd, ScreenEdge };
use super::super::layers::layer::Layer;
use crate::shape::layers::patina::PatinaProcessName;
use crate::shape::layers::geometry::GeometryProcessName;
use crate::webgl::ProcessStanzaElements;

// TODO split this file

/* convert 0-255 colour indices to 0.0-1.0 */
pub(crate) fn scale_colour(value: u8) -> f64 {
    (value as f64)/255.
}

/* n copies of an array */
pub(crate) fn repeat<T>(v: &[T], count: usize) -> Vec<T> where T: Clone {
    let mut out = vec![];
    for _ in 0..count {
        out.extend_from_slice(v);
    }
    out
}

/* four copies of each array entry. useful for seetings which are fixed across arectangle. */
pub(crate) fn quads<T>(vv: &[T], hollow: bool) -> Vec<T> where T: Copy {
    let mut out = vec![];
    for v in vv.iter() {
        out.push(*v);
        out.push(*v);
        out.push(*v);
        out.push(*v);
        if hollow {
            out.push(*v);
            out.push(*v);
            out.push(*v);
            out.push(*v);    
        }
    }
    out
}

/* interleave four arrays, representing co-ordinate pairs, into a sensible order for rendering rectangles:
 * (min,min) (min,max) (max,min) (max,max). This method also cycles around short arrays to make sure all
 * are as long as the first (x-minimum) coordinate.
 */
fn interleave_solid_rect_x(xx1: &[f64], yy1: &[f64], xx2: &[f64], yy2: &[f64]) -> Vec<f64> {
    let mut out = vec![];
    let mut xx2_iter = if xx2.len()!=0 { xx2.iter() } else { [0.].iter() }.cycle();
    let mut yy1_iter = if yy1.len()!=0 { yy1.iter() } else { [0.].iter() }.cycle();
    let mut yy2_iter = if yy2.len()!=0 { yy2.iter() } else { [0.].iter() }.cycle();
    for x1 in xx1 {
        let x2 = xx2_iter.next().unwrap();
        let y1 = yy1_iter.next().unwrap();
        let y2 = yy2_iter.next().unwrap();
        out.push(*x1); out.push(*y1);
        out.push(*x1); out.push(*y2);
        out.push(*x2); out.push(*y1);
        out.push(*x2); out.push(*y2);
    }
    return out;
}

/* interleave four arrays, representing co-ordinate pairs, into a sensible order for rendering hollow rectangles:
 *   (min,min)/in (min.min) (min,max)/in (min,max) (max,min)/in (max,min) (max,max)/in (max,max)
 * where /in denotes inset (ie mins increased by width, maxes decreased). 
 * This method also cycles around short arrays to make sure all are as long as the first (x-minimum) coordinate.
 */
fn interleave_hollow_rect_x(xx1: &[f64], yy1: &[f64], xx2: &[f64], yy2: &[f64], w: f64) -> Vec<f64> {
    let mut out = vec![];
    let mut xx2_iter = if xx2.len()!=0 { xx2.iter() } else { [0.].iter() }.cycle();
    let mut yy1_iter = if yy1.len()!=0 { yy1.iter() } else { [0.].iter() }.cycle();
    let mut yy2_iter = if yy2.len()!=0 { yy2.iter() } else { [0.].iter() }.cycle();
    for x1 in xx1 {
        let x2 = xx2_iter.next().unwrap();
        let y1 = yy1_iter.next().unwrap();
        let y2 = yy2_iter.next().unwrap();
        out.push(*x1+w); out.push(*y1+w);
        out.push(*x1); out.push(*y1);
        out.push(*x1+w); out.push(*y2-w);
        out.push(*x1); out.push(*y2);
        out.push(*x2-w); out.push(*y1+w);
        out.push(*x2); out.push(*y1);
        out.push(*x2-w); out.push(*y2-w);
        out.push(*x2); out.push(*y2);
    }
    return out;
}

pub(crate) fn interleave_rect_x(xx1: &[f64], yy1: &[f64], xx2: &[f64], yy2: &[f64], hollow: Option<f64>) -> Vec<f64> {
    if let Some(width) = hollow {
        interleave_hollow_rect_x(xx1,yy1,xx2,yy2,width)
    } else {
        interleave_solid_rect_x(xx1,yy1,xx2,yy2)
    }
}

pub(crate) fn make_rect_elements(layer: &mut Layer, geometry: &GeometryProcessName, patina: &PatinaProcessName, len: usize, hollow: bool) -> anyhow::Result<ProcessStanzaElements> {
    if hollow {
        layer.make_elements(geometry,patina,len,&[0,1,2, 1,2,3, 2,3,4, 3,4,5, 4,5,6, 5,6,7, 6,7,0, 7,0,1])
    } else {
        layer.make_elements(geometry,patina,len,&[0,3,1,2,1,3])   
    }
}


/* interleave four arrays, representing co-ordinate pairs, into a sensible order for rendering rectangles:
 * (min,min) (min,max) (max,min) (max,max). This method also cycles around short arrays to make sure all
 * are as long as the second (y-minimum) coordinate.
 */
fn interleave_solid_rect_y(xx1: &[f64], yy1: &[f64], xx2: &[f64], yy2: &[f64]) -> Vec<f64> {
    let mut out = vec![];
    let mut yy2_iter = if yy2.len()!=0 { yy2.iter() } else { [0.].iter() }.cycle();
    let mut xx1_iter = if xx1.len()!=0 { xx1.iter() } else { [0.].iter() }.cycle();
    let mut xx2_iter = if xx2.len()!=0 { xx2.iter() } else { [0.].iter() }.cycle();
    for y1 in yy1 {
        let y2 = yy2_iter.next().unwrap();
        let x1 = xx1_iter.next().unwrap();
        let x2 = xx2_iter.next().unwrap();
        out.push(*x1); out.push(*y1);
        out.push(*x1); out.push(*y2);
        out.push(*x2); out.push(*y1);
        out.push(*x2); out.push(*y2);
    }
    return out;
}

fn interleave_hollow_rect_y(xx1: &[f64], yy1: &[f64], xx2: &[f64], yy2: &[f64], w: f64) -> Vec<f64> {
    let mut out = vec![];
    let mut yy2_iter = if yy2.len()!=0 { yy2.iter() } else { [0.].iter() }.cycle();
    let mut xx1_iter = if xx1.len()!=0 { xx1.iter() } else { [0.].iter() }.cycle();
    let mut xx2_iter = if xx2.len()!=0 { xx2.iter() } else { [0.].iter() }.cycle();
    for y1 in yy1 {
        let y2 = yy2_iter.next().unwrap();
        let x1 = xx1_iter.next().unwrap();
        let x2 = xx2_iter.next().unwrap();
        out.push(*x1+w); out.push(*y1+w);
        out.push(*x1); out.push(*y1);
        out.push(*x1+w); out.push(*y2-w);
        out.push(*x1); out.push(*y2);
        out.push(*x2-w); out.push(*y1+w);
        out.push(*x2); out.push(*y1);
        out.push(*x2-w); out.push(*y2-w);
        out.push(*x2); out.push(*y2);
    }
    return out;
}

pub(crate) fn interleave_rect_y(xx1: &[f64], yy1: &[f64], xx2: &[f64], yy2: &[f64], hollow: Option<f64>) -> Vec<f64> {
    if let Some(width) = hollow {
        interleave_hollow_rect_y(xx1,yy1,xx2,yy2,width)
    } else {
        interleave_solid_rect_y(xx1,yy1,xx2,yy2)
    }
}

/* interleaves two arrays representing (min,max) pairs for a single x-axis property when applied to a
 * rectangle-rendering set of variables in the order (min,min) (min,max) (max,min) (max,max) (the
 * ordering used by other interlevaing methods): ie it interleaves in the order min, min, max, max.
 */
pub(crate) fn interleave_line_x(xx1: &[f64], xx2: &[f64], hollow: bool) -> Vec<f64> {
    let mut out = vec![];
    let mut xx2_iter = if xx2.len()!=0 { xx2.iter() } else { [0.].iter() }.cycle();
    for x1 in xx1 {
        let x2 = xx2_iter.next().unwrap();
        out.push(*x1);
        out.push(*x1);
        if hollow {
            out.push(*x1);
            out.push(*x1);    
        }
        out.push(*x2);
        out.push(*x2);
        if hollow {
            out.push(*x2);
            out.push(*x2);    
        }
    }
    return out;
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

/* Interleaves count copies of a then b */
pub(crate) fn interleave_pair_count<X>(a: X, b: X, count: usize) -> anyhow::Result<Vec<X>> where X: Clone {
    let mut out = vec![];
    for _ in 0..count {
        out.push(a.clone());
        out.push(b.clone());
    }
    Ok(out)
}

/* see geometry.txt section on one-anchor shapes for explanation */
pub(crate) fn calculate_vertex_min(sea: &[f64], ship: &ShipEnd, size: &Vec<f64>, max: bool) -> Vec<f64> {
    let (ship,mut fsm) = match ship {
        ShipEnd::Min(x) => (x,0.),
        ShipEnd::Centre(x) => (x,-0.5),
        ShipEnd::Max(x) => (x,-1.)
    };
    if max { fsm += 1.; }; 
    let mut ship_iter = (if ship.len() > 0 { ship.iter() } else { [0.].iter() }).cycle();
    let mut size_iter = (if size.len() > 0 { size.iter() } else { [0.].iter() }).cycle();
    sea.iter().map(|x|
        x - ship_iter.next().unwrap() + fsm * size_iter.next().unwrap()
    ).collect()
}

/* see geometry.txt section on one-anchor shapes for explanation */
pub(crate) fn calculate_vertex(sea: &ScreenEdge, ship: &ShipEnd, size: &Vec<f64>, max: bool) -> Vec<f64> {
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
pub(crate) fn calculate_vertex_delta(count: usize, ship: &ShipEnd, size: &Vec<f64>, max: bool) -> Vec<f64> {
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

pub(crate) fn sea_sign(sea: &ScreenEdge) -> f64 {
    match sea {
        ScreenEdge::Min(_) => 1.,
        ScreenEdge::Max(_) => -1.
    }
}

/* see geometry.txt section on one-anchor shapes for explanation */
pub(crate) fn calculate_stretch_vertex(sea: &ScreenEdge, ship: &ShipEnd) -> Vec<f64> {
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
pub(crate) fn calculate_stretch_vertex_delta(count: usize, ship: &ShipEnd) -> Vec<f64> {
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

pub(crate) fn apply_left(coord: &mut [f64], layer: &Layer) {
    let left = layer.left();
    for x in coord.iter_mut() {
        *x -= left;
    }
}

pub(crate) fn empty_is<T>(value: Vec<T>, default: T) -> Vec<T> {
    if value.len() == 0 { vec![default] } else { value }
}
