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
pub struct DirectColour(pub u8,pub u8,pub u8);

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
    Direct(Vec<DirectColour>),
    Spot(DirectColour)
}

impl Colour {
    pub fn bulk(self, len: usize, primary: bool) -> Colour {
        match self {
            Colour::Direct(d) => Colour::Direct(bulk(d,len,primary)),
            Colour::Spot(d) => Colour::Spot(d)
        }
    }

    pub fn filter(&self, filter: &DataFilter) -> Colour {
        match self {
            Colour::Direct(d) => Colour::Direct(filter.filter(d)),
            Colour::Spot(d) => Colour::Spot(d.clone())            
        }
    }
}

#[derive(Clone,Debug)]
pub enum Patina {
    Filled(Colour),
    Hollow(Colour),
    Stripe(Colour,Colour),
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
            Patina::Filled(c) => Patina::Filled(c.bulk(len,primary)),
            Patina::Hollow(c) => Patina::Hollow(c.bulk(len,primary)),
            Patina::Stripe(a,b) => Patina::Stripe(a.bulk(len,primary),b.bulk(len,primary)),
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
            Patina::Filled(c) => Patina::Filled(c.filter(filter)),
            Patina::Hollow(c) => Patina::Hollow(c.filter(filter)),
            Patina::Stripe(a,b) => Patina::Stripe(a.filter(filter),b.filter(filter)),
            Patina::ZMenu(z,h) => Patina::ZMenu(z.clone(),filter_zmenu(h,filter))
        }
    }
}
