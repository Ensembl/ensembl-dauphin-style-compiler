use std::{sync::{Arc, Mutex}};
use peregrine_toolkit::{puzzle::{StaticAnswer}, lock, timer_start, timer_end, error::Error, log };
use crate::{ShapeRequestGroup, CarriageExtent, shape::{shape::{DrawingShape, UnplacedShape, FloatingShape}, metadata::AbstractMetadataBuilder}, allotment::{core::{allotmentname::allotmentname_hashmap, boxtraits::ContainerOrLeaf}, leafs::anchored::AnchoredLeaf} };
use super::{leaflist::LeafList, trainstate::{CarriageTrainStateSpec}};

struct AbstractCarriageBuilder {
    builder: Arc<LeafList>,
    shapes: Vec<UnplacedShape>,
    shape_request_group: Option<ShapeRequestGroup>,
}

impl AbstractCarriageBuilder {
    fn build(&mut self) -> Result<AbstractCarriageState,Error> {
        /* Extract metadata */
        let mut metadata = AbstractMetadataBuilder::new();
        metadata.add_shapes(&self.shapes);
        let metadata = metadata.build();
        let (spec,plm) = self.builder.position_boxes(self.shape_request_group.as_ref(),&metadata)?;
        /* update leafs to reflect container position */
        let shapes = self.shapes.iter().map(|x| 
                x.map_new_allotment(|r| plm.floating_leaf(r.name()).clone())
            ).collect::<Vec<_>>();
        Ok(AbstractCarriageState { shapes, spec })
    }
}

struct AbstractCarriageState {
    shapes: Vec<FloatingShape>,
    spec: CarriageTrainStateSpec
}

enum LazyAbstractCarriage {
    Unready(AbstractCarriageBuilder),
    Ready(AbstractCarriageState)
}

impl LazyAbstractCarriage {
    fn ready(&mut self) -> Result<&mut AbstractCarriageState,Error> {
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

    pub(crate) fn spec(&self) -> Result<CarriageTrainStateSpec,Error> {
        Ok(lock!(self.0).ready()?.spec.clone())
    }

    pub fn make_drawing_shapes(&self, answer: &mut StaticAnswer) -> Result<Vec<DrawingShape>,Error> {
        let mut out = vec![];
        let mut transformer_cache = allotmentname_hashmap::<AnchoredLeaf>();
        for input in lock!(self.0).ready()?.shapes.iter() {
            timer_start!("make_drawing_shapes");
            let z = input.map_new_allotment(|x| {
                transformer_cache.entry(x.name().clone()).or_insert_with(|| {
                    x.anchor_leaf(answer).unwrap() // FloatingLeaf -> AnchoredLeaf
                }).clone()
            });
            out.append(&mut z.make()); //  AnchoredLeaf -> LeafStyle
            timer_end!("make_drawing_shapes");
        }
        Ok(out)
    }
}
