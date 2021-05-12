use std::collections::HashMap;
use super::zmenu::ZMenu;

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

    pub fn filter(&self, which: &[bool], primary: bool) -> Pen {
        Pen(self.0.clone(),self.1.clone(),filter(&self.2,which,primary))
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

    fn filter(&self, which: &[bool], primary: bool) -> Colour {
        match self {
            Colour::Direct(d) => Colour::Direct(filter(&d,which,primary)),
            Colour::Spot(d) => Colour::Spot(d.clone())
        }
    }
}

#[derive(Clone,Debug)]
pub enum Patina {
    Filled(Colour),
    Hollow(Colour),
    ZMenu(ZMenu,HashMap<String,Vec<String>>)
}

impl Patina {
    pub fn bulk(self, len: usize, primary: bool) -> Patina {
        match self {
            Patina::Filled(c) => Patina::Filled(c.bulk(len,primary)),
            Patina::Hollow(c) => Patina::Hollow(c.bulk(len,primary)),
            Patina::ZMenu(z,mut h) => {
                let mut new_h  : HashMap<String,Vec<String>> = h.clone();
                for (k,v) in h.drain() {
                    new_h.insert(k,if v.len() > 1 { bulk(v,len,primary) } else { v });
                }
                Patina::ZMenu(z,new_h)
            }
        }
    }

    pub fn filter(&self, which: &[bool], primary: bool) -> Patina {
        match self {
            Patina::Filled(c) => Patina::Filled(c.filter(which,primary)),
            Patina::Hollow(c) => Patina::Hollow(c.filter(which,primary)),
            Patina::ZMenu(z,h) => Patina::ZMenu(z.clone(),h.iter().map(|(k,v)| (k.to_string(),filter(&v,which,primary))).collect())
        }
    }
}


#[derive(Clone,Debug)]
pub enum ScreenEdge { Min(Vec<f64>), Max(Vec<f64>) }

impl ScreenEdge {
    pub fn bulk(self, len: usize, primary: bool) -> ScreenEdge {
        match self {
            ScreenEdge::Min(x) => ScreenEdge::Min(bulk(x,len,primary)),
            ScreenEdge::Max(x) => ScreenEdge::Max(bulk(x,len,primary))
        }
    }

    pub fn len(&self) -> usize {
        match self {
            ScreenEdge::Min(x) => x.len(),
            ScreenEdge::Max(x) => x.len()
        }
    }


    pub fn transform<'t,F>(&'t self,cb: F) -> ScreenEdge where F: FnMut(&f64) -> f64 + 't {
        match self {
            ScreenEdge::Min(x) => ScreenEdge::Min(x.iter().map(cb).collect()),
            ScreenEdge::Max(x) => ScreenEdge::Max(x.iter().map(cb).collect())
        }
    }
}

#[derive(Clone,Debug)]
pub enum ShipEnd { Min(Vec<f64>), Centre(Vec<f64>), Max(Vec<f64>) }

impl ShipEnd {
    pub fn bulk(self, len: usize, primary: bool) -> ShipEnd {
        match self {
            ShipEnd::Min(x) => ShipEnd::Min(bulk(x,len,primary)),
            ShipEnd::Centre(x) => ShipEnd::Centre(bulk(x,len,primary)),
            ShipEnd::Max(x) => ShipEnd::Max(bulk(x,len,primary))
        }
    }

    fn filter(&self, which: &[bool], primary: bool) -> ShipEnd {
        match self {
            ShipEnd::Min(x) => ShipEnd::Min(filter(x,which,primary)),
            ShipEnd::Centre(x) => ShipEnd::Centre(filter(x,which,primary)),
            ShipEnd::Max(x) => ShipEnd::Max(filter(x,which,primary))
        }
    }
}

#[derive(Clone,Debug)]
pub enum SeaEnd {
    Paper(Vec<f64>),
    Screen(ScreenEdge)
}

impl SeaEnd {
    pub fn bulk(self, len: usize, primary: bool) -> SeaEnd {
        match self {
            SeaEnd::Paper(x) => SeaEnd::Paper(bulk(x,len,primary)),
            SeaEnd::Screen(x) => SeaEnd::Screen(x.bulk(len,primary))
        }
    }

    pub fn len(&self) -> usize {
        match self {
            SeaEnd::Paper(x) => x.len(),
            SeaEnd::Screen(x) => x.len()
        }
    }

    fn matches(&self, min_value: f64, max_value: f64) -> Vec<bool> {
        match &self {
            SeaEnd::Paper(v) => {
                v.iter().map(|x| *x >= min_value && *x <= max_value).collect()
            },
            SeaEnd::Screen(ScreenEdge::Min(s)) => s.iter().map(|_| true).collect(),
            SeaEnd::Screen(ScreenEdge::Max(s)) => s.iter().map(|_| true).collect(),
        }
    }

