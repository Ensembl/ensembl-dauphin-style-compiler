use std::collections::HashMap;
use std::sync::Arc;
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

pub(super) fn track_split<T>(mut input: Vec<T>, mapping: &[usize], primary: bool) -> Vec<Vec<T>> where T: Clone {
    let max_val = *mapping.iter().max().unwrap_or(&0);
    let mut out : Vec<Vec<T>> = (0..(max_val+1)).map(|_| vec![]).collect();
    if input.len() > 1 || primary {
        for (v,p) in input.drain(..).zip(mapping.iter().cycle()) {
            out[*p].push(v);
        }
    } else if input.len() == 1 {
        for x in out.iter_mut() {
            x.push(input[0].clone());
        }
    }
    out
}

pub(super) fn track_all<T>(input: T, mapping: &[usize]) -> Vec<T> where T: Clone {
    let max_val = *mapping.iter().max().unwrap_or(&0);
    (0..(max_val+1)).map(|_| input.clone()).collect()
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

pub trait Texture : std::fmt::Debug {

}

#[derive(Clone,Debug)]
pub struct DirectColour(pub u8,pub u8,pub u8);

#[derive(Clone,Debug)]
pub struct Pen(pub String,pub f64,pub Vec<DirectColour>);

impl Pen {
    pub fn bulk(self, len: usize, primary: bool) -> Pen {
        Pen(self.0,self.1,bulk(self.2,len,primary))
    }

    pub fn filter(&self, which: &[bool], primary: bool) -> Pen {
        Pen(self.0.clone(),self.1.clone(),filter(&self.2,which,primary))
    }

    pub fn split(self, mapping: &[usize], primary: bool) -> Vec<Pen> {
        let font = self.0;
        let size = self.1;
        track_split(self.2,mapping,primary).drain(..).map(move |x| Pen(font.clone(),size,x)).collect()
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

    pub fn split(self, mapping: &[usize], primary: bool) -> Vec<Colour> {
        match self {
            Colour::Direct(d) => track_split(d,mapping,primary).drain(..).map(|x| Colour::Direct(x)).collect(),
            Colour::Spot(d) => track_all(d,mapping).drain(..).map(|x| Colour::Spot(x)).collect()
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
    Texture(Arc<dyn Texture>),
    ZMenu(ZMenu,HashMap<String,Vec<String>>)
}

impl Patina {
    pub fn bulk(self, len: usize, primary: bool) -> Patina {
        match self {
            Patina::Filled(c) => Patina::Filled(c.bulk(len,primary)),
            Patina::Hollow(c) => Patina::Hollow(c.bulk(len,primary)),
            Patina::Texture(t) => Patina::Texture(t),
            Patina::ZMenu(z,mut h) => {
                let mut new_h  : HashMap<String,Vec<String>> = h.clone();
                for (k,v) in h.drain() {
                    new_h.insert(k,if v.len() > 1 { bulk(v,len,primary) } else { v });
                }
                Patina::ZMenu(z,new_h)
            }
        }
    }

    pub fn split(self, mapping:&[usize], primary: bool) -> Vec<Patina> {
        match self {
            Patina::Filled(c) => c.split(mapping,primary).drain(..).map(|x| Patina::Filled(x)).collect(),
            Patina::Hollow(c) => c.split(mapping,primary).drain(..).map(|x| Patina::Hollow(x)).collect(),
            Patina::Texture(t) => track_all(t,mapping).drain(..).map(|x| Patina::Texture(x)).collect(),
            Patina::ZMenu(z,mut h) => {
                let mut vmap2  : Vec<HashMap<String,Vec<String>>> = track_all(HashMap::new(),&mapping);
                for (k,v) in h.drain() {
                    for (i,v2) in track_split(v,mapping,primary).drain(..).enumerate() {
                        vmap2[i].insert(k.to_string(),v2);
                    }
                }
                vmap2.drain(..).map(|h| Patina::ZMenu(z.clone(),h)).collect()
            }
        }
    }

    pub fn filter(&self, which: &[bool], primary: bool) -> Patina {
        match self {
            Patina::Filled(c) => Patina::Filled(c.filter(which,primary)),
            Patina::Hollow(c) => Patina::Hollow(c.filter(which,primary)),
            Patina::Texture(t) => Patina::Texture(t.clone()),
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

    fn len(&self) -> usize {
        match self {
            ScreenEdge::Min(x) => x.len(),
            ScreenEdge::Max(x) => x.len()
        }
    }

    pub fn split(self, mapping: &[usize], primary: bool) -> Vec<ScreenEdge> {
        match self {
            ScreenEdge::Min(x) => track_split(x,&mapping,primary).drain(..).map(|x| ScreenEdge::Min(x)).collect(),
            ScreenEdge::Max(x) => track_split(x,&mapping,primary).drain(..).map(|x| ScreenEdge::Max(x)).collect()
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

    fn split(self, mapping: &[usize], primary: bool) -> Vec<ShipEnd> {
        match self {
            ShipEnd::Min(x) => track_split(x,mapping,primary).drain(..).map(|x| ShipEnd::Min(x)).collect(),
            ShipEnd::Centre(x) => track_split(x,mapping,primary).drain(..).map(|x| ShipEnd::Centre(x)).collect(),
            ShipEnd::Max(x) => track_split(x,mapping,primary).drain(..).map(|x| ShipEnd::Max(x)).collect(),
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

    fn len(&self) -> usize {
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

    fn split(self, mapping: &[usize], primary: bool) -> Vec<SeaEnd> {
        match self {
            SeaEnd::Paper(x) => track_split(x,&mapping,primary).drain(..).map(|x| SeaEnd::Paper(x)).collect(),
            SeaEnd::Screen(x) => x.split(mapping,primary).drain(..).map(|x| SeaEnd::Screen(x)).collect()
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

    fn split(self, mapping: &[usize], primary: bool) -> Vec<SeaEndPair> {
        match self {
            SeaEndPair::Paper(a,b) => {
                let mut a = track_split(a,&mapping,primary);
                let mut b = track_split(b,&mapping,false);
                a.drain(..).zip(b.drain(..)).map(|(c,d)| SeaEndPair::Paper(c,d)).collect()
            },
            SeaEndPair::Screen(a,b) => {
                let mut a = a.split(&mapping,primary);
                let mut b = b.split(&mapping,false);
                a.drain(..).zip(b.drain(..)).map(|(c,d)| SeaEndPair::Screen(c,d)).collect()
            }
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

    fn split(self, mapping: &[usize], primary: bool) -> Vec<SingleAnchorAxis> {
        let (sea,ship) = (self.0,self.1);
        let mut sea = sea.split(mapping,primary);
        let mut ship = ship.split(mapping,false);
        let it = sea.drain(..).zip(ship.drain(..));
        it.map(|(sea,ship)| SingleAnchorAxis(sea,ship)).collect()
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

    fn split(self, mapping: &[usize], primary: bool) -> Vec<AnchorPairAxis> {
        let (sea,ship_a,ship_b) = (self.0,self.1,self.2);
        let mut sea = sea.split(mapping,primary);
        let mut ship_a = ship_a.split(mapping,false);
        let mut ship_b = ship_b.split(mapping,false);
        let it = sea.drain(..).zip(ship_a.drain(..).zip(ship_b.drain(..)));
        it.map(|(sea,(ship_a,ship_b))| AnchorPairAxis(sea,ship_a,ship_b)).collect()
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

    pub fn split(self, mapping: &[usize], primary: bool) -> Vec<SingleAnchor> {
        let (a,b) = (self.0,self.1);
        let mut a = a.split(mapping,primary);
        let mut b = b.split(mapping,false);
        a.drain(..).zip(b.drain(..)).map(|(x,y)| SingleAnchor(x,y)).collect()
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

    pub fn split(self, mapping: &[usize], primary: bool) -> Vec<AnchorPair> {
        let (a,b) = (self.0,self.1);
        let mut a = a.split(mapping,primary);
        let mut b = b.split(mapping,false);
        a.drain(..).zip(b.drain(..)).map(|(x,y)| AnchorPair(x,y)).collect()
    }

    pub fn matches(&self, min_value: f64, max_value: f64) -> Vec<bool> {
        self.0.matches(min_value,max_value)
    }

    pub fn filter(&self, which: &[bool], primary: bool) -> AnchorPair {
        AnchorPair(self.0.filter(which,primary),self.1.filter(which,false))
    }
}
