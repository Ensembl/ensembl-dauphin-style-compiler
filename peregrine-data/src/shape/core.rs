use std::{collections::{HashMap, hash_map::DefaultHasher}, hash::{Hash, Hasher}, sync::Arc};
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
struct PenGeometry {
    name: String,
    size: u32,
    hash: u64
}

impl PenGeometry {
    fn new(name: &str, size:u32) -> PenGeometry {
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        size.hash(&mut hasher);
        let hash = hasher.finish();
        PenGeometry {
            name: name.to_string(),
            size,
            hash
        }
    }
}

#[derive(Clone,Debug,PartialEq,Eq,Hash)]
struct PenInner {
    geometry: Arc<PenGeometry>,
    colours: Vec<DirectColour>,
    background: Option<DirectColour>
}

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct Pen {
    inner: Arc<PenInner>,
    hash: u64
}

impl Hash for Pen {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl Pen {
    fn new_real(geometry: &Arc<PenGeometry>, colours: &[DirectColour], background: &Option<DirectColour>) -> Pen {
        let inner = PenInner {
            geometry: geometry.clone(),
            colours: colours.to_vec(),
            background: background.clone()
        };
        let mut h = DefaultHasher::new();
        inner.hash(&mut h);
        Pen {
            inner: Arc::new(inner),
            hash: h.finish()
        }
    }

    pub fn new(name: &str, size: u32, colours: &[DirectColour], background: &Option<DirectColour>) -> Pen {
        Pen::new_real(&Arc::new(PenGeometry::new(name,size)), colours,background)
    }

    pub fn name(&self) -> &str { &self.inner.geometry.name }
    pub fn size(&self) -> u32 { self.inner.geometry.size }
    pub fn colours(&self) -> &[DirectColour] { &self.inner.colours }
    pub fn background(&self) -> &Option<DirectColour> { &self.inner.background }

    pub fn bulk(self, len: usize, primary: bool) -> Pen {
        Pen::new_real(&self.inner.geometry,&bulk(self.inner.colours.to_vec(),len,primary),&self.inner.background)
    }

    pub fn filter(&self, filter: &DataFilter) -> Pen {
        Pen::new_real(&self.inner.geometry,&filter.filter(&self.inner.colours),&self.inner.background)
    }

    pub fn group_hash(&self) -> u64 { self.inner.geometry.hash }
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub struct Plotter(pub f64, pub DirectColour);

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
pub enum Colour {
    Direct(DirectColour),
    Stripe(DirectColour,DirectColour,(u32,u32),f64),
    Bar(DirectColour,DirectColour,(u32,u32),f64)
}

#[derive(Clone)]
#[cfg_attr(debug_assertions,derive(Debug))]
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
