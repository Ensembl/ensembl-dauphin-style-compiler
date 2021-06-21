use std::sync::{ Arc };
use peregrine_data::{AllotmentPetitioner, AllotmentRequest, Colour, DirectColour, Patina, ShapeListBuilder, SpaceBase};
use crate::Message;
use std::cmp::min;

#[derive(Debug)]
struct BoundingBox {
    tlbr: (f64,f64,f64,f64)
}

#[derive(Clone,Debug)]
pub(crate) struct MarchingAnts(Arc<BoundingBox>);

impl MarchingAnts {
    pub(crate) fn new(tlbr: &(f64,f64,f64,f64)) -> MarchingAnts {
        let tlbr = (
            tlbr.0.min(tlbr.2),
            tlbr.1.min(tlbr.3),
            tlbr.0.max(tlbr.2),
            tlbr.1.max(tlbr.3),
        );
        MarchingAnts(Arc::new(BoundingBox {
            tlbr
        }))
    }

    pub(crate) fn draw(&self, shapes: &mut ShapeListBuilder, allotment_petitioner: &mut AllotmentPetitioner) -> Result<(),Message> {
        let window_origin = allotment_petitioner.add(AllotmentRequest::new("window:origin-over",0));
        let pos = self.0.tlbr;
        shapes.add_allotment(&window_origin);
        shapes.add_rectangle(SpaceBase::new(vec![0.],vec![pos.0],vec![pos.1]), 
                          SpaceBase::new(vec![0.],vec![pos.2],vec![pos.3]), 
                                      Patina::Hollow(vec![Colour::Bar(DirectColour(255,255,255,0),DirectColour(255,0,0,255),(4,2))]),
                            vec![window_origin]);
        Ok(())
    }
}


#[derive(Clone,Debug)]
pub(crate) struct Stain(Arc<BoundingBox>,bool);

impl Stain {
    pub(crate) fn new(tlbr: &(f64,f64,f64,f64),flipped: bool) -> Stain {
        let tlbr = (
            tlbr.0.min(tlbr.2),
            tlbr.1.min(tlbr.3),
            tlbr.0.max(tlbr.2),
            tlbr.1.max(tlbr.3),
        );
        Stain(Arc::new(BoundingBox {
            tlbr
        }),flipped)
    }

    pub(crate) fn draw(&self, shapes: &mut ShapeListBuilder, allotment_petitioner: &mut AllotmentPetitioner) -> Result<(),Message> {
        let window_origin = allotment_petitioner.add(AllotmentRequest::new("window:origin",-1));
        let pos = self.0.tlbr;
        shapes.add_allotment(&window_origin);
        if self.1 {
            shapes.add_rectangle(SpaceBase::new(vec![0.],vec![0.],vec![0.]),
                              SpaceBase::new(vec![0.],vec![-1.],vec![pos.1.min(pos.3)]),
                              Patina::Filled(vec![Colour::Direct(DirectColour(0,0,255,128))]),
                              vec![window_origin.clone()]);
            shapes.add_rectangle(SpaceBase::new(vec![0.],vec![-1.],vec![pos.1.max(pos.3)]),
                              SpaceBase::new(vec![0.],vec![0.],vec![-1.]),
                              Patina::Filled(vec![Colour::Direct(DirectColour(0,0,255,128))]),
                              vec![window_origin.clone()]);
            shapes.add_rectangle(SpaceBase::new(vec![0.],vec![0.],vec![pos.1.min(pos.3)]),
                              SpaceBase::new(vec![0.],vec![pos.0.min(pos.2)],vec![pos.1.max(pos.3)]),
                              Patina::Filled(vec![Colour::Direct(DirectColour(0,0,255,128))]),
                              vec![window_origin.clone()]);
            shapes.add_rectangle(SpaceBase::new(vec![0.],vec![pos.0.max(pos.2)],vec![pos.1.min(pos.3)]),
                              SpaceBase::new(vec![0.],vec![-1.],vec![pos.1.max(pos.3)]),
                              Patina::Filled(vec![Colour::Direct(DirectColour(0,0,255,128))]),
                              vec![window_origin.clone()]);

        } else {
            shapes.add_rectangle(SpaceBase::new(vec![0.],vec![pos.0],vec![pos.1]), 
                            SpaceBase::new(vec![0.],vec![pos.2],vec![pos.3]), 
                                        Patina::Filled(vec![Colour::Direct(DirectColour(0,0,255,128))]),
                                vec![window_origin]);
        }
        Ok(())
    }
}

#[derive(Clone,Debug)]
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
