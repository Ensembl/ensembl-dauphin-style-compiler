use peregrine_toolkit::puzzle::PuzzleSolution;

use crate::{DataMessage, Plotter, ShapeDemerge, Shape, util::{eachorevery::EachOrEveryFilter}, allotment::{transformers::transformers::Transformer, style::{pendingleaf::PendingLeaf, style::LeafCommonStyle}}, EachOrEvery, CoordinateSystem, Pen};
use std::{cmp::{max, min}, hash::Hash, sync::Arc};

const SCALE : i64 = 200; // XXX configurable

fn wiggle_filter(wanted_min: f64, wanted_max: f64, got_min: f64, got_max: f64, y: &[Option<f64>]) -> (f64,f64,Vec<Option<f64>>) {
    if y.len() == 0 { return (wanted_min,wanted_max,vec![]); }
    /* add in angel's share */
    let angel_share = (wanted_max-wanted_min)/(SCALE as f64);
    let wanted_min = (wanted_min - angel_share).floor();
    let wanted_max = (wanted_max + angel_share).ceil();
    /* truncation */
    let aim_min = if wanted_min < got_min { got_min } else { wanted_min }; // ie invariant: aim_min >= got_min
    let aim_max = if wanted_max > got_max { got_max } else { wanted_max }; // ie invariant: aim_max <= got_max
    let pitch = (got_max-got_min)/(y.len() as f64);
    let left_truncate = ((aim_min-got_min)/pitch).floor() as i64;
    let right_truncate = ((got_max-aim_max)/pitch).floor() as i64;
    let y_len = y.len() as i64;
    let left = min(max(left_truncate,0),y_len);
    let right = max(left,min(max(0,y_len-right_truncate),y_len-1));
    /* weeding */
    let y = if right-left+1 > SCALE*2 {
        let mut y2 = vec![];
        let input = &y[(left as usize)..(right as usize)];
        let mut index = 0.5;
        let incr_index = (input.len() as f64)/(SCALE as f64);
        for _ in 0_usize..(SCALE as usize) {
            y2.push(input[min(index as usize,y.len()-1)].clone());
            index += incr_index;
        }
        y2
    } else {
        let right = min(1+(right as usize),y.len());
        y[(left as usize)..(right as usize)].to_vec()
    };
    (aim_min,aim_max,y)
}

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct WiggleShape<A> {
    x_limits: (f64,f64),
    values: Arc<Vec<Option<f64>>>,
    plotter: Plotter,
    allotments: EachOrEvery<A> // actually always a single allotment
}

impl<A> Clone for WiggleShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self { x_limits: self.x_limits.clone(), values: self.values.clone(), plotter: self.plotter.clone(), allotments: self.allotments.clone() }
    }
}

fn draw_wiggle(input: &[Option<f64>], height: f64) -> Vec<Option<f64>> {
    input.iter().map(|y| y.map(|y| ((1.-y)*height))).collect::<Vec<_>>()
}

impl<A> WiggleShape<A> {
    pub fn map_new_allotment<F,B>(&self, cb: F) -> WiggleShape<B> where F: Fn(&A) -> B {
        WiggleShape {
            x_limits: self.x_limits.clone(),
            values: self.values.clone(),
            plotter: self.plotter.clone(),
            allotments: self.allotments.map(cb)
        }
    }

    pub fn len(&self) -> usize { 1 }
    pub fn plotter(&self) -> &Plotter { &self.plotter }

    pub fn iter_allotments(&self, len: usize) -> impl Iterator<Item=&A> {
        self.allotments.iter(len).unwrap()
    }

    pub fn new_details(x_limits: (f64,f64), values: Vec<Option<f64>>, plotter: Plotter, allotment: A) -> WiggleShape<A> {
        WiggleShape {
            x_limits,
            values: Arc::new(draw_wiggle(&values,plotter.0)),
            plotter,
            allotments: EachOrEvery::each(vec![allotment])
        }
    }

