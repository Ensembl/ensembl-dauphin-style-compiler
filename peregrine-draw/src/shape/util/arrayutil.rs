use std::ops::{Add, Sub};

use crate::{ webgl::ProcessBuilder};
use crate::webgl::ProcessStanzaElements;
use crate::util::message::Message;

pub fn plain_rectangle<T>(data: &mut Vec<T>, left: T, top: T, right: T, bottom: T) where T: Copy {
    data.push(left);
    data.push(top);
    data.push(left);
    data.push(bottom);
    data.push(right);
    data.push(top);
    data.push(right);
    data.push(bottom);
}

pub fn hollow_rectangle<T>(data: &mut Vec<T>, left: T, top: T, right: T, bottom: T, w: T) where T: Sub<Output=T> + Add<Output=T> + Copy {
    data.push(left+w);
    data.push(top+w);
    data.push(left);
    data.push(top);

    data.push(left+w);
    data.push(bottom+w);
    data.push(left);
    data.push(bottom);

    data.push(right+w);
    data.push(bottom+w);
    data.push(right);
    data.push(bottom);    

    data.push(right+w);
    data.push(top+w);
    data.push(right);
    data.push(top);
}

pub fn rectangle<T>(data: &mut Vec<T>, left: T, top: T, right: T, bottom: T, w: Option<T>) where T: Sub<Output=T> + Add<Output=T> + Copy {
    match w {
        Some(w) => hollow_rectangle(data,left,top,right,bottom,w),
        None => plain_rectangle(data,left,top,right,bottom)
    }
}

pub fn rectangle64(data: &mut Vec<f32>, left: f64, top: f64, right: f64, bottom: f64, w: Option<f64>) {
    rectangle(data,left as f32,top as f32,right as f32,bottom as f32,w.map(|x| x as f32))
}

/* convert 0-255 colour indices to 0.0-1.0 */
pub(crate) fn scale_colour(value: u8) -> f32 {
    (value as f32)/255.
}

pub(crate) fn make_rect_elements(process: &mut ProcessBuilder, len: usize, hollow: bool) -> Result<ProcessStanzaElements,Message> {
    if hollow {
        Ok(process.get_stanza_builder().make_elements(len,&[0,1,2, 1,2,3, 2,3,4, 3,4,5, 4,5,6, 5,6,7, 6,7,0, 7,0,1])?)
    } else {
        Ok(process.get_stanza_builder().make_elements(len,&[0,3,1,2,0,3])?)
    }
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

pub(crate) fn empty_is<T>(value: Vec<T>, default: T) -> Vec<T> {
    if value.len() == 0 { vec![default] } else { value }
}
