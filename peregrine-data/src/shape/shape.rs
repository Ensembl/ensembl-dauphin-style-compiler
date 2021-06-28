use super::core::{ Patina, filter, bulk, Pen, Plotter };
use std::cmp::{ max, min };
use crate::HoleySpaceBase;
use crate::HoleySpaceBaseArea;
use crate::switch::allotment::AllotmentHandle;
use crate::util::ringarray::DataFilter;

#[derive(Clone)]
pub enum Shape {
    Text2(HoleySpaceBase,Pen,Vec<String>,Vec<AllotmentHandle>),
    Wiggle((f64,f64),Vec<Option<f64>>,Plotter,AllotmentHandle),
    SpaceBaseRect(HoleySpaceBaseArea,Patina,Vec<AllotmentHandle>)
}

fn wiggle_filter(wanted_min: f64, wanted_max: f64, got_min: f64, got_max: f64, y: &[Option<f64>]) -> (f64,f64,Vec<Option<f64>>) {
    if y.len() == 0 { return (wanted_min,wanted_max,vec![]); }
    let aim_min = if wanted_min < got_min { got_min } else { wanted_min }; // ie invariant: aim_min >= got_min
    let aim_max = if wanted_max > got_max { got_max } else { wanted_max }; // ie invariant: aim_max <= got_max
    let pitch = (got_max-got_min)/(y.len() as f64);
    let left_truncate = ((aim_min-got_min)/pitch).floor() as i64 - 1;
    let right_truncate = ((got_max-aim_max)/pitch).floor() as i64 - 1;
    let y_len = y.len() as i64;
    let left = min(max(left_truncate,0),y_len);
    let right = max(left,min(max(0,y_len-right_truncate),y_len));
    (aim_min,aim_max,y[(left as usize)..(right as usize)].to_vec())
}

impl Shape {
    pub fn filter(&self, min_value: f64, max_value: f64) -> Shape {
        match self {
            Shape::SpaceBaseRect(area,patina,allotments) => {
                let filter = area.make_base_filter(min_value,max_value);
                Shape::SpaceBaseRect(area.filter(&filter),patina.filter(&filter),filter.filter(allotments))
            },
            Shape::Text2(position,pen,text,allotments) => {
                let filter = position.make_base_filter(min_value,max_value);
                Shape::Text2(position.filter(&filter),pen.filter(&filter),filter.filter(text),filter.filter(allotments))
            },
            Shape::Wiggle((x_start,x_end),y,plotter,allotment) => {
                let (aim_min,aim_max,new_y) = wiggle_filter(min_value,max_value,*x_start,*x_end,y);
                Shape::Wiggle((aim_min,aim_max),new_y,plotter.clone(),allotment.clone())
            }
        }
    }

    pub fn len(&self) -> usize {
        match self {
            Shape::SpaceBaseRect(area,_,_) => area.len(),
            Shape::Text2(position,_,_,_) => position.len(),
            Shape::Wiggle(_,y,_,_) => y.len()
        }
    }

    pub fn is_empty(&self) -> bool { self.len() == 0 }

    pub fn remove_nulls(self) -> Shape {
        match self {
            Shape::SpaceBaseRect(area,patina,allotments) => {
                let mut allotment_iter = allotments.iter();
                let mut filter = DataFilter::new(&mut allotment_iter, |a| !a.is_null());
                filter.set_size(area.len());
                Shape::SpaceBaseRect(area.filter(&filter),patina.filter(&filter),filter.filter(&allotments))
            },
            Shape::Text2(position,pen,text,allotments) => {
                let mut allotment_iter = allotments.iter();
                let mut filter = DataFilter::new(&mut allotment_iter, |a| !a.is_null());
                filter.set_size(position.len());
                Shape::Text2(position.filter(&filter),pen.filter(&filter),filter.filter(&text),filter.filter(&allotments))
            },
            Shape::Wiggle(x,mut y,plotter,allotment) => {
                if allotment.is_null() { y = vec![]; }
                Shape::Wiggle(x,y,plotter.clone(),allotment.clone())
            }
        }

    }
}
