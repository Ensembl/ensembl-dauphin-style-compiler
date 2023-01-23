use std::{sync::{Arc, Mutex}, collections::HashMap, any::Any};
use eard_interp::{ ContextItem, HandleStore, InterpreterBuilder, Operation, RunContext };
use peregrine_data::{LeafRequest, ProgramShapesBuilder, Colour, Patina, SpaceBase, DataRequest, DataResponse, DataStore, LoadMode, RunReport, ShapeRequest, AccessorResolver, Plotter, Pen, SmallValuesStore};
use crate::{leaf::{op_leaf, op_leaf_s}, style::op_style, paint::{op_colour, op_paint_solid, op_paint_solid_s, op_graph_type, op_pen, op_paint_hollow, op_paint_hollow_s, op_paint_special, op_zmenu, op_paint_dotted, op_paint_metadata, op_paint_setting}, coord::op_coord, shape::{op_rectangle, op_wiggle, op_text, op_image, op_running_text, op_empty, op_running_rectangle}, data::{op_get_data, op_request, op_scope, op_data_boolean, op_data_number, op_data_string, op_bp_range, op_scope_s, op_small_value, op_only_warm, op_stick}, setting::{op_setting_boolean, op_setting_string, op_setting_number_seq, op_setting_number, op_setting_string_seq, op_setting_boolean_seq, op_setting_boolean_keys, op_setting_number_keys, op_setting_string_keys}};

#[derive(Clone)]
pub struct LibPeregrineBuilder {
    shapes: ContextItem<Arc<Mutex<Option<ProgramShapesBuilder>>>>,
    leafs: ContextItem<HandleStore<LeafRequest>>,
    colours: ContextItem<HandleStore<Colour>>,
    paint: ContextItem<HandleStore<Patina>>,
    coords: ContextItem<HandleStore<SpaceBase<f64,()>>>,
    requests: ContextItem<HandleStore<DataRequest>>,
    responses: ContextItem<HandleStore<DataResponse>>,
    graph_types: ContextItem<HandleStore<Plotter>>,
    pens: ContextItem<HandleStore<Pen>>,
    shape_request: ContextItem<ShapeRequest>,
    data_store: ContextItem<DataStore>,
    small_values_store: ContextItem<SmallValuesStore>,
    mode: ContextItem<LoadMode>,
    report: ContextItem<Arc<Mutex<RunReport>>>,
    resolver: ContextItem<AccessorResolver>
}

