use std::collections::HashMap;
use std::sync::Arc;
use super::zmenu::ZMenu;

pub(super) fn filter<F>(mut x: Vec<F>, w: &[bool]) -> Vec<F> {
    let mut out = vec![];
    for (v,f) in x.drain(..).zip(w.iter().cycle()) {
        if *f { out.push(v); }
    }
    out
}

pub trait Texture : std::fmt::Debug {

}

#[derive(Clone,Debug)]
pub struct DirectColour(pub u8,pub u8,pub u8);

#[derive(Clone,Debug)]
pub enum Colour {
    Direct(Vec<DirectColour>),
    Spot(DirectColour)
}

impl Colour {
    fn filter(self, which: &[bool]) -> Colour {
        match self {
            Colour::Direct(d) => Colour::Direct(filter(d,which)),
            Colour::Spot(d) => Colour::Spot(d)
        }
    }
}

#[derive(Clone,Debug)]
pub enum Patina {
    Filled(Colour),
    Hollow(Colour),
    Texture(Arc<dyn Texture>),
    ZMenu(ZMenu,HashMap<String,Vec<String>>)
}

impl Patina {
    pub fn filter(self, which: &[bool]) -> Patina {
        match self {
            Patina::Filled(c) => Patina::Filled(c.filter(which)),
            Patina::Hollow(c) => Patina::Hollow(c.filter(which)),
            Patina::Texture(t) => Patina::Texture(t),
            Patina::ZMenu(z,mut h) => Patina::ZMenu(z,h.drain().map(|(k,v)| (k,filter(v,which))).collect())
        }
    }
}


#[derive(Clone,Debug)]
pub enum ScreenEdge { Min(Vec<f64>), Max(Vec<f64>) }

#[derive(Clone,Debug)]
pub enum ShipEnd { Min(Vec<f64>), Centre(Vec<f64>), Max(Vec<f64>) }

impl ShipEnd {
    fn map_all_into<F>(self, cb: F) -> ShipEnd where F: Fn(Vec<f64>) -> Vec<f64> {
        match self {
            ShipEnd::Min(x) => ShipEnd::Min(cb(x)),
            ShipEnd::Centre(x) => ShipEnd::Centre(cb(x)),
            ShipEnd::Max(x) => ShipEnd::Max(cb(x))
        }
    }

    fn filter(self, which: &[bool]) -> ShipEnd {
        self.map_all_into(|mut x| filter(x,which))
    }
}

#[derive(Clone,Debug)]
pub enum SeaEnd {
    Paper(Vec<f64>),
    Screen(ScreenEdge)
}

impl SeaEnd {
    fn matches(&self, min_value: f64, max_value: f64) -> Vec<bool> {
        match &self {
            SeaEnd::Paper(v) => {
                v.iter().map(|x| *x >= min_value && *x <= max_value).collect()
            },
            SeaEnd::Screen(ScreenEdge::Min(s)) => s.iter().map(|_| true).collect(),
            SeaEnd::Screen(ScreenEdge::Max(s)) => s.iter().map(|_| true).collect(),
        }
    }

    fn filter(self, which: &[bool]) -> SeaEnd {
        match self {
            SeaEnd::Paper(v) => SeaEnd::Paper(filter(v,which)),
            x => x
        }
    }
}

#[derive(Clone,Debug)]
pub enum SeaEndPair {
    Paper(Vec<f64>,Vec<f64>),
    Screen(ScreenEdge,ScreenEdge)
}

impl SeaEndPair {
    fn matches(&self, min_value: f64, max_value: f64) -> Vec<bool> {
        match &self {
            SeaEndPair::Paper(a,b) => {
                let x = a.iter().map(|x| *x >= min_value && *x <= max_value);
                let y = b.iter().map(|x| *x >= min_value && *x <= max_value);
                x.zip(y.cycle()).map(|(c,d)| c || d).collect()
            },
            SeaEndPair::Screen(ScreenEdge::Min(s),_) =>  s.iter().map(|_| true).collect(),
            SeaEndPair::Screen(ScreenEdge::Max(s),_) =>  s.iter().map(|_| true).collect(),
        }
    }

    fn filter(self, which: &[bool]) -> SeaEndPair {
        match self {
            SeaEndPair::Paper(a,b) => SeaEndPair::Paper(filter(a,which),filter(b,which)),
            x => x
        }
    }
}

#[derive(Clone,Debug)]
pub struct SingleAnchorAxis(pub SeaEnd,pub ShipEnd,pub Vec<f64>);

impl SingleAnchorAxis {
    fn matches(&self, min_value: f64, max_value: f64) -> Vec<bool> {
        self.0.matches(min_value,max_value)
    }

    fn filter(self, which: &[bool]) -> SingleAnchorAxis {
        SingleAnchorAxis(self.0.filter(which),self.1.filter(which),filter(self.2,which))
    }
}

#[derive(Clone,Debug)]
pub struct AnchorPairAxis(pub SeaEndPair,pub ShipEnd,pub ShipEnd);

impl AnchorPairAxis {
    fn matches(&self, min_value: f64, max_value: f64) -> Vec<bool> {
        self.0.matches(min_value,max_value)
    }

    fn filter(self, which: &[bool]) -> AnchorPairAxis {
        AnchorPairAxis(self.0.filter(which),self.1.filter(which),self.2.filter(which))
    }
}

#[derive(Clone,Debug)]
pub struct SingleAnchor(pub SingleAnchorAxis,pub SingleAnchorAxis);

impl SingleAnchor {
    pub fn matches(&self, min_value: f64, max_value: f64) -> Vec<bool> {
        self.0.matches(min_value,max_value)
    }

    pub fn filter(self, which: &[bool]) -> SingleAnchor {
        SingleAnchor(self.0.filter(which),self.1.filter(which))
    }
}

#[derive(Clone,Debug)]
pub struct AnchorPair(pub AnchorPairAxis,pub AnchorPairAxis);

impl AnchorPair {
    pub fn matches(&self, min_value: f64, max_value: f64) -> Vec<bool> {
        self.0.matches(min_value,max_value)
    }

    pub fn filter(self, which: &[bool]) -> AnchorPair {
        AnchorPair(self.0.filter(which),self.1.filter(which))
    }
}

pub trait ShapeSet {
}
