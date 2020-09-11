use super::core::{ AnchorPair, Patina, SingleAnchor, filter, bulk, Pen};

#[derive(Debug)]
enum Shape {
    SingleAnchorRect(SingleAnchor,Patina,Vec<String>,Vec<f64>,Vec<f64>),
    DoubleAnchorRect(AnchorPair,Patina,Vec<String>),
    Text(SingleAnchor,Pen,Vec<String>,Vec<String>)
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