    pub fn base_filter(&self, min: f64, max: f64) -> WiggleShape<A> {
        let (aim_min,aim_max,new_y) = wiggle_filter(min,max,self.x_limits.0,self.x_limits.1,&self.values);
        WiggleShape {
            x_limits: (aim_min,aim_max),
            values: Arc::new(new_y),
            plotter: self.plotter.clone(),
            allotments: self.allotments.clone()
        }
    }
}

impl WiggleShape<PendingLeaf> {
    pub fn new2(x_limits: (f64,f64), values: Vec<Option<f64>>,plotter: Plotter, pending_leaf: PendingLeaf) -> Result<Shape<PendingLeaf>,DataMessage> {
        let details = WiggleShape::new_details(x_limits,values,plotter,pending_leaf);
        Ok(Shape::Wiggle(details))
    }
}

impl WiggleShape<LeafCommonStyle> {
    pub fn get_style(&self) -> &LeafCommonStyle { &self.allotments.get(0).unwrap() }
}

impl<A: Clone> WiggleShape<A> {
    /*
    pub fn new(x_limits: (f64,f64), values: Vec<Option<f64>>, plotter: Plotter, allotment: AllotmentRequest) -> Result<Vec<Shape<AllotmentRequest>>,DataMessage> {
        let mut out = vec![];
        let details = WiggleShape::new_details(x_limits,values,plotter,allotment.clone());
        out.push(Shape::Wiggle(details));
        Ok(out)
    }
    */

    pub(super) fn filter(&self, filter: &EachOrEveryFilter) -> WiggleShape<A> {
        let y = if filter.filter_clone(&[()]).len() > 0 {
            self.values.clone()
        } else {
            Arc::new(vec![])
        };
        WiggleShape {
            x_limits: self.x_limits,
            values: y,
            plotter: self.plotter.clone(),
            allotments: self.allotments.clone()
        }
    }

    pub fn range(&self) -> (f64,f64) { self.x_limits }
    pub fn values(&self) -> Arc<Vec<Option<f64>>> { self.values.clone() }

    pub fn make_base_filter(&self, _min: f64, _max: f64) -> EachOrEveryFilter {
        EachOrEveryFilter::all(1)
    }
}

/*
impl WiggleShape<AllotmentRequest> {
    pub fn allot<F,E>(self, cb: F) -> Result<WiggleShape<AllotmentBox>,E> where F: Fn(&AllotmentRequest) -> Result<AllotmentBox,E> {
        let allotments = self.allotments.map_results(cb)?;
        Ok(WiggleShape {
            x_limits: self.x_limits,
            values: self.values,
            plotter: self.plotter,
            allotments
        })
    }
}
*/

impl WiggleShape<LeafCommonStyle> {
    pub fn demerge<T: Hash + PartialEq + Eq,D>(self, cat: &D) -> Vec<(T,WiggleShape<LeafCommonStyle>)> where D: ShapeDemerge<X=T> {
        let demerge = self.allotments.demerge(1,|a| cat.categorise(&a.coord_system));
        let mut out = vec![];
        for (draw_group,mut filter) in demerge {
            out.push((draw_group,self.filter(&mut filter)));
        }
        out
    }

}

/*
impl WiggleShape<AllotmentBox> {
    pub fn transform(&self, common: &ShapeCommon, solution: &PuzzleSolution) -> WiggleShape<LeafCommonStyle> {
        let allotment = self.allotments.get(0).unwrap();
        WiggleShape {
            x_limits: self.x_limits.clone(),
            values: Arc::new(transform_yy(solution,common.coord_system(),allotment,&self.values)),
            plotter: self.plotter.clone(),
            allotments: EachOrEvery::each(vec![])
        }
    }
}
*/

impl WiggleShape<Arc<dyn Transformer>> {
    pub fn make(&self, _solution: &PuzzleSolution) -> Vec<WiggleShape<LeafCommonStyle>> {
        let allotment = self.allotments.get(0).unwrap();
        let (variety,coord_system) = allotment.choose_variety();
        vec![WiggleShape {
            x_limits: self.x_limits.clone(),
            values: Arc::new(variety.graph_transform(&coord_system, allotment,&self.values)),
            plotter: self.plotter.clone(),
            allotments: EachOrEvery::each(vec![allotment.get_style().clone()])
        }]
    }
}

