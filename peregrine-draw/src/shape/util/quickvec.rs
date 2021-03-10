use super::glaxis::GLAxis;

/* ugly but fast */

// TODO split this file
const HOLLOW_WIDTH : f64 = 1.; // XXX

fn vec1d_solid_x(x: &GLAxis) -> Vec<f64> {
    let mut out = vec![];
    for x in x.iter() {
        out.push(*x.0);
        out.push(*x.0);
        out.push(*x.1);
        out.push(*x.1);
    }
    return out;
}

fn vec1d_hollow_x(x: &GLAxis, w: f64) -> Vec<f64> {
    let mut out = vec![];
    for x in x.iter() {
        out.push(*x.0+w);
        out.push(*x.0);
        out.push(*x.0+w);
        out.push(*x.0);
        out.push(*x.1-w);
        out.push(*x.1);
        out.push(*x.1-w);
        out.push(*x.1);
    }
    out
}

pub(super) fn vec1d_x(x: &GLAxis, hollow: bool, origin: bool) -> Vec<f64> {
    if hollow {
        let width = if origin { 0. } else { HOLLOW_WIDTH };
        vec1d_hollow_x(x,width)
    } else {
        vec1d_solid_x(x)
    }
}

fn vec1d_solid_y(y: &GLAxis) -> Vec<f64> {
    let mut out = vec![];
    for y in y.iter() {
        out.push(*y.0);
        out.push(*y.1);
        out.push(*y.0);
        out.push(*y.1);
    }
    return out;
}

fn vec1d_hollow_y(y: &GLAxis, w: f64) -> Vec<f64> {
    let mut out = vec![];
    for y in y.iter() {
        out.push(*y.0+w);
        out.push(*y.0);
        out.push(*y.1-w);
        out.push(*y.1);
        out.push(*y.0+w);
        out.push(*y.0);
        out.push(*y.1-w);
        out.push(*y.1);
    }
    out
}

pub(super) fn vec1d_y(y: &GLAxis, hollow: bool, origin: bool) -> Vec<f64> {
    if hollow {
        let width = if origin { 0. } else { HOLLOW_WIDTH };
        vec1d_hollow_y(y,width)
    } else {
        vec1d_solid_y(y)
    }
}

fn vec2d_solid(x: &GLAxis, y: &GLAxis) -> Vec<f64> {
    let mut out = vec![];
    for (x,y) in x.iter().zip(y.iter()) {
        out.push(*x.0); out.push(*y.0);
        out.push(*x.0); out.push(*y.1);
        out.push(*x.1); out.push(*y.0);
        out.push(*x.1); out.push(*y.1);
    }
    return out;
}

fn vec2d_hollow(x: &GLAxis, y: &GLAxis, w: f64) -> Vec<f64> {
    let mut out = vec![];
    for (x,y) in x.iter().zip(y.iter()) {
        out.push(*x.0+w); out.push(*y.0+w);
        out.push(*x.0); out.push(*y.0);
        out.push(*x.0+w); out.push(*y.1-w);
        out.push(*x.0); out.push(*y.1);
        out.push(*x.1-w); out.push(*y.0+w);
        out.push(*x.1); out.push(*y.0);
        out.push(*x.1-w); out.push(*y.1-w);
        out.push(*x.1); out.push(*y.1);
    }
    out
}

pub(super) fn vec2d(x: &GLAxis, y: &GLAxis, hollow: bool, origin: bool) -> Vec<f64> {
    if hollow {
        let width = if origin { 0. } else { HOLLOW_WIDTH };
        vec2d_hollow(x,y,width)
    } else {
        vec2d_solid(x,y)
    }
}
