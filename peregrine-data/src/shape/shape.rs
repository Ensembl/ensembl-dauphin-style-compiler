use super::core::{ AnchorPair, Patina, SingleAnchor, filter, bulk, Pen, Plotter };
use std::cmp::{ max, min };

#[derive(Clone,Debug)]
pub enum Shape {
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
    pub fn filter(&self, min_value: f64, max_value: f64) -> Shape {
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
