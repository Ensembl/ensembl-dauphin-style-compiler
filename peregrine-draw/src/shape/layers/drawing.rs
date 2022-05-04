use std::sync::{Arc, Mutex};
use super::drawingtools::DrawingToolsBuilder;
use super::layer::Layer;
use peregrine_data::{Assets, Scale, Shape, LeafStyle };
use peregrine_toolkit::{lock};
use peregrine_toolkit::sync::needed::Needed;
use peregrine_toolkit::sync::retainer::RetainTest;
use super::super::core::prepareshape::{ prepare_shape_in_layer };
use super::super::core::drawshape::{ add_shape_to_layer, GLShape };
use crate::shape::core::drawshape::ShapeToAdd;
use crate::webgl::{ DrawingAllFlats, DrawingAllFlatsBuilder, DrawingSession, Process};
use crate::webgl::global::WebGlGlobal;
use super::drawingzmenus::{ DrawingHotspots, HotspotEntryDetails };
use crate::stage::stage::ReadStage;
use crate::util::message::Message;

pub(crate) trait DynamicShape {
    fn any_dynamic(&self) -> bool;
    fn recompute(&mut self) -> Result<(),Message>;
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
        Ok(DrawingBuilder {
            main_layer: Layer::new(gl_ref.program_store,left)?,
            tools: DrawingToolsBuilder::new(assets,scale,left),
            flats: None,
            dynamic_shapes: vec![]
        })
    }

    pub(crate) fn prepare_shape(&mut self, shape: &Shape<LeafStyle>) -> Result<Vec<GLShape>,Message> {
        let shape = shape.clone(); // XXX don't clone
        let (layer, tools) = (&mut self.main_layer,&mut self.tools);
        prepare_shape_in_layer(layer,tools,shape)
    }

    pub(crate) fn prepare_tools(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        let mut prep = self.tools.start_preparation(gl)?;
        let mut drawable = DrawingAllFlatsBuilder::new();
        prep.allocate(gl,&mut drawable)?;
        let gl_ref = gl.refs();
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
                self.tools.zmenus().add_rectangle(area,&hotspot);
            },
            ShapeToAdd::None => {}
        }
        Ok(())
    }

    pub(crate) async fn build(mut self, gl: &Arc<Mutex<WebGlGlobal>>, retain_test: &RetainTest) -> Result<Option<Drawing>,Message> {
        let flats = self.flats.take().unwrap().built();
        let processes = self.main_layer.build(gl,&flats,retain_test).await?;
        Ok(if let Some(processes) = processes {
            let tools = self.tools.build();
            Some(Drawing::new_real(processes,flats,tools.zmenus,self.dynamic_shapes)?)
        } else {
            None
        })
    }

    pub(crate) fn build_sync(mut self, gl: &Arc<Mutex<WebGlGlobal>>) -> Result<Drawing,Message> {
        let flats = self.flats.take().unwrap().built();
        let processes = self.main_layer.build_sync(gl,&flats)?;
        let tools = self.tools.build();
        Ok(Drawing::new_real(processes,flats,tools.zmenus,self.dynamic_shapes)?)
    }
}

struct DrawingData {
    processes: Vec<Process>,
    canvases: DrawingAllFlats,
    zmenus: DrawingHotspots,
    dynamic_shapes: Vec<Box<dyn DynamicShape>>,
    recompute: Needed
}

#[derive(Clone)]
pub(crate) struct Drawing(Arc<Mutex<DrawingData>>);

impl Drawing {
    pub(crate) async fn new(scale: Option<&Scale>, shapes: Arc<Vec<Shape<LeafStyle>>>, gl: &Arc<Mutex<WebGlGlobal>>, left: f64, assets: &Assets, retain_test: &RetainTest) -> Result<Option<Drawing>,Message> {
        /* convert core shape data model into gl shapes */
        let mut lgl = lock!(gl);
        let mut drawing = DrawingBuilder::new(scale,&mut lgl,assets,left)?;
        let mut prepared_shapes = shapes.iter().map(|s| drawing.prepare_shape(s)).collect::<Result<Vec<_>,_>>()?;
        /* gather and allocate aux requirements (2d canvas space etc) */
        drawing.prepare_tools(&mut lgl)?;
        /* draw shapes (including any 2d work) */
        for mut shapes in prepared_shapes.drain(..) {
            for shape in shapes.drain(..) {
                drawing.add_shape(&mut lgl,shape)?;
            }
        }
        drop(lgl);
        /* convert stuff to WebGL processes */
        drawing.build(gl,retain_test).await
    }

    pub(crate) fn new_sync(scale: Option<&Scale>, shapes: Vec<Shape<LeafStyle>>, gl: &Arc<Mutex<WebGlGlobal>>, left: f64, assets: &Assets) -> Result<Drawing,Message> {
        let mut lgl = lock!(gl);
        /* convert core shape data model into gl shapes */
        let mut drawing = DrawingBuilder::new(scale,&mut lgl,assets,left)?;
        let mut prepared_shapes = shapes.iter().map(|s| drawing.prepare_shape(s)).collect::<Result<Vec<_>,_>>()?;
        /* gather and allocate aux requirements (2d canvas space etc) */
        drawing.prepare_tools(&mut lgl)?;
        /* draw shapes (including any 2d work) */
        for mut shapes in prepared_shapes.drain(..) {
            for shape in shapes.drain(..) {
                drawing.add_shape(&mut lgl,shape)?;
            }
        }
        /* convert stuff to WebGL processes */
        drop(lgl);
        drawing.build_sync(gl)
    }

    fn new_real(processes: Vec<Process>, canvases: DrawingAllFlats, zmenus: DrawingHotspots, dynamic_shapes: Vec<Box<dyn DynamicShape>>) -> Result<Drawing,Message> {
        let mut out = Drawing(Arc::new(Mutex::new(DrawingData {
            processes,
            canvases,
            zmenus,
            dynamic_shapes,
            recompute: Needed::new()
        })));
        out.recompute()?;
        Ok(out)
    }

    pub(crate) fn set_zmenu_px_per_screen(&mut self, px_per_screen: f64) {
        lock!(self.0).zmenus.set_px_per_screen(px_per_screen);
    }

    pub(crate) fn get_hotspot(&self, stage: &ReadStage, position: (f64,f64)) -> Result<Vec<HotspotEntryDetails>,Message> {
        lock!(self.0).zmenus.get_hotspot(stage,position)
    }

    pub(crate) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &mut DrawingSession, opacity: f64) -> Result<(),Message> {
        let mut state = lock!(self.0);
        let recompute =  state.recompute.is_needed();
        for process in &mut state.processes {
            if recompute {
                process.update_attributes(gl)?;
            }
            session.run_process(gl,stage,process,opacity)?;
        }
        Ok(())
    }

    pub(crate) fn recompute(&mut self) -> Result<(),Message> {
        let mut state = lock!(self.0);
        let mut any = false;
        for shape in &mut state.dynamic_shapes {
            any |= shape.any_dynamic();
            shape.recompute()?;
        }
        if any {
            state.recompute.set();
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        let mut state = lock!(self.0);
        for process in &mut state.processes {
            process.discard(gl)?;
        }
        let gl = gl.refs();
        state.canvases.discard(gl.flat_store,gl.bindery)?;
        Ok(())
    }
}
