use std::{sync::Arc};

use peregrine_data::{AllotmentMetadataRequest, AllotmentMetadataStore, Colour, DirectColour, DrawnType, EachOrEvery, ParameterValue, Patina, ShapeListBuilder, Universe, Variable, VariableValues, HoleySpaceBaseArea, SpaceBase, SpaceBaseArea, PartialSpaceBase, AllotmentRequest, reactive::{Reactive, Observable}};
use crate::{Message, run::{PgConfigKey, PgPeregrineConfig}, shape::util::iterators::eoe_throw};
use peregrine_data::reactive;
use super::spectremanager::SpectreConfigKey;

#[derive(Clone)]
pub(crate) struct AreaVariables {
    tlbr: (Variable,Variable,Variable,Variable),
    variables: VariableValues<f64>
}

impl AreaVariables {
    pub(crate) fn new(variables: &VariableValues<f64>) -> AreaVariables {
        AreaVariables {
            tlbr: (variables.new_variable(0.),variables.new_variable(0.),variables.new_variable(0.),variables.new_variable(0.)),
            variables: variables.clone()
        }
    }

    pub(crate) fn update(&mut self, tlbr: (f64,f64,f64,f64)) {
        self.variables.update_variable(&self.tlbr.0,tlbr.0);
        self.variables.update_variable(&self.tlbr.1,tlbr.1);
        self.variables.update_variable(&self.tlbr.2,tlbr.2);
        self.variables.update_variable(&self.tlbr.3,tlbr.3);
    }

    pub(crate) fn tlbr(&self) -> &(Variable,Variable,Variable,Variable) { &self.tlbr }
}

#[derive(Clone)]
pub(crate) struct AreaVariables2<'a> {
    tlbr: (reactive::Variable<'a,f64>,reactive::Variable<'a,f64>,reactive::Variable<'a,f64>,reactive::Variable<'a,f64>),
    reactive: Reactive<'a>
}

impl<'a> AreaVariables2<'a> {
    pub(crate) fn new(reactive: &Reactive<'a>) -> AreaVariables2<'a> {
        AreaVariables2 {
            tlbr: (reactive.variable(0.),reactive.variable(0.),reactive.variable(0.),reactive.variable(0.)),
            reactive: reactive.clone()
        }
    }

    pub(crate) fn update(&mut self, tlbr: (f64,f64,f64,f64)) {
        self.tlbr.0.set(tlbr.0);
        self.tlbr.1.set(tlbr.1);
        self.tlbr.2.set(tlbr.2);
        self.tlbr.3.set(tlbr.3);
    }

    pub(crate) fn tlbr(&self) -> &(reactive::Variable<'a,f64>,reactive::Variable<'a,f64>,reactive::Variable<'a,f64>,reactive::Variable<'a,f64>) { &self.tlbr }
}

#[derive(Clone)]
pub(crate) struct MarchingAnts {
    area: AreaVariables,
    area2: AreaVariables2<'static>,
    width: f64,
    colour: DirectColour,
    length: u32,
    prop: f64
}

impl MarchingAnts {
    pub(super) fn new(config: &PgPeregrineConfig, area: &AreaVariables, area2: &AreaVariables2<'static>) -> Result<MarchingAnts,Message> {
        Ok(MarchingAnts {
            area: area.clone(),
            area2: area2.clone(),
            width: config.get_f64(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsWidth))?,
            colour: config.get_colour(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsColour))?,
            length: config.get_f64(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsLength))? as u32,
            prop: config.get_f64(&PgConfigKey::Spectre(SpectreConfigKey::MarchingAntsProp))?
        })
    }

    pub(crate) fn draw(&self, shapes: &mut ShapeListBuilder, allotment_metadata: &AllotmentMetadataStore) -> Result<(),Message> {
        allotment_metadata.add(AllotmentMetadataRequest::new("window:origin[101]",0));
        let window_origin = shapes.universe().make_request("window:origin[101]").unwrap(); // XXX
        let pos = self.area.tlbr().clone();
        let pos2 = self.area2.tlbr().clone();
        shapes.use_allotment(&window_origin);
        let top_left = PartialSpaceBase::from_spacebase(SpaceBase::new(
            &EachOrEvery::Each(Arc::new(vec![ParameterValue::Constant(0.)])),
            &EachOrEvery::Each(Arc::new(vec![ParameterValue::Variable(pos.0,0.)])),
            &EachOrEvery::Each(Arc::new(vec![ParameterValue::Variable(pos.1,0.)])),
            &EachOrEvery::Each(Arc::new(vec![window_origin.clone()]))
        ).unwrap());
        let bottom_right =  PartialSpaceBase::from_spacebase(SpaceBase::new(
            &EachOrEvery::Each(Arc::new(vec![ParameterValue::Constant(0.)])),
            &EachOrEvery::Each(Arc::new(vec![ParameterValue::Variable(pos.2,0.)])),
            &EachOrEvery::Each(Arc::new(vec![ParameterValue::Variable(pos.3,0.)])),
            &EachOrEvery::Each(Arc::new(vec![window_origin]))
        ).unwrap());
        let area = HoleySpaceBaseArea::Parametric(SpaceBaseArea::new(top_left,bottom_right).unwrap());
        let top_left_obs = PartialSpaceBase::new(
            &EachOrEvery::Each(Arc::new(vec![Observable::constant(0.)])),
            &EachOrEvery::Every(pos2.0.observable()),
            &EachOrEvery::Every(pos2.1.observable()),
            &EachOrEvery::Every(())
        );
        let bottom_right_obs = PartialSpaceBase::new(
            &EachOrEvery::Each(Arc::new(vec![Observable::constant(0.)])),
            &EachOrEvery::Every(pos2.2.observable()),
            &EachOrEvery::Every(pos2.3.observable()),
            &EachOrEvery::Every(())
        );
        let obs = SpaceBaseArea::new(top_left_obs,bottom_right_obs).unwrap();
        shapes.add_rectangle(area,Patina::Drawn(
            DrawnType::Stroke(self.width as u32),
            EachOrEvery::Every(Colour::Bar(DirectColour(255,255,255,0),self.colour.clone(),(self.length,self.length),self.prop))
        ),Some(obs));
        Ok(())
    }
}

