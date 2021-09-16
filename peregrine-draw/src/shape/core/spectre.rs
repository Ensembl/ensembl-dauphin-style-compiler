use peregrine_data::{ AllotmentRequestBuilder, AllotmentPetitioner, AllotmentRequest, Colour, DirectColour, HoleySpaceBaseArea, ParameterValue, Patina, ShapeListBuilder, SpaceBase, SpaceBaseArea, Variable, VariableValues};
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

    pub(crate) fn draw(&self, shapes: &mut ShapeListBuilder, allotment_petitioner: &mut AllotmentPetitioner) -> Result<(),Message> {
        let window_origin = allotment_petitioner.add(AllotmentRequest::new(AllotmentRequestBuilder::new("window:origin-over"),0));
        let pos = self.area.tlbr().clone();
        shapes.add_allotment(&window_origin);
        let top_left = SpaceBase::new(
            vec![ParameterValue::Constant(0.)],
            vec![ParameterValue::Variable(pos.0,0.)],
            vec![ParameterValue::Variable(pos.1,0.)]
        );
        let bottom_right = SpaceBase::new(
            vec![ParameterValue::Constant(0.)],
            vec![ParameterValue::Variable(pos.2,16.)],
            vec![ParameterValue::Variable(pos.3,16.)]
        );
        let area = HoleySpaceBaseArea::Parametric(SpaceBaseArea::new(top_left,bottom_right));
        shapes.add_rectangle(area,Patina::Hollow(vec![Colour::Bar(DirectColour(255,255,255,0),self.colour.clone(),(self.length,self.length),self.prop)],self.width as u32,0),vec![window_origin]);
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
    
    pub(crate) fn draw(&self, shapes: &mut ShapeListBuilder, allotment_petitioner: &mut AllotmentPetitioner) -> Result<(),Message> {
        let window_origin = allotment_petitioner.add(AllotmentRequest::new(AllotmentRequestBuilder::new("window:origin"),-1));
        shapes.add_allotment(&window_origin);
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
            /* top right of screen to bottom of screen, allong righthand edge of selection */
            rectangles.push(HoleySpaceBaseArea::Parametric(SpaceBaseArea::new(
                SpaceBase::new(vec![ParameterValue::Constant(0.)],
                                     vec![ParameterValue::Constant(0.)],vec![ParameterValue::Variable(pos.3,0.)]),
                SpaceBase::new(vec![ParameterValue::Constant(0.)],
                                     vec![ParameterValue::Constant(-1.)],vec![ParameterValue::Constant(-1.)])
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
            shapes.add_rectangle(area,Patina::Filled(vec![Colour::Direct(self.colour.clone())],0),vec![window_origin.clone()]);
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
    pub(crate) fn draw(&self, shapes: &mut ShapeListBuilder, allotment_petitioner: &mut AllotmentPetitioner) -> Result<(),Message> {
        match self {
            Spectre::MarchingAnts(a) => a.draw(shapes,allotment_petitioner)?,
            Spectre::Stain(a) => a.draw(shapes,allotment_petitioner)?,
            Spectre::Compound(spectres) => {
                for spectre in spectres {
                    spectre.draw(shapes,allotment_petitioner)?;
                }
            }
        }
        Ok(())
    }
}
