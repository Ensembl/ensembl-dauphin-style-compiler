use std::sync::Arc;

pub trait Texture {

}

pub struct ZMenu {

}

pub struct DirectColour(pub u8,pub u8,pub u8);

pub enum Colour {
    Direct(Vec<DirectColour>),
    Spot(DirectColour)
}

pub enum Patina {
    Filled(Colour),
    Hollow(Colour),
    Texture(Vec<Arc<dyn Texture>>),
    ZMenu(Vec<ZMenu>)
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

#[derive(Clone,Debug)]
pub enum SeaEndPair {
    Paper(Vec<f64>,Vec<f64>),
    Screen(ScreenEdge,ScreenEdge)
}

pub struct SingleAnchorAxis(SeaEnd,ShipEnd);
pub struct AnchorPairAxis(SeaEndPair,ShipEnd,ShipEnd);

pub struct SingleAnchor(SingleAnchorAxis,SingleAnchorAxis);
pub struct AnchorPair(AnchorPairAxis,AnchorPairAxis);

pub trait ShapeSet {

}