fn make_stain_param(var: Option<&Variable>, c: f64, d: f64) -> ParameterValue<f64> {
    if let Some(var) = var { ParameterValue::Variable(var.clone(),d) } else { ParameterValue::Constant(c) }
}

fn make_stain_param2(var: Option<&reactive::Variable<'static,f64>>, c: f64, d: f64) -> Observable<'static,f64> {
    if let Some(var) = var { var.observable() } else { Observable::constant(c) }
}

fn make_stain_point<X: Clone,Y: Clone>(base: X, normal: X, tangent: X, allotment: &Y) -> Result<PartialSpaceBase<X,Y>,Message> {
    Ok(PartialSpaceBase::from_spacebase(
        eoe_throw("stain1",SpaceBase::new(&EachOrEvery::Each(Arc::new(vec![base])),
        &EachOrEvery::Each(Arc::new(vec![normal])),
        &EachOrEvery::Each(Arc::new(vec![tangent])),
            &EachOrEvery::Every(allotment.clone())))?))

}

fn make_stain_rect(n0: Option<&Variable>, t0: Option<&Variable>, n1: Option<&Variable>, t1: Option<&Variable>, b1: f64, ar: &AllotmentRequest) -> Result<HoleySpaceBaseArea<f64,AllotmentRequest>,Message> {
    let n0 = make_stain_param(n0,0.,0.);
    let t0 = make_stain_param(t0,0.,0.);
    let n1 = make_stain_param(n1,-1.,0.);
    let t1 = make_stain_param(t1,-1.,0.);
    let top_left = make_stain_point(ParameterValue::Constant(0.),n0,t0,ar)?;
    let bottom_right = make_stain_point(ParameterValue::Constant(b1),n1,t1,ar)?;
    Ok(HoleySpaceBaseArea::Parametric(SpaceBaseArea::new(top_left,bottom_right).unwrap()))
}

fn make_stain_rect2(n0: Option<&reactive::Variable<'static,f64>>, t0: Option<&reactive::Variable<'static,f64>>, n1: Option<&reactive::Variable<'static,f64>>, t1: Option<&reactive::Variable<'static,f64>>, b1: f64) -> Result<SpaceBaseArea<Observable<'static,f64>,()>,Message> {
    let n0 = make_stain_param2(n0,0.,0.);
    let t0 = make_stain_param2(t0,0.,0.);
    let n1 = make_stain_param2(n1,-1.,0.);
    let t1 = make_stain_param2(t1,-1.,0.);
    let top_left = make_stain_point(Observable::constant(0.),n0,t0,&())?;
    let bottom_right = make_stain_point(Observable::constant(b1),n1,t1,&())?;
    Ok(SpaceBaseArea::new(top_left,bottom_right).unwrap())
}

