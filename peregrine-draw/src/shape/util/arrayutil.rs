use std::ops::{Add, Sub};

const PLAIN4 : &[usize] = &[
    0, 1, 4, 5,
    0, 3, 4, 7,
    2, 1, 6, 5,
    2, 3, 6, 7 
];

const HOLLOW4 : &[usize] = &[
     8, 9, 4, 5,
     0, 1, 4, 5,
     8,11, 4, 7,
     0, 3, 4, 7,
    10,11, 6, 7,
     2, 3, 6, 7,
    10, 9, 6, 5,
     2, 1, 6, 5
];


fn broadcast(data: &mut Vec<f32>, indexes: &[usize], values: &[f64]) {
    for i in indexes {
        data.push(values[*i] as f32);
    }
}

pub fn plain_rectangle4(data: &mut Vec<f32>, b_left: f64, b_top: f64, b_right: f64, b_bottom: f64,
                            d_left: f64, d_top: f64, d_right: f64, d_bottom: f64) {
    broadcast(data,PLAIN4,&[
        b_left,b_top,b_right,b_bottom,
        d_left,d_top,d_right,d_bottom
    ]);
}

pub fn hollow_rectangle4(data: &mut Vec<f32>, b_left: f64, b_top: f64, b_right: f64, b_bottom: f64, d_left: f64, d_top: f64, d_right: f64, d_bottom: f64, w: f64) {
    broadcast(data,HOLLOW4,&[
        b_left,b_top,b_right,b_bottom,
        d_left,d_top,d_right,d_bottom,
        b_left+w,b_top+w,b_right+w,b_bottom+w
    ]);
}

pub fn rectangle4(data: &mut Vec<f32>, b_left: f64, b_top: f64, b_right: f64, b_bottom: f64, 
                                       d_left: f64, d_top: f64, d_right: f64, d_bottom: f64,
                    w: Option<f64>) {
    match w {
        Some(w) => hollow_rectangle4(data,b_left,b_top,b_right,b_bottom,
                                            d_left,d_top,d_right,d_bottom,w),
        None => plain_rectangle4(data,b_left,b_top,b_right,b_bottom,
                                            d_left,d_top,d_right,d_bottom)
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
