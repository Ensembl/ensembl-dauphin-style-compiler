use std::{sync::{Arc, Mutex}};
use peregrine_toolkit::{puzzle::{StaticAnswer}, lock};

use crate::{allotment::{style::{style::LeafCommonStyle }, boxes::{root::{Root}, boxtraits::Transformable}, collision::{bumperfactory::BumperFactory}, util::bppxconverter::BpPxConverter}, ShapeRequestGroup, Shape, DataMessage, LeafRequest};

use super::{leafrequest::LeafTransformableMap, leaflist::LeafList, trainstate::{CarriageTrainStateRequest, CarriageTrainStateSpec}};

pub(crate) struct BoxPositionContext {
    //pub bump_requests: BumpRequests,
    pub bp_px_converter: Arc<BpPxConverter>,
    pub root: Root,
    pub plm: LeafTransformableMap,
    pub state_request: CarriageTrainStateRequest,
    pub bumper_factory: BumperFactory
}

impl BoxPositionContext {
    pub(crate) fn new(extent: Option<&ShapeRequestGroup>) -> BoxPositionContext {
        //let region = extent.map(|x| x.region().clone());
        BoxPositionContext {
            bp_px_converter: Arc::new(BpPxConverter::new(extent)),
            //bump_requests: BumpRequests::new(region.as_ref().map(|r| r.index()).unwrap_or(0) as usize),
            root: Root::new(),
            plm: LeafTransformableMap::new(),
            state_request: CarriageTrainStateRequest::new(),
            bumper_factory: BumperFactory::new()
        }
    }
}

struct CarriageOutputPrep {
    builder: Arc<LeafList>,
    shapes: Arc<Vec<Shape<LeafRequest>>>,
    extent: Option<ShapeRequestGroup>,
}

impl CarriageOutputPrep {
    fn build(&mut self) -> Result<CarriageOutputReady,DataMessage> {
        #[cfg(debug_trains)] debug_log!("position_boxes {:?}",self.extent.as_ref().map(|x| x.region()));
        let (prep,spec) = self.builder.position_boxes(self.extent.as_ref())?;
        /* update leafs to reflect container position */
        let shapes = self.shapes.iter().map(|x| 
                x.map_new_allotment(|r| prep.plm.transformable(r.name()).cloned())
            ).collect::<Vec<_>>();
        Ok(CarriageOutputReady {
            shapes: Arc::new(shapes),
            spec: Arc::new(spec)
        })
    }
}

#[derive(Clone)]
struct CarriageOutputReady {
    shapes: Arc<Vec<Shape<Arc<dyn Transformable>>>>,
    spec: Arc<CarriageTrainStateSpec>
}

enum CarriageOutputChoice {
    Unready(CarriageOutputPrep),
    Ready(CarriageOutputReady)
}

impl CarriageOutputChoice {
    fn ready(&mut self) -> Result<&mut CarriageOutputReady,DataMessage> {
        let built = match self {
            CarriageOutputChoice::Unready(prep) => prep.build()?,
            CarriageOutputChoice::Ready(ready) => { return Ok(ready); }
        };
        *self = CarriageOutputChoice::Ready(built);
        if let CarriageOutputChoice::Ready(ready) = self {
            return Ok(ready);
        } else {
            panic!("impossible error building carriage");
        }
    }
}

#[derive(Clone)]
pub struct CarriageOutput(Arc<Mutex<CarriageOutputChoice>>);

impl CarriageOutput {
    pub fn new(builder: Arc<LeafList>, shapes: Arc<Vec<Shape<LeafRequest>>>, extent: Option<&ShapeRequestGroup>) -> CarriageOutput {
        CarriageOutput(Arc::new(Mutex::new(CarriageOutputChoice::Unready(
            CarriageOutputPrep { 
                builder, shapes,
                extent: extent.cloned()
            }
        ))))
    }

    pub(crate) fn spec(&self) -> Result<CarriageTrainStateSpec,DataMessage> {
        Ok(lock!(self.0).ready()?.spec.as_ref().clone())
    }

    pub fn get(&self, answer_index: &mut StaticAnswer) -> Result<Vec<Shape<LeafCommonStyle>>,DataMessage> {
        let mut out = vec![];
        for input in lock!(self.0).ready()?.shapes.iter() {
            out.append(&mut input.map_new_allotment(|x| x.make(answer_index)).make());
        }
        Ok(out)
    }
}
