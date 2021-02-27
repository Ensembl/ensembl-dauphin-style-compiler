use super::programstore::ProgramStore;
use super::layer::Layer;
use peregrine_core::Shape;
use super::super::core::glshape::{ prepare_shape_in_layer, add_shape_to_layer, PreparedShape };
use crate::webgl::{ Process, DrawingSession };
use super::super::core::text::DrawingText;
use crate::webgl::canvas::flatplotallocator::FlatPlotAllocator;
use crate::webgl::canvas::drawingflats::{ DrawingFlats, DrawingFlatsDrawable };
use crate::webgl::canvas::flatstore::FlatStore;

pub(crate) struct DrawingTools {
    text: DrawingText
}

impl DrawingTools {
    fn new() -> DrawingTools {
        DrawingTools {
            text: DrawingText::new()
        }
    }

    pub(crate) fn text(&mut self) -> &mut DrawingText { &mut self.text }

    pub(crate) fn finish_preparation(&mut self,  canvas_store: &mut FlatStore, allocator: &mut FlatPlotAllocator) -> anyhow::Result<()> {
        self.text.populate_allocator(canvas_store,allocator)?;
        Ok(())
    }

    pub(crate) fn build(&mut self, canvas_store: &FlatStore, builder: &mut DrawingFlatsDrawable) -> anyhow::Result<()> {
        self.text.build(canvas_store,builder)?;
        Ok(())
    }
}

// TODO canvas builder to tools?
pub(crate) struct DrawingBuilder {
    main_layer: Layer,
    tools: DrawingTools
}

impl DrawingBuilder {
    pub(crate) fn new(programs: &ProgramStore, left: f64) -> DrawingBuilder {
        DrawingBuilder {
            main_layer: Layer::new(programs,left),
            tools: DrawingTools::new()
        }
    }

    pub(crate) fn prepare_shape(&mut self, shape: Shape) -> anyhow::Result<PreparedShape> {
        let (layer, tools) = (&mut self.main_layer,&mut self.tools);
        prepare_shape_in_layer(layer,tools,shape)
    }

    pub(crate) fn add_shape(&mut self, canvas_builder: &DrawingFlatsDrawable, shape: PreparedShape) -> anyhow::Result<()> {
        let (layer, tools) = (&mut self.main_layer,&mut self.tools);
        add_shape_to_layer(layer,tools,canvas_builder,shape)
    }

    pub(crate) fn finish_preparation(&mut self,  canvas_store: &mut FlatStore, allocator: &mut FlatPlotAllocator) -> anyhow::Result<()> {
        self.tools.finish_preparation(canvas_store,allocator)?;
        Ok(())
    }

    pub fn build(mut self, canvas_store: &mut FlatStore, mut builder: DrawingFlatsDrawable) -> anyhow::Result<Drawing> {
        self.tools.build(canvas_store,&mut builder)?;
        let canvases = builder.built();
        let mut processes = vec![];
        self.main_layer.build(canvas_store,&mut processes,&canvases)?;
        Ok(Drawing::new(processes,canvases)?)
    }
}

pub(crate) struct Drawing {
    processes: Vec<Process>,
    canvases: DrawingFlats
}

impl Drawing {
    fn new(processes: Vec<Process>, canvases: DrawingFlats) -> anyhow::Result<Drawing> {        
        Ok(Drawing {
            processes,
            canvases
        })
    }

    pub(crate) fn draw(&mut self, session: &DrawingSession, opacity: f64) -> anyhow::Result<()> {
        for process in &mut self.processes {
            session.run_process(process,opacity)?;
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self, canvas_store: &mut FlatStore) -> anyhow::Result<()> {
        for process in &mut self.processes {
            process.discard()?;
        }
        self.canvases.discard(canvas_store)?;
        Ok(())
    }
}