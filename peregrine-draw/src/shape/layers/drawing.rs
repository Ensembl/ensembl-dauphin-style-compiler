use std::sync::{Arc, Mutex};
use super::drawingtools::DrawingToolsBuilder;
use super::layer::Layer;
use commander::cdr_tick;
use peregrine_data::{Assets, Scale, DrawingShape, DataMessage, HotspotGroupEntry, SingleHotspotEntry };
use peregrine_toolkit::error::Error;
use peregrine_toolkit::lock;
use peregrine_toolkit_async::sync::needed::Needed;
use peregrine_toolkit_async::sync::retainer::RetainTest;
use super::super::core::prepareshape::{ prepare_shape_in_layer };
use super::super::core::drawshape::{ add_shape_to_layer, GLShape };
use crate::shape::core::drawshape::ShapeToAdd;
use crate::webgl::{ DrawingAllFlats, DrawingAllFlatsBuilder, DrawingSession, Process};
use crate::webgl::global::WebGlGlobal;
use crate::stage::stage::ReadStage;
use crate::util::message::Message;
use crate::hotspots::drawinghotspots::DrawingHotspots;

#[cfg(debug_trains)]
use peregrine_toolkit::log_extra;

pub(crate) trait DynamicShape {
    fn any_dynamic(&self) -> bool;
    fn recompute(&mut self, gl: &WebGlGlobal) -> Result<(),Message>;
}

pub(crate) struct DrawingBuilder {
    main_layer: Layer,
    tools: DrawingToolsBuilder,
    flats: Option<DrawingAllFlatsBuilder>,
    dynamic_shapes: Vec<Box<dyn DynamicShape>>
}

impl DrawingBuilder {
    pub(crate) fn new(scale: Option<&Scale>, gl: &mut WebGlGlobal, assets: &Assets, left: f64) -> Result<DrawingBuilder,Message> {
        let gl_ref = gl.refs();
        let bitmap_multiplier = gl_ref.flat_store.bitmap_multiplier() as f64;
        Ok(DrawingBuilder {
            main_layer: Layer::new(gl_ref.program_store,left)?,
            tools: DrawingToolsBuilder::new(gl_ref.fonts,assets,scale,left,bitmap_multiplier),
            flats: None,
            dynamic_shapes: vec![]
        })
    }

    pub(crate) fn prepare_shape(&mut self, shape: &DrawingShape, gl: &mut WebGlGlobal) -> Result<Vec<GLShape>,Message> {
        let shape = shape.clone(); // XXX don't clone
        prepare_shape_in_layer(&mut self.tools,shape,gl)
    }

    pub(crate) async fn prepare_tools(&mut self, gl: &Arc<Mutex<WebGlGlobal>>) -> Result<(),Error> {
        let mut prep = self.tools.start_preparation(gl).await?;
        let mut drawable = DrawingAllFlatsBuilder::new();
        let mut lgl = lock!(gl);
        prep.allocate(&mut lgl,&mut drawable)?;
        let gl_ref = lgl.refs();
        self.tools.finish_preparation(gl_ref.flat_store,prep)?;
        self.flats = Some(drawable);
        Ok(())
    }

    pub(crate) fn add_shape(&mut self, gl: &mut WebGlGlobal, shape: GLShape) -> Result<(),Message> {
        let (layer, tools,) = (&mut self.main_layer,&mut self.tools);
        match add_shape_to_layer(layer,gl,tools,shape)? {
            ShapeToAdd::Dynamic(dynamic) => {
                self.dynamic_shapes.push(dynamic);
            },
            ShapeToAdd::Hotspot(area,hotspot) => {
                self.tools.hotspots().add_rectangle(HotspotGroupEntry::new(area,&hotspot));
            },
            ShapeToAdd::None => {}
        }
        Ok(())
    }

