use std::collections::HashMap;
use super::zmenu::ZMenu;
use crate::util::ringarray::{ UniformData, DataFilter };

pub(super) fn filter<F>(x: &[F], w: &[bool], primary: bool) -> Vec<F> where F: Clone {
    let mut out = vec![];
    if !primary && x.len() < 2 {
        return x.to_vec();
    }
    for (v,f) in x.iter().zip(w.iter().cycle()) {
        if *f { out.push(v.clone()); }
    }
    out
}

pub(super) fn bulk<T>(b: Vec<T>, a_len: usize, primary: bool) -> Vec<T> where T: Clone {
    if b.len() < a_len && (b.len() > 1 || primary) {
        let mut out = b.to_vec();
        let mut it = b.iter().cycle();
        out.extend((b.len()..a_len).map(|_| it.next().unwrap().clone()));
        out
    } else {
        b
    }
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct DirectColour(pub u8,pub u8,pub u8,pub u8);

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
pub struct Pen(pub String,pub u32,pub Vec<DirectColour>);

impl Pen {
    pub fn bulk(self, len: usize, primary: bool) -> Pen {
        Pen(self.0,self.1,bulk(self.2,len,primary))
    }

    pub fn filter(&self, filter: &DataFilter) -> Pen {
        Pen(self.0.clone(),self.1,filter.filter(&self.2))
    }
}

#[derive(Clone,Debug)]
pub struct Plotter(pub f64, pub DirectColour);

#[derive(Clone,Debug)]
pub enum Colour {
    Direct(DirectColour),
    Stripe(DirectColour,DirectColour,(u32,u32),f64),
    Bar(DirectColour,DirectColour,(u32,u32),f64)
}

#[derive(Clone,Debug)]
pub enum Patina {
    Filled(Vec<Colour>),
    Hollow(Vec<Colour>,u32),
    ZMenu(ZMenu,Vec<(String,Vec<String>)>)
}

fn filter_zmenu(h : &Vec<(String,Vec<String>)>, filter: &DataFilter) -> Vec<(String,Vec<String>)> {
    let mut out = Vec::with_capacity(h.len());
    for (k,v) in h {
        out.push((k.to_string(),filter.filter(&v)));
    }
    out
}

impl Patina {
    pub fn bulk(self, len: usize, primary: bool) -> Patina {
        match self {
            Patina::Filled(c) => Patina::Filled(bulk(c,len,primary)),
            Patina::Hollow(c,w) => Patina::Hollow(bulk(c,len,primary),w),
            Patina::ZMenu(z,mut h) => {
                let mut new_h  = h.clone();
                for (k,v) in h.drain(..) {
                    new_h.push((k,if v.len() > 1 { bulk(v,len,primary) } else { v }));
                }
                Patina::ZMenu(z,new_h)
            }
        }
    }

    pub fn filter(&self, filter: &DataFilter) -> Patina {
        match self {
            Patina::Filled(c) => Patina::Filled(filter.filter(c)),
            Patina::Hollow(c,w) => Patina::Hollow(filter.filter(c),*w),
            Patina::ZMenu(z,h) => Patina::ZMenu(z.clone(),filter_zmenu(h,filter))
        }
    }
}
