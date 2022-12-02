use std::{sync::{Arc, Mutex}};
use peregrine_toolkit::{puzzle::{StaticAnswer}, lock, timer_start, timer_end, error::Error };
use crate::{ShapeRequestGroup, CarriageExtent, shape::{shape::{FloatingShape}, metadata::AbstractMetadataBuilder}, allotment::{core::{allotmentname::allotmentname_hashmap}, leafs::anchored::AnchoredLeaf, layout::stylebuilder::ContainerOrLeaf}, Shape, LeafRequest, AuxLeaf };
use super::{leafrequestsource::LeafRequestSource, trainstate::{CarriageTrainStateSpec}};

struct FloatingCarriageBuilder {
    builder: Arc<LeafRequestSource>,
    shapes: Vec<Shape<LeafRequest>>,
    shape_request_group: Option<ShapeRequestGroup>,
}

impl FloatingCarriageBuilder {
    fn build(&mut self) -> Result<FloatingCarriageState,Error> {
        /* Extract metadata */
        let mut metadata = AbstractMetadataBuilder::new();
        metadata.add_shapes(&self.shapes);
        let metadata = metadata.build();
        let (spec,shapes) = self.builder.to_floating_shapes(&self.shapes,self.shape_request_group.as_ref(),&metadata)?;
        Ok(FloatingCarriageState { shapes, spec })
    }
}

struct FloatingCarriageState {
    shapes: Vec<FloatingShape>,
    spec: CarriageTrainStateSpec
}

enum LazyFloatingCarriage {
    Unready(FloatingCarriageBuilder),
    Ready(FloatingCarriageState)
}

impl LazyFloatingCarriage {
    fn ready(&mut self) -> Result<&mut FloatingCarriageState,Error> {
        let built = match self {
            LazyFloatingCarriage::Unready(prep) => prep.build()?,
            LazyFloatingCarriage::Ready(ready) => { return Ok(ready); }
        };
        *self = LazyFloatingCarriage::Ready(built);
        if let LazyFloatingCarriage::Ready(ready) = self {
            return Ok(ready);
        } else {
            panic!("impossible error building carriage");
        }
    }
}

#[derive(Clone)]
pub struct FloatingCarriage(Arc<Mutex<LazyFloatingCarriage>>,Option<CarriageExtent>);

impl PartialEq for FloatingCarriage {
    fn eq(&self, other: &Self) -> bool {
        self.extent() == other.extent()
    }
}

impl Eq for FloatingCarriage {}

impl std::hash::Hash for FloatingCarriage {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.extent().hash(state);
    }
}

impl FloatingCarriage {
    pub(crate) fn new(builder: Arc<LeafRequestSource>, shapes: Vec<Shape<LeafRequest>>, shape_request_group: Option<&ShapeRequestGroup>, extent: Option<&CarriageExtent>) -> FloatingCarriage {
        FloatingCarriage(Arc::new(Mutex::new(LazyFloatingCarriage::Unready(
            FloatingCarriageBuilder {
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

    pub fn unfloat_shapes(&self, answer: &mut StaticAnswer) -> Result<Vec<Shape<AuxLeaf>>,Error> {
        let mut out = vec![];
        let mut transformer_cache = allotmentname_hashmap::<AnchoredLeaf>();
        for input in lock!(self.0).ready()?.shapes.iter() {
            timer_start!("make_drawing_shapes");
            let z = input.map_new_allotment(|x| {
                transformer_cache.entry(x.name().clone()).or_insert_with(|| {
                    x.anchor_leaf(answer).unwrap() // FloatingLeaf -> AnchoredLeaf
                }).clone()
            });
            out.append(&mut z.make()); //  AnchoredLeaf -> AuxLeaf
            timer_end!("make_drawing_shapes");
        }
        Ok(out)
    }
}
