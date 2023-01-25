use eachorevery::{EachOrEveryFilter, EachOrEvery};
use crate::{DataMessage, Patina, ShapeDemerge, Shape, SpaceBaseArea, reactive::Observable, allotment::{leafs::anchored::AnchoredLeaf}, LeafRequest, CoordinateSystem, AuxLeaf, PartialSpaceBase, SpaceBase};
use std::{hash::Hash};

#[cfg_attr(debug_assertions,derive(Debug))]
pub struct RectangleShape<A> {
    area: SpaceBaseArea<f64,A>,
    run: Option<EachOrEvery<f64>>,
    patina: Patina,
    wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>
}

impl<A> RectangleShape<A> {
    fn new_details(area: SpaceBaseArea<f64,A>, run: Option<EachOrEvery<f64>>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<RectangleShape<A>,DataMessage> {
        if !patina.compatible(area.len()) { return Err(DataMessage::LengthMismatch(format!("rectangle patina"))); }
        Ok(RectangleShape {
            area, run, patina, wobble
        })
    }

    pub fn map_new_allotment<F,B>(&self, cb: F) -> RectangleShape<B> where F: FnMut(&A) -> B {
        RectangleShape {
            area: self.area.map_allotments(cb),
            run: self.run.clone(),
            patina: self.patina.clone(),
            wobble: self.wobble.clone()
        }
    }

    pub(super) fn filter(&self, filter: &EachOrEveryFilter) -> RectangleShape<A> {
        RectangleShape {
            area: self.area.filter(filter),
            run: self.run.as_ref().map(|x| x.filter(filter)),
            patina: self.patina.filter(filter),
            wobble: self.wobble.as_ref().map(|w| w.filter(filter))
        }
    }

    pub(crate) fn len(&self) -> usize { self.area.len() }
    pub fn area(&self) -> &SpaceBaseArea<f64,A> { &self.area }
    pub fn run(&self) -> &Option<EachOrEvery<f64>> { &self.run }
    pub fn patina(&self) -> &Patina { &self.patina }
    pub fn wobble(&self) -> &Option<SpaceBaseArea<Observable<'static,f64>,()>> { &self.wobble }
}

fn run_area<A: Clone>(area: &SpaceBaseArea<f64,A>, run: &EachOrEvery<f64>) -> SpaceBaseArea<f64,A> {
    let mut new_end = area.bottom_right().clone();
    if let Some(mut run) = run.iter(area.len()) {
        new_end.update_base(|b| { *run.next().unwrap_or(b) })
    }
    SpaceBaseArea::new(
        PartialSpaceBase::from_spacebase(area.top_left().clone()),
        PartialSpaceBase::from_spacebase(new_end)
    ).unwrap_or_else(|| area.clone())
}

impl RectangleShape<LeafRequest> {
    pub fn new(area: SpaceBaseArea<f64,LeafRequest>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<Shape<LeafRequest>,DataMessage> {
        let details = RectangleShape::new_details(area,None,patina.clone(),wobble.clone())?;
        Ok(Shape::Rectangle(details))
    }

    pub fn new_running(area: SpaceBaseArea<f64,LeafRequest>, run: EachOrEvery<f64>, patina: Patina, wobble: Option<SpaceBaseArea<Observable<'static,f64>,()>>) -> Result<Shape<LeafRequest>,DataMessage> {
        let details = RectangleShape::new_details(area,Some(run),patina.clone(),wobble.clone())?;
        Ok(Shape::Rectangle(details))
    }

    pub fn base_filter(&self, min_value: f64, max_value: f64) -> RectangleShape<LeafRequest> {
        let non_tracking = self.area.top_left().allotments().make_filter(self.area.len(),|a| !a.leaf_style().aux.coord_system.is_tracking());
        let total_area = if let Some(run) = &self.run {
            run_area(&self.area,run)
        } else {
            self.area.clone()
        };
        let filter = total_area.make_base_filter(min_value,max_value);
        self.filter(&filter.or(&non_tracking))
    }
}

impl<A> Clone for RectangleShape<A> where A: Clone {
    fn clone(&self) -> Self {
        Self { area: self.area.clone(), run: self.run.clone(), patina: self.patina.clone(), wobble: self.wobble.clone() }
    }
}

impl RectangleShape<AuxLeaf> {
    pub fn demerge<T: Hash + Clone + Eq,D>(self, cat: &D) -> Vec<(T,RectangleShape<AuxLeaf>)> where D: ShapeDemerge<X=T> {
        let demerge = match &self.patina {
            Patina::Drawn(drawn_type,colours) => {
                let allotments_and_colours = self.area.top_left().allotments().zip(&colours,|x,y| (x.clone(),y.clone()));
                allotments_and_colours.demerge(self.area.len(),|(a,c)| 
                    cat.categorise_with_colour(&a.coord_system,a.depth,drawn_type,c)
                )
            },
            _ => {
                self.area.top_left().allotments().demerge(self.area.len(),|a| cat.categorise(&a.coord_system,a.depth))
            }
        };
        let mut out = vec![];
        for (draw_group,filter) in demerge {
            if filter.count() > 0 {
                out.push((draw_group,self.filter(&filter)));
            }
        }
        out
    }
}

impl RectangleShape<AnchoredLeaf> {
    fn demerge_by_variety(&self) -> Vec<(CoordinateSystem,RectangleShape<AnchoredLeaf>)> {
        let demerge = self.area.top_left().allotments().demerge(self.area.len(),|x| {
            x.coordinate_system().clone()
        });
        let mut out = vec![];
        for (coordinate_system,filter) in demerge {
            out.push((coordinate_system,self.filter(&filter)));
        }
        out
    }

    pub fn make(&self) -> Vec<RectangleShape<AuxLeaf>> {
        let mut out = vec![];
        for (coord_system,rectangles) in self.demerge_by_variety() {
            out.push(RectangleShape {
                area: rectangles.area.spacebasearea_transform(&coord_system),
                run: rectangles.run.clone(),
                patina: rectangles.patina.clone(),
                wobble: rectangles.wobble.clone()
            });
        }
        out
    }
}
