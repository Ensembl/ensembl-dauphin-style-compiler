use std::collections::HashMap;
use super::core::{ AnchorPair, Patina, SingleAnchor, filter, bulk };

#[derive(Debug)]
enum RectShape {
    SingleAnchor(SingleAnchor,Patina,Vec<String>),
    DoubleAnchor(AnchorPair,Patina,Vec<String>)
}

impl RectShape {
    fn filter(&self, min_value: f64, max_value: f64) -> RectShape {
        match self {
            RectShape::SingleAnchor(anchor,patina,allotment) => {
                let count = anchor.len();
                let anchor = anchor.clone().bulk(count,true);
                let patina = patina.clone().bulk(count,false);
                let allotment = bulk(allotment.clone(),count,false);
                let which = anchor.matches(min_value,max_value);
                RectShape::SingleAnchor(anchor.filter(&which,true),
                                        patina.filter(&which,false),
                                        filter(&allotment,&which,false))
            },
            RectShape::DoubleAnchor(anchor,patina,allotment) => {
                let count = anchor.len();
                let anchor = anchor.clone().bulk(count,true);
                let patina = patina.clone().bulk(count,false);
                let allotment = bulk(allotment.clone(),count,false);
                let which = anchor.matches(min_value,max_value);
                RectShape::DoubleAnchor(anchor.filter(&which,true),
                                        patina.filter(&which,false),
                                        filter(&allotment,&which,false))
            }
        }
    }
}

#[derive(Debug)]
pub struct TrackShapes {
    shapes: Vec<RectShape>
}

impl TrackShapes {
    pub fn new() -> TrackShapes {
        TrackShapes {
            shapes: vec![]
        }
    }

    pub fn add_rectangle_1(&mut self, anchors: SingleAnchor, patina: Patina, allotments: Vec<String>) {
        self.shapes.push(RectShape::SingleAnchor(anchors,patina,allotments));
    }

    pub fn add_rectangle_2(&mut self, anchors: AnchorPair, patina: Patina, allotments: Vec<String>) {
        self.shapes.push(RectShape::DoubleAnchor(anchors,patina,allotments));
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> TrackShapes {
        let mut new = vec![];
        for shape in self.shapes.iter() {
            new.push(shape.filter(min_value,max_value));
        }
        TrackShapes {
            shapes: new
        }
    }
}
