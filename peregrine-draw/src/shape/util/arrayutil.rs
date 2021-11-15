use std::ops::{Add, Sub};

pub fn plain_rectangle4<T>(data: &mut Vec<T>, b_left: T, b_top: T, b_right: T, b_bottom: T,
                            d_left: T, d_top: T, d_right: T, d_bottom: T) where T: Copy {
    data.push(b_left);
    data.push(b_top);
    data.push(d_left);
    data.push(d_top);

    data.push(b_left);
    data.push(b_bottom);
    data.push(d_left);
    data.push(d_bottom);

    data.push(b_right);
    data.push(b_top);
    data.push(d_right);
    data.push(d_top);

    data.push(b_right);
    data.push(b_bottom);
    data.push(d_right);
    data.push(d_bottom);
}

fn hollow_rectangle4<T>(data: &mut Vec<T>, b_left: T, b_top: T, b_right: T, b_bottom: T, d_left: T, d_top: T, d_right: T, d_bottom: T, w: T) where T: Sub<Output=T> + Add<Output=T> + Copy {
    data.push(b_left);
    data.push(b_top);
    data.push(d_left+w);
    data.push(d_top+w);

    data.push(b_left);
    data.push(b_top);
    data.push(d_left);
    data.push(d_top);

    data.push(b_left);
    data.push(b_bottom);
    data.push(d_left+w);
    data.push(d_bottom+w);

    data.push(b_left);
    data.push(b_bottom);
    data.push(d_left);
    data.push(d_bottom);

    data.push(b_right);
    data.push(b_bottom);
    data.push(d_right+w);
    data.push(d_bottom+w);

    data.push(b_right);
    data.push(b_bottom);    
    data.push(d_right);
    data.push(d_bottom);    

    data.push(b_right);
    data.push(b_top);
    data.push(d_right+w);
    data.push(d_top+w);

    data.push(b_right);
    data.push(b_top);
    data.push(d_right);
    data.push(d_top);
}

pub fn rectangle4(data: &mut Vec<f32>, b_left: f64, b_top: f64, b_right: f64, b_bottom: f64, 
                                       d_left: f64, d_top: f64, d_right: f64, d_bottom: f64,
                    w: Option<f64>) {
    match w {
        Some(w) => hollow_rectangle4(data,b_left as f32,b_top as f32,b_right as f32,b_bottom as f32,
                                            d_left as f32,d_top as f32,d_right as f32,d_bottom as f32,w as f32),
        None => plain_rectangle4(data,b_left as f32,b_top as f32,b_right as f32,b_bottom as f32,
                                            d_left as f32,d_top as f32,d_right as f32,d_bottom as f32)
    }                    
}


/* convert 0-255 colour indices to 0.0-1.0 */
pub(crate) fn scale_colour(value: u8) -> f32 {
    (value as f32)/255.
}

/* interleaves pairs (eg interleaving x and y when drawing wiggles) */
pub(crate) fn interleave_pair(xx: &[f32], yy: &[f32]) -> Vec<f32> {
    let mut out = vec![];
    let mut yy_iter = if yy.len()!=0 { yy.iter() } else { [0.].iter() }.cycle();
    for x in xx {
        out.push(*x);
        out.push(*yy_iter.next().unwrap());
    }
    return out;
}

pub(crate) fn apply_left(coord: &mut [f64], left: f64) {
    for x in coord.iter_mut() {
        *x -= left;
    }
}