    fn filter(&self, which: &[bool], primary: bool) -> SeaEnd {
        match self {
            SeaEnd::Paper(v) => SeaEnd::Paper(filter(v,which, primary)),
            x => x.clone()
        }
    }
}

#[derive(Clone,Debug)]
pub enum SeaEndPair {
    Paper(Vec<f64>,Vec<f64>),
    Screen(ScreenEdge,ScreenEdge)
}

impl SeaEndPair {
    pub fn bulk(self, len: usize, primary: bool) -> SeaEndPair {
        match self {
            SeaEndPair::Paper(a,b) => SeaEndPair::Paper(bulk(a,len,primary),bulk(b,len,false)),
            SeaEndPair::Screen(a,b) => SeaEndPair::Screen(a.bulk(len,primary),b.bulk(len,false))
        }
    }
    
    fn len(&self) -> usize {
        match self {
            SeaEndPair::Paper(a,_) => a.len(),
            SeaEndPair::Screen(a,_) => a.len()
        }
    }

    fn matches(&self, min_value: f64, max_value: f64) -> Vec<bool> {
        match &self {
            SeaEndPair::Paper(a,b) => {
                a.iter().zip(b.iter().cycle()).map(|(x,y)| {
                    (*x >= min_value && *x <= max_value) ||
                    (*y >= min_value && *y <= max_value)
                }).collect()
            },
            SeaEndPair::Screen(ScreenEdge::Min(s),_) =>  s.iter().map(|_| true).collect(),
            SeaEndPair::Screen(ScreenEdge::Max(s),_) =>  s.iter().map(|_| true).collect(),
        }
    }

    fn filter(&self, which: &[bool], primary: bool) -> SeaEndPair {
        match self {
            SeaEndPair::Paper(a,b) => {
                SeaEndPair::Paper(filter(a,which,primary),filter(&b,which,false))
            },
            x => x.clone()
        }
    }
}

#[derive(Clone,Debug)]
pub struct SingleAnchorAxis(pub SeaEnd,pub ShipEnd);

impl SingleAnchorAxis {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn bulk(self, len: usize, primary: bool) -> SingleAnchorAxis {
        SingleAnchorAxis(self.0.bulk(len,primary),self.1.bulk(len,false))
    }

    fn matches(&self, min_value: f64, max_value: f64) -> Vec<bool> {
        self.0.matches(min_value,max_value)
    }

    fn filter(&self, which: &[bool], primary: bool) -> SingleAnchorAxis {
        let (sea,ship) = (&self.0,&self.1);
        SingleAnchorAxis(sea.filter(which,primary),ship.filter(which,false))
    }
}

#[derive(Clone,Debug)]
pub struct AnchorPairAxis(pub SeaEndPair,pub ShipEnd,pub ShipEnd);

impl AnchorPairAxis {  
    pub fn len(&self) -> usize {
        self.0.len()
    }
    
    pub fn bulk(self, len: usize, primary: bool) -> AnchorPairAxis {
        AnchorPairAxis(self.0.bulk(len,primary),self.1.bulk(len,false),self.2.bulk(len,false))
    }

    fn matches(&self, min_value: f64, max_value: f64) -> Vec<bool> {
        self.0.matches(min_value,max_value)
    }

    fn filter(&self, which: &[bool], primary: bool) -> AnchorPairAxis {
        AnchorPairAxis(self.0.filter(which,primary),self.1.filter(which,false),self.2.filter(which,false))
    }
}

#[derive(Clone,Debug)]
pub struct SingleAnchor(pub SingleAnchorAxis,pub SingleAnchorAxis);

impl SingleAnchor {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn bulk(self, len: usize, primary: bool) -> SingleAnchor {
        SingleAnchor(self.0.bulk(len,primary),self.1.bulk(len,false))
    }

    pub fn matches(&self, min_value: f64, max_value: f64) -> Vec<bool> {
        self.0.matches(min_value,max_value)
    }

    pub fn filter(&self, which: &[bool], primary: bool) -> SingleAnchor {
        SingleAnchor(self.0.filter(which,primary),self.1.filter(which,false))
    }
}

#[derive(Clone,Debug)]
pub struct AnchorPair(pub AnchorPairAxis,pub AnchorPairAxis);

impl AnchorPair {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn bulk(self, len: usize, primary: bool) -> AnchorPair {
        AnchorPair(self.0.bulk(len,primary),self.1.bulk(len,false))
    }

    pub fn matches(&self, min_value: f64, max_value: f64) -> Vec<bool> {
        self.0.matches(min_value,max_value)
    }

    pub fn filter(&self, which: &[bool], primary: bool) -> AnchorPair {
        AnchorPair(self.0.filter(which,primary),self.1.filter(which,false))
    }
}
