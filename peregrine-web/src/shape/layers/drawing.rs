use super::programstore::ProgramStore;
use super::layer::Layer;
use peregrine_core::Shape;
use super::super::core::glshape::{ prepare_shape_in_layer, add_shape_to_layer, PreparedShape };
use crate::webgl::{ Process, DrawingSession };
use super::super::canvas::text::DrawingText;
use super::super::canvas::allocator::DrawingCanvasesAllocator;
use super::super::canvas::weave::DrawingCanvasesBuilder;
use super::super::canvas::store::{ CanvasStore, DrawingCanvases };
use crate::webgl::global::WebGlGlobal;

pub(crate) struct Drawing {
    processes: Vec<Process>,
    canvases: DrawingCanvases
}

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

    pub(crate) fn finish_preparation(&mut self, allocator: &mut DrawingCanvasesAllocator) -> anyhow::Result<()> {
        self.text.populate_allocator(allocator)?;
        Ok(())
    }

    pub(crate) fn build(&mut self, canvas_store: &CanvasStore, builder: &mut DrawingCanvasesBuilder) -> anyhow::Result<()> {
        self.text.build(canvas_store,builder)?;
        Ok(())
    }
}

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

    pub(crate) fn add_shape(&mut self, shape: PreparedShape) -> anyhow::Result<()> {
        let (layer, tools) = (&mut self.main_layer,&mut self.tools);
        add_shape_to_layer(layer,tools,shape)
    }

    pub(crate) fn finish_preparation(&mut self, allocator: &mut DrawingCanvasesAllocator) -> anyhow::Result<()> {
        self.tools.finish_preparation(allocator)?;
        Ok(())
    }

    pub fn build(mut self, canvas_store: &CanvasStore, mut builder: DrawingCanvasesBuilder) -> anyhow::Result<Drawing> {
        let mut processes = vec![];
        self.main_layer.build(&mut processes)?;
        self.tools.build(canvas_store,&mut builder)?;
        Ok(Drawing {
            processes,
            canvases: builder.built()
        })
    }
}

impl Drawing {
    pub(crate) fn draw(&mut self, session: &DrawingSession, opacity: f64) -> anyhow::Result<()> {
        for process in &mut self.processes {
            session.run_process(process,opacity)?;
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self, canvas_store: &mut CanvasStore) -> anyhow::Result<()> {
        for process in &mut self.processes {
            process.discard()?;
        }
        self.canvases.discard(canvas_store);
        Ok(())
    }
}