use std::sync::Arc;

use peregrine_data::{AllotmentMetadataRequest, AllotmentMetadataStore, Colour, DirectColour, DrawnType, EachOrEvery, HoleySpaceBaseArea, ParameterValue, Patina, ShapeListBuilder, SpaceBase, SpaceBaseArea, Universe, Variable, VariableValues, HoleySpaceBaseArea2, SpaceBase2, SpaceBaseArea2, PartialSpaceBase2};
use crate::{Message, run::{PgConfigKey, PgPeregrineConfig}};

use super::spectremanager::SpectreConfigKey;

#[cfg_attr(debug_assertions,derive(Debug))]
struct BoundingBox {
    tlbr: (f64,f64,f64,f64)
}

#[derive(Clone)]
pub(crate) struct VariableArea {
    vars: VariableValues<f64>,
    tlbr: (Variable,Variable,Variable,Variable)
}

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
pub(crate) struct MarchingAnts {
    area: AreaVariables,
    width: f64,
    colour: DirectColour,
    length: u32,
    prop: f64
}

impl MarchingAnts {
    pub(super) fn new(config: &PgPeregrineConfig, area: &AreaVariables) -> Result<MarchingAnts,Message> {
        Ok(MarchingAnts {
            area: area.clone(),
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
        shapes.use_allotment(&window_origin);
        let top_left = PartialSpaceBase2::from_spacebase(SpaceBase2::new(
            &EachOrEvery::Each(Arc::new(vec![ParameterValue::Constant(0.)])),
            &EachOrEvery::Each(Arc::new(vec![ParameterValue::Variable(pos.0,0.)])),
            &EachOrEvery::Each(Arc::new(vec![ParameterValue::Variable(pos.1,0.)])),
            &EachOrEvery::Each(Arc::new(vec![window_origin.clone()]))
        ).unwrap());
        let bottom_right =  PartialSpaceBase2::from_spacebase(SpaceBase2::new(
            &EachOrEvery::Each(Arc::new(vec![ParameterValue::Constant(0.)])),
            &EachOrEvery::Each(Arc::new(vec![ParameterValue::Variable(pos.2,16.)])),
            &EachOrEvery::Each(Arc::new(vec![ParameterValue::Variable(pos.3,16.)])),
            &EachOrEvery::Each(Arc::new(vec![window_origin]))
        ).unwrap());
        let area = HoleySpaceBaseArea2::Parametric(SpaceBaseArea2::new(top_left,bottom_right).unwrap());
        shapes.add_rectangle(area,Patina::Drawn(
            DrawnType::Stroke(self.width as u32),
            EachOrEvery::Every(Colour::Bar(DirectColour(255,255,255,0),self.colour.clone(),(self.length,self.length),self.prop))
        ));
        Ok(())
    }
}

#[derive(Clone)]
pub(crate) struct Stain {
    area: AreaVariables,
    invert: bool,
    colour: DirectColour
}

impl Stain {
    pub(super) fn new(config: &PgPeregrineConfig, area: &AreaVariables,invert: bool) -> Result<Stain,Message> {
        Ok(Stain { 
            area: area.clone(),
            invert,
            colour: config.get_colour(&PgConfigKey::Spectre(SpectreConfigKey::StainColour))?
        })
    }
    
    pub(crate) fn draw(&self, shapes: &mut ShapeListBuilder, allotment_metadata: &AllotmentMetadataStore) -> Result<(),Message> {
        return Ok(());
        allotment_metadata.add(AllotmentMetadataRequest::new("window:origin[100]",-1));
        let window_origin = shapes.universe().make_request("window:origin[100]").unwrap(); // XXX
        shapes.use_allotment(&window_origin);
        let mut rectangles = vec![];
        if self.invert {
            /* top left of screen to bottom of screen, along lefthand edge of selection */
            let pos = self.area.tlbr().clone();
            rectangles.push(HoleySpaceBaseArea::Parametric(SpaceBaseArea::new(
                SpaceBase::new(vec![ParameterValue::Constant(0.)],
                                     vec![ParameterValue::Constant(0.)],vec![ParameterValue::Constant(0.)]),
                SpaceBase::new(vec![ParameterValue::Constant(0.)],
                                     vec![ParameterValue::Constant(-1.)],vec![ParameterValue::Variable(pos.1,16.)])
            )));
            let pos = self.area.tlbr().clone();
            /* top right of screen to bottom of screen, along righthand edge of selection */
            rectangles.push(HoleySpaceBaseArea::Parametric(SpaceBaseArea::new(
                SpaceBase::new(vec![ParameterValue::Constant(0.)],
                                     vec![ParameterValue::Constant(0.)],vec![ParameterValue::Variable(pos.3,0.)]),
                SpaceBase::new(vec![ParameterValue::Constant(1.)],
                                     vec![ParameterValue::Constant(-1.)],vec![ParameterValue::Constant(0.)])
            )));
            /* length of top of shape from top of screen to that shape */
            let pos = self.area.tlbr().clone();
            rectangles.push(HoleySpaceBaseArea::Parametric(SpaceBaseArea::new(
                SpaceBase::new(vec![ParameterValue::Constant(0.)],
                                     vec![ParameterValue::Constant(0.)],vec![ParameterValue::Variable(pos.1,0.)]),
                SpaceBase::new(vec![ParameterValue::Constant(0.)],
                                     vec![ParameterValue::Variable(pos.0,0.)],vec![ParameterValue::Variable(pos.3,16.)])
            )));
            /* length of bottom of shape from bottom of shape to bottom of screen */
            let pos = self.area.tlbr().clone();
            rectangles.push(HoleySpaceBaseArea::Parametric(SpaceBaseArea::new(
                SpaceBase::new(vec![ParameterValue::Constant(0.)],
                                     vec![ParameterValue::Variable(pos.2,0.)],vec![ParameterValue::Variable(pos.1,0.)]),
                SpaceBase::new(vec![ParameterValue::Constant(0.)],
                                     vec![ParameterValue::Constant(-1.)],vec![ParameterValue::Variable(pos.3,16.)])
            )));
        } else {
            let pos = self.area.tlbr().clone();
            rectangles.push(HoleySpaceBaseArea::Parametric(SpaceBaseArea::new(
                SpaceBase::new(vec![ParameterValue::Constant(0.)],
                                     vec![ParameterValue::Variable(pos.0,0.)],vec![ParameterValue::Variable(pos.1,0.)]),
                SpaceBase::new(vec![ParameterValue::Constant(0.)],
                                     vec![ParameterValue::Variable(pos.2,0.)],vec![ParameterValue::Variable(pos.3,16.)])
            )));
        }
        for area in rectangles.drain(..) {           
            let area2 = HoleySpaceBaseArea2::xxx_from_original(area,EachOrEvery::Every(window_origin.clone()));
            shapes.add_rectangle(area2,Patina::Drawn(DrawnType::Fill,EachOrEvery::Every(Colour::Direct(self.colour.clone()))));
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
