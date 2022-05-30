use std::{sync::{Arc, Mutex}};
use peregrine_toolkit::{puzzle::{StaticAnswer}, lock};
use crate::{ShapeRequestGroup, DataMessage, CarriageExtent, shape::shape::{DrawingShape, UnplacedShape, AbstractShape} };
use super::{leaflist::LeafList, trainstate::{CarriageTrainStateSpec}};

struct AbstractCarriageBuilder {
    builder: Arc<LeafList>,
    shapes: Vec<UnplacedShape>,
    shape_request_group: Option<ShapeRequestGroup>,
}

impl AbstractCarriageBuilder {
    fn build(&mut self) -> Result<AbstractCarriageState,DataMessage> {
        let (prep,spec) = self.builder.position_boxes(self.shape_request_group.as_ref())?;
        /* update leafs to reflect container position */
        let shapes = self.shapes.iter().map(|x| 
                x.map_new_allotment(|r| prep.plm.transformable(r.name()).cloned())
            ).collect::<Vec<_>>();
        Ok(AbstractCarriageState { shapes, spec })
    }
}

struct AbstractCarriageState {
    shapes: Vec<AbstractShape>,
    spec: CarriageTrainStateSpec
}

enum LazyAbstractCarriage {
    Unready(AbstractCarriageBuilder),
    Ready(AbstractCarriageState)
}

impl LazyAbstractCarriage {
    fn ready(&mut self) -> Result<&mut AbstractCarriageState,DataMessage> {
        let built = match self {
            LazyAbstractCarriage::Unready(prep) => prep.build()?,
            LazyAbstractCarriage::Ready(ready) => { return Ok(ready); }
        };
        *self = LazyAbstractCarriage::Ready(built);
        if let LazyAbstractCarriage::Ready(ready) = self {
            return Ok(ready);
        } else {
            panic!("impossible error building carriage");
        }
    }
}

#[derive(Clone)]
pub struct AbstractCarriage(Arc<Mutex<LazyAbstractCarriage>>,Option<CarriageExtent>);

impl PartialEq for AbstractCarriage {
    fn eq(&self, other: &Self) -> bool {
        self.extent() == other.extent()
    }
}

impl Eq for AbstractCarriage {}

impl std::hash::Hash for AbstractCarriage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.extent().hash(state);
    }
}

impl AbstractCarriage {
    pub fn new(builder: Arc<LeafList>, shapes: Vec<UnplacedShape>, shape_request_group: Option<&ShapeRequestGroup>, extent: Option<&CarriageExtent>) -> AbstractCarriage {
        AbstractCarriage(Arc::new(Mutex::new(LazyAbstractCarriage::Unready(
            AbstractCarriageBuilder {
                builder, shapes,
                shape_request_group: shape_request_group.cloned()
            }
        ))),
        extent.cloned()
        )
    }

    pub(crate) fn extent(&self) -> Option<&CarriageExtent> { self.1.as_ref() }

    pub(crate) fn spec(&self) -> Result<CarriageTrainStateSpec,DataMessage> {
        Ok(lock!(self.0).ready()?.spec.clone())
    }

    pub fn make_drawing_shapes(&self, answer: &mut StaticAnswer) -> Result<Vec<DrawingShape>,DataMessage> {
        let mut out = vec![];
        for input in lock!(self.0).ready()?.shapes.iter() {
            out.append(&mut input.map_new_allotment(|x| x.make(answer)).make());
        }
        Ok(out)
    }
}
