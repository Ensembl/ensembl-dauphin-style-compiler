use super::core::{ AnchorPair, Patina, SingleAnchor, filter, bulk, Pen, Plotter };
use std::cmp::{ max, min };

#[derive(Debug)]
enum Shape {
    SingleAnchorRect(SingleAnchor,Patina,Vec<String>,Vec<f64>,Vec<f64>),
    DoubleAnchorRect(AnchorPair,Patina,Vec<String>),
    Text(SingleAnchor,Pen,Vec<String>,Vec<String>),
    Wiggle((f64,f64),Vec<Option<f64>>,Plotter,String)
}

fn wiggle_filter(wanted_min: f64, wanted_max: f64, got_min: f64, got_max: f64, y: &[Option<f64>]) -> (f64,f64,Vec<Option<f64>>) {
    if y.len() == 0 { return (wanted_min,wanted_max,vec![]); }
    let aim_min = if wanted_min < got_min { got_min } else { wanted_min }; // ie invariant: aim_min >= got_min
    let aim_max = if wanted_max > got_max { got_max } else { wanted_max }; // ie invariant: aim_max <= got_max
    let pitch = (got_max-got_min)/(y.len() as f64);
    let left_truncate = ((aim_min-got_min)/pitch).floor() as usize -1;
    let right_truncate = ((got_max-aim_max)/pitch).floor() as usize -1;
    let left = min(max(left_truncate,0),y.len());
    let right = max(left,min(max(0,y.len()-right_truncate),y.len()));
    (aim_min,aim_max,y[left..right].to_vec())
}

impl Shape {
    fn filter(&self, min_value: f64, max_value: f64) -> Shape {
        match self {
            Shape::SingleAnchorRect(anchor,patina,allotment,x_size,y_size) => {
                let count = anchor.len();
                let anchor = anchor.clone().bulk(count,true);
                let patina = patina.clone().bulk(count,false);
                let x_size = bulk(x_size.clone(),count,false);
                let y_size = bulk(y_size.clone(),count,false);
                let allotment = bulk(allotment.clone(),count,false);
                let which = anchor.matches(min_value,max_value);
                Shape::SingleAnchorRect(anchor.filter(&which,true),
                                        patina.filter(&which,false),
                                        filter(&allotment,&which,false),
                                        filter(&x_size,&which,false),
                                        filter(&y_size,&which,false))

            },
            Shape::DoubleAnchorRect(anchor,patina,allotment) => {
                let count = anchor.len();
                let anchor = anchor.clone().bulk(count,true);
                let patina = patina.clone().bulk(count,false);
                let allotment = bulk(allotment.clone(),count,false);
                let which = anchor.matches(min_value,max_value);
                Shape::DoubleAnchorRect(anchor.filter(&which,true),
                                        patina.filter(&which,false),
                                        filter(&allotment,&which,false))
            },

            Shape::Text(anchor,pen,allotment,text) => {
                let count = anchor.len();
                let anchor = anchor.clone().bulk(count,true);
                let pen = pen.clone().bulk(count,false);                
                let allotment = bulk(allotment.clone(),count,false);
                let text = bulk(text.clone(),count,false);
                let which = anchor.matches(min_value,max_value);
                Shape::Text(anchor.filter(&which,true),
                            pen.filter(&which,false),
                            filter(&text,&which,false),
                            filter(&allotment,&which,false))
            },

            Shape::Wiggle((x_start,x_end),y,plotter,allotment) => {
                let (aim_min,aim_max,new_y) = wiggle_filter(min_value,max_value,*x_start,*x_end,y);
                Shape::Wiggle((aim_min,aim_max),new_y,plotter.clone(),allotment.clone())
            }
        }
    }
}

#[derive(Debug)]
pub struct TrackShapes {
    shapes: Vec<Shape>
}

impl TrackShapes {
    pub fn new() -> TrackShapes {
        TrackShapes {
            shapes: vec![]
        }
    }

    pub fn add_rectangle_1(&mut self, anchors: SingleAnchor, patina: Patina, allotments: Vec<String>, x_size: Vec<f64>, y_size: Vec<f64>) {
        self.shapes.push(Shape::SingleAnchorRect(anchors,patina,allotments,x_size,y_size));
    }

    pub fn add_rectangle_2(&mut self, anchors: AnchorPair, patina: Patina, allotments: Vec<String>) {
        self.shapes.push(Shape::DoubleAnchorRect(anchors,patina,allotments));
    }

    pub fn add_text(&mut self, anchors: SingleAnchor, pen: Pen, text: Vec<String>, allotments: Vec<String>) {
        self.shapes.push(Shape::Text(anchors,pen,text,allotments));
    }

    pub fn add_wiggle(&mut self, min: f64, max: f64, plotter: Plotter, values: Vec<Option<f64>>, allotment: String) {
        self.shapes.push(Shape::Wiggle((min,max),values,plotter,allotment))
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