#[derive(Clone)]
pub(crate) struct Stain {
    area: AreaVariables,
    area2: AreaVariables2<'static>,
    invert: bool,
    colour: DirectColour
}

impl Stain {
    pub(super) fn new(config: &PgPeregrineConfig, area: &AreaVariables, area2: &AreaVariables2<'static>, invert: bool) -> Result<Stain,Message> {
        Ok(Stain { 
            area: area.clone(),
            area2: area2.clone(),
            invert,
            colour: config.get_colour(&PgConfigKey::Spectre(SpectreConfigKey::StainColour))?
        })
    }
    
    pub(crate) fn draw(&self, shapes: &mut ShapeListBuilder, allotment_metadata: &AllotmentMetadataStore) -> Result<(),Message> {
        allotment_metadata.add(AllotmentMetadataRequest::new("window:origin[100]",-1));
        let window_origin = shapes.universe().make_request("window:origin[100]").unwrap(); // XXX
        shapes.use_allotment(&window_origin);
        let mut rectangles = vec![];
        let pos = self.area.tlbr().clone();
        let pos2 = self.area2.tlbr().clone();
        if self.invert {
            /* top left of screen to bottom of screen, along lefthand edge of selection */
            rectangles.push(
                (make_stain_rect(None,None,None,Some(&pos.1),0.,&window_origin)?,
                 make_stain_rect2(None,None,None,Some(&pos2.1),0.)?));
            /* top right of screen to bottom of screen, along righthand edge of selection */
            rectangles.push(
                (make_stain_rect(None,Some(&pos.3),None,None,1.,&window_origin)?,
                 make_stain_rect2(None,Some(&pos2.3),None,None,1.)?));
            /* length of top of shape from top of screen to that shape */
            rectangles.push((
                make_stain_rect(None,Some(&pos.1),Some(&pos.0),Some(&pos.3),0.,&window_origin)?,
                make_stain_rect2(None,Some(&pos2.1),Some(&pos2.0),Some(&pos2.3),0.)?));
            /* length of bottom of shape from bottom of shape to bottom of screen */
            rectangles.push((make_stain_rect(Some(&pos.2),Some(&pos.1),None,Some(&pos.3),0.,&window_origin)?,
                            make_stain_rect2(Some(&pos2.2),Some(&pos2.1),None,Some(&pos2.3),0.)?));
        } else {
            rectangles.push((make_stain_rect(Some(&pos.0),Some(&pos.1),Some(&pos.2),Some(&pos.3),0.,&window_origin)?,
                            make_stain_rect2(Some(&pos2.0),Some(&pos2.1),Some(&pos2.2),Some(&pos2.3),0.)?));
        }
        for (area,wobble) in rectangles.drain(..) {           
            shapes.add_rectangle(area,Patina::Drawn(DrawnType::Fill,EachOrEvery::Every(Colour::Direct(self.colour.clone()))),Some(wobble))
                .map_err(|e| Message::DataError(e))?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) enum Spectre {
    MarchingAnts(MarchingAnts),
    Stain(Stain),
    Compound(Vec<Spectre>)
}

impl Spectre {
    pub(crate) fn draw(&self, shapes: &mut ShapeListBuilder, allotment_metadata: &AllotmentMetadataStore) -> Result<(),Message> {
        match self {
            Spectre::MarchingAnts(a) => a.draw(shapes,allotment_metadata)?,
            Spectre::Stain(a) => a.draw(shapes,allotment_metadata)?,
            Spectre::Compound(spectres) => {
                for spectre in spectres {
                    spectre.draw(shapes,allotment_metadata)?;
                }
            }
        }
        Ok(())
    }
}
