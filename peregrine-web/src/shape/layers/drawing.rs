use super::layer::Layer;
use peregrine_core::Shape;
use super::super::core::glshape::{ prepare_shape_in_layer, add_shape_to_layer, PreparedShape };
use crate::webgl::{ Process, DrawingFlatsDrawable, DrawingSession, FlatStore, FlatPlotAllocator, DrawingFlats };
use super::super::core::text::DrawingText;
use crate::webgl::global::WebGlGlobal;
use super::drawingzmenus::{ DrawingZMenusBuilder, DrawingZMenus };
use crate::shape::core::stage::ReadStage;

pub(crate) struct DrawingTools {
    text: DrawingText,
    zmenus: DrawingZMenusBuilder
}

impl DrawingTools {
    fn new() -> DrawingTools {
        DrawingTools {
            text: DrawingText::new(),
            zmenus: DrawingZMenusBuilder::new()
        }
    }

    pub(crate) fn text(&mut self) -> &mut DrawingText { &mut self.text }
    pub(crate) fn zmenus(&mut self) -> &mut DrawingZMenusBuilder { &mut self.zmenus }

    pub(crate) fn finish_preparation(&mut self, gl: &mut WebGlGlobal, allocator: &mut FlatPlotAllocator) -> anyhow::Result<()> {
        self.text.populate_allocator(gl,allocator)?;
        Ok(())
    }

    pub(crate) fn build(&mut self, canvas_store: &FlatStore, builder: &mut DrawingFlatsDrawable) -> anyhow::Result<()> {
        self.text.build(canvas_store,builder)?;
        Ok(())
    }
}

pub(crate) struct DrawingBuilder {
    main_layer: Layer,
    tools: DrawingTools,
    flats: Option<DrawingFlatsDrawable>
}

impl DrawingBuilder {
    pub(crate) fn new(gl: &WebGlGlobal, left: f64) -> DrawingBuilder {
        DrawingBuilder {
            main_layer: Layer::new(gl.program_store(),left),
            tools: DrawingTools::new(),
            flats: None
        }
    }

    pub(crate) fn prepare_shape(&mut self, shape: Shape) -> anyhow::Result<PreparedShape> {
        let (layer, tools) = (&mut self.main_layer,&mut self.tools);
        prepare_shape_in_layer(layer,tools,shape)
    }

    pub(crate) fn finish_preparation(&mut self, gl: &mut WebGlGlobal) -> anyhow::Result<()> {
        let mut canvas_allocator = FlatPlotAllocator::new("uSampler");
        self.tools.finish_preparation(gl,&mut canvas_allocator)?;
        self.flats = Some(canvas_allocator.make(gl)?);
        Ok(())
    }

    pub(crate) fn add_shape(&mut self, gl: &mut WebGlGlobal, shape: PreparedShape) -> anyhow::Result<()> {
        let (layer, tools, canvas_builder) = (&mut self.main_layer,&mut self.tools,&self.flats.as_ref().unwrap());
        add_shape_to_layer(layer,tools,canvas_builder,gl.bindery(),shape)
    }

    pub fn build(mut self, gl: &mut WebGlGlobal) -> anyhow::Result<Drawing> {
        let (tools, mut builder) = (&mut self.tools, self.flats.take().unwrap());
        tools.build(gl.canvas_store_mut(),&mut builder)?;
        let canvases = builder.built();
        let mut processes = vec![];
        self.main_layer.build(&mut processes,&canvases)?;
        Ok(Drawing::new(processes,canvases,self.tools.zmenus.build())?)
    }
}

pub(crate) struct Drawing {
    processes: Vec<Process>,
    canvases: DrawingFlats,
    zmenus: DrawingZMenus
}

impl Drawing {
    fn new(processes: Vec<Process>, canvases: DrawingFlats, zmenus: DrawingZMenus) -> anyhow::Result<Drawing> {
        Ok(Drawing {
            processes,
            canvases,
            zmenus
        })
    }

    pub(crate) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &DrawingSession, opacity: f64) -> anyhow::Result<()> {
        for process in &mut self.processes {
            session.run_process(gl,stage,process,opacity)?;
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self, gl: &mut WebGlGlobal) -> anyhow::Result<()> {
        for process in &mut self.processes {
            process.discard(gl)?;
        }
        self.canvases.discard(gl.canvas_store_mut())?;
        Ok(())
    }
}