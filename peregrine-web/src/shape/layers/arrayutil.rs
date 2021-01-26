use peregrine_core::{ ShipEnd, ScreenEdge };
use anyhow::{ bail };

pub(super) fn scale_colour(value: u8) -> f64 {
    (value as f64)/255.
}

pub(super) fn interleave<X>(mut main: Vec<X>, sub: &[X]) -> anyhow::Result<Vec<X>> where X: Clone {
    let sub_len = sub.len();
    if sub_len == 0 { bail!("Cannot interleave zero-length array"); }
    let mut out = vec![];
    for (i,a) in main.drain(..).enumerate() {
        out.push(a);
        out.push(sub[i%sub_len].clone());
    }
    Ok(out)
}

pub(super) fn interleave_one<X>(a: X, b: X, count: usize) -> anyhow::Result<Vec<X>> where X: Clone {
    let mut out = vec![];
    for _ in 0..count {
        out.push(a.clone());
        out.push(b.clone());
    }
    Ok(out)
}

pub(super) fn ship_box(ship_x: ShipEnd, size_x: Vec<f64>, ship_y: ShipEnd, size_y: Vec<f64>, count: usize) -> Vec<f64> {
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

pub(super) fn add_fixed_sea_box(values: &mut [f64], y: bool, screen: ScreenEdge) {
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
