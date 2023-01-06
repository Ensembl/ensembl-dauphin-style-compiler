use std::{sync::{Arc, Mutex}, collections::HashMap, any::Any};
use eard_interp::{ ContextItem, HandleStore, InterpreterBuilder, Operation, RunContext };
use peregrine_data::{LeafRequest, ProgramShapesBuilder, Colour, Patina, SpaceBase};
use crate::{leaf::{op_leaf, op_leaf_s}, style::op_style, paint::{op_colour, op_paint_solid, op_paint_solid_s}, coord::op_coord, shape::op_rectangle};

#[derive(Clone)]
pub struct LibPeregrineBuilder {
    shapes: ContextItem<Arc<Mutex<Option<ProgramShapesBuilder>>>>,
    leafs: ContextItem<HandleStore<LeafRequest>>,
    colours: ContextItem<HandleStore<Colour>>,
    paint: ContextItem<HandleStore<Patina>>,
    coords: ContextItem<HandleStore<SpaceBase<f64,()>>>
}

pub fn build_libperegrine(builder: &mut InterpreterBuilder) -> Result<LibPeregrineBuilder,String> {
    let leafs = builder.add_context::<HandleStore<LeafRequest>>("leaf");
    let colours = builder.add_context::<HandleStore<Colour>>("colours");
    let paint = builder.add_context::<HandleStore<Patina>>("paint");
    let shapes = builder.add_context::<Arc<Mutex<Option<ProgramShapesBuilder>>>>("shapes");
    let coords = builder.add_context::<HandleStore<SpaceBase<f64,()>>>("coords");
    builder.add_version("libperegrine",(0,0));
    builder.add_operation(256,Operation::new(op_leaf));
    builder.add_operation(257,Operation::new(op_leaf_s));
    builder.add_operation(258,Operation::new(op_style));
    builder.add_operation(259,Operation::new(op_colour));
    builder.add_operation(260,Operation::new(op_paint_solid));
    builder.add_operation(261,Operation::new(op_paint_solid_s));
    builder.add_operation(262,Operation::new(op_coord));
    builder.add_operation(263,Operation::new(op_rectangle));
    Ok(LibPeregrineBuilder { leafs, shapes, colours, paint, coords })
}

fn payload_get<'a,T: 'static>(payloads: &'a HashMap<String,Box<dyn Any>>, key: &str) -> Result<&'a T,String> {
    payloads.get(key).unwrap().downcast_ref::<T>().ok_or_else(||
        format!("missing payload key={}",key)
    )
}

pub fn prepare_libperegrine(context: &mut RunContext, builder: &LibPeregrineBuilder, payloads: HashMap<String,Box<dyn Any>>) -> Result<(),String> {
    let shapes = payload_get::<Arc<Mutex<Option<ProgramShapesBuilder>>>>(&payloads,"out")?.clone();
    context.add(&builder.leafs,HandleStore::new());
    context.add(&builder.colours,HandleStore::new());
    context.add(&builder.paint,HandleStore::new());
    context.add(&builder.coords,HandleStore::new());
    context.add(&builder.shapes,shapes);
    Ok(())
}
