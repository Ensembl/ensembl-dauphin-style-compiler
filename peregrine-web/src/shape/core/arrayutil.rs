use peregrine_core::{ ShipEnd, ScreenEdge };

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
pub(crate) fn quads<T>(vv: &[T]) -> Vec<T> where T: Copy {
    let mut out = vec![];
    for v in vv.iter() {
        out.push(*v);
        out.push(*v);
        out.push(*v);
        out.push(*v);
    }
    out
}

/* interleave four arrays, representing co-ordinate pairs, into a sensible order for rendering triangles:
 * (min,min) (min,max) (max,min) (max,max). This method also cycles around short arrays to make sure all
 * are as long as the first (x-minimum) coordinate.
 */
pub(crate) fn interleave_rect_x(xx1: &[f64], yy1: &[f64], xx2: &[f64], yy2: &[f64]) -> Vec<f64> {
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

/* interleave four arrays, representing co-ordinate pairs, into a sensible order for rendering rectangles:
 * (min,min) (min,max) (max,min) (max,max). This method also cycles around short arrays to make sure all
 * are as long as the second (y-minimum) coordinate.
 */
pub(crate) fn interleave_rect_y(xx1: &[f64], yy1: &[f64], xx2: &[f64], yy2: &[f64]) -> Vec<f64> {
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

/* interleaves two arrays representing (min,max) pairs for a single x-axis property when applied to a
 * rectangle-rendering set of variables in the order (min,min) (min,max) (max,min) (max,max) (the
 * ordering used by other interlevaing methods): ie it interleaves in the order min, min, max, max.
 */
pub(crate) fn interleave_line_x(xx1: &[f64], xx2: &[f64]) -> Vec<f64> {
    let mut out = vec![];
    let mut xx2_iter = if xx2.len()!=0 { xx2.iter() } else { [0.].iter() }.cycle();
    for x1 in xx1 {
        let x2 = xx2_iter.next().unwrap();
        out.push(*x1);
        out.push(*x1);
        out.push(*x2);
        out.push(*x2);
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
