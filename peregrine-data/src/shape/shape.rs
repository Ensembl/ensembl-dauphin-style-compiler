use super::core::{ Patina, filter, bulk, Pen, Plotter };
use std::cmp::{ max, min };
use crate::switch::allotment::AllotmentHandle;
use crate::shape::spacebase::{ SpaceBase, SpaceBaseArea };
use crate::util::ringarray::DataFilter;

#[derive(Clone,Debug)]
pub enum Shape {
    Text2(SpaceBase,Pen,Vec<String>,Vec<AllotmentHandle>),
    Wiggle((f64,f64),Vec<Option<f64>>,Plotter,AllotmentHandle),
    SpaceBaseRect(SpaceBaseArea,Patina,Vec<AllotmentHandle>)
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
            Shape::SpaceBaseRect(area,patina,allotments) => {
                let filter = area.make_base_filter(min_value,max_value);
                Shape::SpaceBaseRect(area.filter(&filter),patina.filter2(&filter),filter.filter(allotments))
            },
            Shape::Text2(position,pen,text,allotments) => {
                let filter = position.make_base_filter(min_value,max_value);
                Shape::Text2(position.filter(&filter),pen.filter2(&filter),filter.filter(text),filter.filter(allotments))
            },
            Shape::Wiggle((x_start,x_end),y,plotter,allotment) => {
                let (aim_min,aim_max,new_y) = wiggle_filter(min_value,max_value,*x_start,*x_end,y);
                Shape::Wiggle((aim_min,aim_max),new_y,plotter.clone(),allotment.clone())
            }
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Shape::SpaceBaseRect(area,patina,allotments) => {
                area.len() == 0
            },
            Shape::Text2(position,pen,text,allotments) => {
                position.len() == 0
            },
            Shape::Wiggle(x,y,plotter,allotment) => {
                y.len() == 0
            }
        }
    }

    pub fn remove_nulls(self) -> Shape {
        match self {
            Shape::SpaceBaseRect(area,patina,allotments) => {
                let mut filter = DataFilter::new_filter(&allotments, |a| !a.is_null());
                filter.set_size(area.len());
                Shape::SpaceBaseRect(area.filter(&filter),patina.filter2(&filter),filter.filter(&allotments))
            },
            Shape::Text2(position,pen,text,allotments) => {
                let mut filter = DataFilter::new_filter(&allotments, |a| !a.is_null());
                filter.set_size(position.len());
                Shape::Text2(position.filter(&filter),pen.filter2(&filter),filter.filter(&text),filter.filter(&allotments))
            },
            Shape::Wiggle(x,mut y,plotter,allotment) => {
                if allotment.is_null() { y = vec![]; }
                Shape::Wiggle(x,y,plotter.clone(),allotment.clone())
            }
        }

    }
}
