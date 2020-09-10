use std::collections::HashMap;
use std::sync::Arc;
use super::zmenu::ZMenu;

pub trait Texture : std::fmt::Debug {

}

#[derive(Clone,Debug)]
pub struct DirectColour(pub u8,pub u8,pub u8);

#[derive(Clone,Debug)]
pub enum Colour {
    Direct(Vec<DirectColour>),
    Spot(DirectColour)
}

#[derive(Clone,Debug)]
pub enum Patina {
    Filled(Colour),
    Hollow(Colour),
    Texture(Vec<Arc<dyn Texture>>),
    ZMenu(ZMenu,HashMap<String,Vec<String>>)
}

#[derive(Clone,Debug)]
pub enum ScreenEdge { Min(Vec<f64>), Max(Vec<f64>) }

#[derive(Clone,Debug)]
pub enum ShipEnd { Min(Vec<f64>), Centre(Vec<f64>), Max(Vec<f64>) }

#[derive(Clone,Debug)]
pub enum SeaEnd {
    Paper(Vec<f64>),
    Screen(ScreenEdge)
}

impl SeaEnd {
    fn filter(self, min_value: f64, max_value: f64) -> SeaEnd {
        match self {
            SeaEnd::Paper(mut v) => {
                SeaEnd::Paper(v.drain(..).filter(|x| *x >= min_value && *x <= max_value).collect())
            },
            SeaEnd::Screen(s) => SeaEnd::Screen(s)
        }
    }
}

#[derive(Clone,Debug)]
pub enum SeaEndPair {
    Paper(Vec<f64>,Vec<f64>),
    Screen(ScreenEdge,ScreenEdge)
}

impl SeaEndPair {
    fn filter(self, min_value: f64, max_value: f64) -> SeaEndPair {
        match self {
            SeaEndPair::Paper(mut a, b) => {
                let mut x = vec![];
                let mut y = vec![];
                for (a,b) in a.drain(..).zip(b.iter().cycle()).filter(|(a,b)|
                                    **b >= min_value && *a <= max_value
                                ) {
                    x.push(a);
                    y.push(*b);
                }
                SeaEndPair::Paper(x,y)
            },
            SeaEndPair::Screen(a,b) => SeaEndPair::Screen(a,b)
        }
    }
}

#[derive(Clone,Debug)]
pub struct SingleAnchorAxis(pub SeaEnd,pub ShipEnd,pub Vec<f64>);

impl SingleAnchorAxis {
    fn filter(self, min_value: f64, max_value: f64) -> SingleAnchorAxis {
        SingleAnchorAxis(self.0.filter(min_value,max_value),self.1,self.2)
    }
}

#[derive(Clone,Debug)]
pub struct AnchorPairAxis(pub SeaEndPair,pub ShipEnd,pub ShipEnd);

impl AnchorPairAxis {
    fn filter(self, min_value: f64, max_value: f64) -> AnchorPairAxis {
        AnchorPairAxis(self.0.filter(min_value,max_value),self.1,self.2)
    }
}

#[derive(Clone,Debug)]
pub struct SingleAnchor(pub SingleAnchorAxis,pub SingleAnchorAxis);

impl SingleAnchor {
    pub fn filter(self, min_value: f64, max_value: f64) -> SingleAnchor {
        SingleAnchor(self.0.filter(min_value,max_value),self.1)
    }
}

#[derive(Clone,Debug)]
pub struct AnchorPair(pub AnchorPairAxis,pub AnchorPairAxis);

impl AnchorPair {
    pub fn filter(self, min_value: f64, max_value: f64) -> AnchorPair {
        AnchorPair(self.0.filter(min_value,max_value),self.1)
    }
}

pub trait ShapeSet {
}