    pub(crate) async fn build(mut self, gl: &Arc<Mutex<WebGlGlobal>>, retain_test: &RetainTest) -> Result<Option<Drawing>,Message> {
        let flats = self.flats.take().unwrap().built();
        let processes = self.main_layer.build(gl,&flats,retain_test).await?;
        Ok(if let Some(processes) = processes {
            let tools = self.tools.build()?;
            Some(Drawing::new_real(processes,flats,tools.hotspots,self.dynamic_shapes,&*lock!(gl))?)
        } else {
            None
        })
    }
}

struct DrawingData {
    processes: Vec<Process>,
    canvases: DrawingAllFlats,
    hotspots: DrawingHotspots,
    dynamic_shapes: Vec<Box<dyn DynamicShape>>,
    recompute: Needed
}

#[derive(Clone)]
pub(crate) struct Drawing(Arc<Mutex<DrawingData>>);

impl Drawing {
    pub(crate) async fn new(scale: Option<&Scale>, shapes: Arc<Vec<DrawingShape>>, gl: &Arc<Mutex<WebGlGlobal>>, left: f64, assets: &Assets, retain_test: &RetainTest) -> Result<Option<Drawing>,Message> {
        /* convert core shape data model into gl shapes */
        let mut lgl = lock!(gl);
        let mut drawing = DrawingBuilder::new(scale,&mut lgl,assets,left)?;
        let mut prepared_shapes = shapes.iter().map(|s| drawing.prepare_shape(s,&mut lgl)).collect::<Result<Vec<_>,_>>()?;
        /* gather and allocate aux requirements (2d canvas space etc) */
        drop(lgl);
        drawing.prepare_tools(gl).await.map_err(|e| Message::DataError(DataMessage::XXXTransitional(e)))?;
        if !retain_test.test() {
            #[cfg(debug_trains)]
            log_extra!("drop discared after prepare");
            return Ok(None);
        }
        let mut lgl = lock!(gl);
        /* draw shapes (including any 2d work) */
        for mut shapes in prepared_shapes.drain(..) {
            for shape in shapes.drain(..) {
                drawing.add_shape(&mut lgl,shape)?;
            }
        }
        drop(lgl);
        cdr_tick(0).await;
        if !retain_test.test() {
            #[cfg(debug_trains)]
            log_extra!("drop discared after flat");
            return Ok(None);
        }
        /* convert stuff to WebGL processes */
        drawing.build(gl,retain_test).await
    }

    fn new_real(processes: Vec<Process>, canvases: DrawingAllFlats, hotspots: DrawingHotspots, dynamic_shapes: Vec<Box<dyn DynamicShape>>, gl: &WebGlGlobal) -> Result<Drawing,Message> {
        let mut out = Drawing(Arc::new(Mutex::new(DrawingData {
            processes,
            canvases,
            hotspots,
            dynamic_shapes,
            recompute: Needed::new()
        })));
        out.recompute(gl)?;
        Ok(out)
    }

    pub(crate) fn set_hotspot_px_per_screen(&mut self, px_per_screen: (f64,f64)) {
        lock!(self.0).hotspots.set_px_per_screen(px_per_screen);
    }

    pub(crate) fn get_hotspot(&self, stage: &ReadStage, position: (f64,f64)) -> Result<Vec<SingleHotspotEntry>,Message> {
        lock!(self.0).hotspots.get_hotspot(stage,position)
    }

    pub(crate) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &mut DrawingSession, opacity: f64) -> Result<(),Message> {
        let mut state = lock!(self.0);
        let recompute =  state.recompute.is_needed();
        for process in &mut state.processes {
            if recompute {
                process.update_attributes()?;
            }
            session.run_process(gl,stage,process,opacity)?;
        }
        Ok(())
    }

    pub(crate) fn recompute(&mut self, gl: &WebGlGlobal) -> Result<(),Message> {
        let mut state = lock!(self.0);
        let mut any = false;
        for shape in &mut state.dynamic_shapes {
            any |= shape.any_dynamic();
            shape.recompute(gl)?;
        }
        if any {
            state.recompute.set();
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Error> {
        let mut state = lock!(self.0);
        for process in &mut state.processes {
            process.discard(gl)?;
        }
        let gl = gl.refs();
        state.canvases.discard(gl.flat_store,gl.bindery)?;
        Ok(())
    }
}
