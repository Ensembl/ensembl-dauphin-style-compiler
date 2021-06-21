use std::sync::{ Arc };
use peregrine_data::{AllotmentPetitioner, AllotmentRequest, Colour, DirectColour, Patina, ShapeListBuilder, SpaceBase};
use crate::Message;

#[derive(Debug)]
struct MarchingAntsState {
    tlbr: (f64,f64,f64,f64)
}

#[derive(Clone,Debug)]
pub(crate) struct MarchingAnts(Arc<MarchingAntsState>);

impl MarchingAnts {
    pub(crate) fn new(tlbr: (f64,f64,f64,f64)) -> MarchingAnts {
        let tlbr = (
            tlbr.0.min(tlbr.2),
            tlbr.1.min(tlbr.3),
            tlbr.0.max(tlbr.2),
            tlbr.1.max(tlbr.3),
        );
        MarchingAnts(Arc::new(MarchingAntsState {
            tlbr
        }))
    }

    fn position(&self) -> (f64,f64,f64,f64) {
        self.0.tlbr
    }

    pub(crate) fn draw(&self, shapes: &mut ShapeListBuilder, allotment_petitioner: &mut AllotmentPetitioner) -> Result<(),Message> {
        let window_origin = allotment_petitioner.add(AllotmentRequest::new("window:origin",0));
        let pos = self.position();
        shapes.add_allotment(&window_origin);
        shapes.add_rectangle(SpaceBase::new(vec![0.],vec![pos.0],vec![pos.1]), 
                          SpaceBase::new(vec![0.],vec![pos.2],vec![pos.3]), 
                                      Patina::Filled(vec![Colour::Direct(DirectColour(0,255,0))]),
                            vec![window_origin]);
        Ok(())
    }
}

#[derive(Clone,Debug)]
pub(crate) enum Spectre {
    MarchingAnts(MarchingAnts)
}

impl Spectre {
    pub(crate) fn draw(&self, shapes: &mut ShapeListBuilder, allotment_petitioner: &mut AllotmentPetitioner) -> Result<(),Message> {
        match self {
            Spectre::MarchingAnts(a) => a.draw(shapes,allotment_petitioner)?
        }
        Ok(())
    }
}