pub fn build_libperegrine(builder: &mut InterpreterBuilder) -> Result<LibPeregrineBuilder,String> {
    let leafs = builder.add_context::<HandleStore<LeafRequest>>("leaf")?;
    let colours = builder.add_context::<HandleStore<Colour>>("colours")?;
    let paint = builder.add_context::<HandleStore<Patina>>("paint")?;
    let shapes = builder.add_context::<Arc<Mutex<Option<ProgramShapesBuilder>>>>("shapes")?;
    let coords = builder.add_context::<HandleStore<SpaceBase<f64,()>>>("coords")?;
    let requests = builder.add_context::<HandleStore<DataRequest>>("requests")?;
    let responses = builder.add_context::<HandleStore<DataResponse>>("responses")?;
    let graph_types = builder.add_context::<HandleStore<Plotter>>("graph-types")?;
    let pens = builder.add_context::<HandleStore<Pen>>("pens")?;
    let shape_request = builder.add_context::<ShapeRequest>("shape-request")?;
    let data_store = builder.add_context::<DataStore>("data-store")?;
    let small_values_store = builder.add_context::<SmallValuesStore>("small-values-store")?;
    let mode = builder.add_context::<LoadMode>("mode")?;
    let report = builder.add_context::<Arc<Mutex<RunReport>>>("report")?;
    let resolver = builder.add_context::<AccessorResolver>("channel-resolver")?;
    builder.add_version("libperegrine",(0,0));
    builder.add_operation(256,Operation::new(op_leaf));
    builder.add_operation(257,Operation::new(op_leaf_s));
    builder.add_operation(258,Operation::new(op_style));
    builder.add_operation(259,Operation::new(op_colour));
    builder.add_operation(260,Operation::new(op_paint_solid));
    builder.add_operation(261,Operation::new(op_paint_solid_s));
    builder.add_operation(262,Operation::new(op_coord));
    builder.add_operation(263,Operation::new(op_rectangle));
    builder.add_operation(264,Operation::new(op_request));
    builder.add_operation(265,Operation::new(op_scope));
    builder.add_operation(266,Operation::new(op_get_data));
    builder.add_operation(267,Operation::new(op_data_boolean));
    builder.add_operation(268,Operation::new(op_data_number));
    builder.add_operation(269,Operation::new(op_data_string));
    builder.add_operation(270,Operation::new(op_graph_type));
    builder.add_operation(271,Operation::new(op_wiggle));
    builder.add_operation(272,Operation::new(op_setting_boolean));
    builder.add_operation(273,Operation::new(op_setting_number));
    builder.add_operation(274,Operation::new(op_setting_string));
    builder.add_operation(275,Operation::new(op_setting_boolean_seq));
    builder.add_operation(276,Operation::new(op_setting_number_seq));
    builder.add_operation(277,Operation::new(op_setting_string_seq));
    builder.add_operation(278,Operation::new(op_pen));
    builder.add_operation(279,Operation::new(op_text));
    builder.add_operation(280,Operation::new(op_paint_hollow));
    builder.add_operation(281,Operation::new(op_paint_hollow_s));
    builder.add_operation(282,Operation::new(op_bp_range));
    builder.add_operation(283,Operation::new(op_paint_special));
    builder.add_operation(284,Operation::new(op_image));
    builder.add_operation(285,Operation::new(op_running_text));
    builder.add_operation(286,Operation::new(op_zmenu));
    builder.add_operation(287,Operation::new(op_paint_dotted));
    builder.add_operation(288,Operation::new(op_empty));
    builder.add_operation(289,Operation::new(op_paint_metadata));
    builder.add_operation(290,Operation::new(op_paint_setting));
    builder.add_operation(291,Operation::new(op_setting_boolean_keys));
    builder.add_operation(292,Operation::new(op_setting_number_keys));
    builder.add_operation(293,Operation::new(op_setting_string_keys));
    builder.add_operation(294,Operation::new(op_scope_s));
    builder.add_operation(295,Operation::new(op_running_rectangle));
    builder.add_operation(296,Operation::new(op_small_value));
    builder.add_operation(297,Operation::new(op_only_warm));
    builder.add_operation(298,Operation::new(op_stick));
    Ok(LibPeregrineBuilder { 
        leafs, shapes, colours, paint, coords, requests, responses, data_store, mode, report,
        shape_request, resolver, graph_types, pens, small_values_store
    })
}

fn payload_get<'a,T: 'static>(payloads: &'a HashMap<String,Box<dyn Any>>, key: &str) -> Result<&'a T,String> {
    payloads.get(key).unwrap().downcast_ref::<T>().ok_or_else(||
        format!("missing payload key={}",key)
    )
}

pub fn prepare_libperegrine(context: &mut RunContext, builder: &LibPeregrineBuilder, data_store: &DataStore, small_values_store: &SmallValuesStore, payloads: HashMap<String,Box<dyn Any>>) -> Result<(),String> {
    let shapes = payload_get::<Arc<Mutex<Option<ProgramShapesBuilder>>>>(&payloads,"out")?.clone();
    let mode = payload_get::<LoadMode>(&payloads,"mode")?;
    let report = payload_get::<Arc<Mutex<RunReport>>>(&payloads,"report")?;
    let shape_request = payload_get::<ShapeRequest>(&payloads,"request")?;
    let resolver = payload_get::<AccessorResolver>(&payloads,"channel-resolver")?;
    context.add(&builder.leafs,HandleStore::new());
    context.add(&builder.colours,HandleStore::new());
    context.add(&builder.paint,HandleStore::new());
    context.add(&builder.coords,HandleStore::new());
    context.add(&builder.requests,HandleStore::new());
    context.add(&builder.responses,HandleStore::new());
    context.add(&builder.graph_types,HandleStore::new());
    context.add(&builder.pens,HandleStore::new());
    context.add(&builder.shapes,shapes);
    context.add(&builder.data_store,data_store.clone());
    context.add(&builder.small_values_store,small_values_store.clone());
    context.add(&builder.mode,mode.clone());
    context.add(&builder.report,report.clone());
    context.add(&builder.shape_request,shape_request.clone());
    context.add(&builder.resolver,resolver.clone());
    Ok(())
}
