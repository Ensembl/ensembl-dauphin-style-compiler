use peregrine_core::{ ShipEnd, ScreenEdge };
use anyhow::{ bail };

pub(crate) fn scale_colour(value: u8) -> f64 {
    (value as f64)/255.
}

pub(crate) fn repeat(v: &[f64], count: usize) -> Vec<f64> {
    let mut out = vec![];
    for _ in 0..count {
        out.extend_from_slice(v);
    }
    out
}

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

fn flip_sense(values: &mut [f64], edge: &ScreenEdge, max: bool) {
    if match edge {
        ScreenEdge::Max(_) => { max },
        ScreenEdge::Min(_) => { !max }
    } {
        for v in values {
            *v *= -1.;
        }      
    }
}

fn add_extending(values: &mut [f64], delta: &[f64]) -> anyhow::Result<()> {
    let mut delta_iter = if delta.len()!=0 { delta.iter() } else { [0.].iter() }.cycle();
    for v in values {
        *v += delta_iter.next().unwrap();
    }
    Ok(())
}

pub(crate) fn stretchtangle(a: ScreenEdge, p: ShipEnd,  max: bool) -> anyhow::Result<(Vec<f64>,f64)> {
    let mut p = match p { ShipEnd::Min(x) => x, ShipEnd::Centre(x) => x, ShipEnd::Max(x) => x };
    flip_sense(&mut p,&a,max);
    let s = match a { ScreenEdge::Min(_) => 1., ScreenEdge::Max(_) => -1. };
    let mut a = match a { ScreenEdge::Min(x) => x, ScreenEdge::Max(x) => x };
    add_extending(&mut a,&p)?;
    Ok((a,s))
}

pub(crate) fn interleave<X>(mut main: Vec<X>, sub: &[X]) -> anyhow::Result<Vec<X>> where X: Clone {
    let sub_len = sub.len();
    if sub_len == 0 { bail!("Cannot interleave zero-length array"); }
    let mut out = vec![];
    for (i,a) in main.drain(..).enumerate() {
        out.push(a);
        out.push(sub[i%sub_len].clone());
    }
    Ok(out)
}

pub(crate) fn interleave_one<X>(a: X, b: X, count: usize) -> anyhow::Result<Vec<X>> where X: Clone {
    let mut out = vec![];
    for _ in 0..count {
        out.push(a.clone());
        out.push(b.clone());
    }
    Ok(out)
}

pub(crate) fn ship_box(ship_x: ShipEnd, size_x: Vec<f64>, ship_y: ShipEnd, size_y: Vec<f64>, count: usize) -> Vec<f64> {
    let mut out = vec![];
    /* the weird procedures in this method are to keep branch-free inner loops and to avoid copying */
    /* order is (min,min) (min,max) (max,max) (max,min) */
    let x = match &ship_x {
        ShipEnd::Min(z) => z,
        ShipEnd::Centre(z) => z,
        ShipEnd::Max(z) => z
    };
    let y = match &ship_y {
        ShipEnd::Min(z) => z,
        ShipEnd::Centre(z) => z,
        ShipEnd::Max(z) => z
    };
    let x_len = x.len();
    let y_len = y.len();    
    let x_factor = match ship_x {
        ShipEnd::Min(_) => 0.,
        ShipEnd::Centre(_) => 0.5,
        ShipEnd::Max(_) => 1.
    };
    let y_factor = match ship_y {
        ShipEnd::Min(_) => 0.,
        ShipEnd::Centre(_) => 0.5,
        ShipEnd::Max(_) => 1.
    };
    let size_x_len = size_x.len();
    let size_y_len = size_y.len();
    for i in 0..count {
        let x_size = size_x[i%size_x_len];
        let y_size = size_y[i%size_y_len];
        let x_shift = x_size * x_factor;
        let y_shift = y_size * y_factor;
        /* (min,min) */
        out.push(-x[i%x_len]-x_shift);
        out.push(-y[i%y_len]-y_shift);
        /* (min,max) */
        out.push(-x[i%x_len]-x_shift);
        out.push(-y[i%y_len]-y_shift+y_size);
        /* (max,max) */
        out.push(-x[i%x_len]-x_shift+x_size);
        out.push(-y[i%y_len]-y_shift+y_size);
        /* (max,min) */
        out.push(-x[i%x_len]-x_shift+x_size);
        out.push(-y[i%y_len]-y_shift);
    }
    out
}

pub(crate) fn add_fixed_sea_box(values: &mut [f64], y: bool, screen: ScreenEdge) {
    let nudge = if y { 1 } else { 0 };
    match &screen {
        ScreenEdge::Min(z) => {
            for i in 0..values.len()/2 {
                values[i*2+nudge] += z[i];
            }
        },
        ScreenEdge::Max(z) => {
            for i in 0..values.len()/2 {
                values[i*2+nudge] = z[i] - values[i*2+nudge];
            }
        }
    }
}
