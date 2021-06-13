use super::layer::Layer;
use peregrine_data::{ Shape, Allotter, ShapeList };
use super::super::core::prepareshape::{ prepare_shape_in_layer };
use super::super::core::drawshape::{ add_shape_to_layer, GLShape };
use crate::webgl::canvas::flatplotallocator::FlatPositionAllocator;
//use crate::shape::core::heraldry::DrawingHeraldry;
use crate::webgl::{CanvasWeave, DrawingFlats, DrawingFlatsDrawable, DrawingSession, FlatStore, Process};
use super::super::core::text::DrawingText;
use crate::webgl::global::WebGlGlobal;
use super::drawingzmenus::{ DrawingZMenusBuilder, DrawingZMenus, ZMenuEvent };
use crate::stage::stage::ReadStage;
use crate::util::message::Message;

pub(crate) struct ToolPreparations {
    crisp: FlatPositionAllocator
}

impl ToolPreparations {
    fn new() -> ToolPreparations {
        ToolPreparations {
            crisp: FlatPositionAllocator::new(&CanvasWeave::Crisp,"uSampler")
        }
    }

    fn draw(&mut self, gl: &mut WebGlGlobal, drawable: &mut DrawingFlatsDrawable) -> Result<(),Message> {
        self.crisp.make(gl,drawable)?;
        Ok(())
    }
}

pub(crate) struct DrawingTools {
    text: DrawingText,
    //heraldry: DrawingHeraldry,
    zmenus: DrawingZMenusBuilder
}

impl DrawingTools {
    fn new() -> DrawingTools {
        DrawingTools {
            text: DrawingText::new(),
            //heraldry: DrawingHeraldry::new(),
            zmenus: DrawingZMenusBuilder::new()
        }
    }

    pub(crate) fn text(&mut self) -> &mut DrawingText { &mut self.text }
    //pub(crate) fn heraldry(&mut self) -> &mut DrawingHeraldry { &mut self.heraldry }
    pub(crate) fn zmenus(&mut self) -> &mut DrawingZMenusBuilder { &mut self.zmenus }

    pub(crate) fn start_preparation(&mut self, gl: &mut WebGlGlobal) -> Result<ToolPreparations,Message> {
        let mut preparations = ToolPreparations::new();
        self.text.start_preparation(gl,&mut preparations.crisp,"uSampler")?;
        Ok(preparations)
    }

    pub(crate) fn finish_preparation(&mut self, canvas_store: &mut FlatStore, builder: &DrawingFlatsDrawable, preparations: ToolPreparations) -> Result<(),Message> {
        self.text.finish_preparation(canvas_store,builder)?;
        Ok(())
    }
}

pub(crate) struct DrawingBuilder {
    main_layer: Layer,
    tools: DrawingTools,
    flats: Option<DrawingFlatsDrawable>
}

impl DrawingBuilder {
    pub(crate) fn new(gl: &WebGlGlobal, left: f64) -> Result<DrawingBuilder,Message> {
        Ok(DrawingBuilder {
            main_layer: Layer::new(gl.program_store(),left)?,
            tools: DrawingTools::new(),
            flats: None
        })
    }

    pub(crate) fn prepare_shape(&mut self, shape: &Shape, allotter: &Allotter) -> Result<Vec<GLShape>,Message> {
        let shape = shape.clone(); // XXX don't clone
        let (layer, tools) = (&mut self.main_layer,&mut self.tools);
        prepare_shape_in_layer(layer,tools,shape,allotter)
    }

    pub(crate) fn finish_preparation(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        let mut prep = self.tools.start_preparation(gl)?;
        let mut drawable = DrawingFlatsDrawable::new();
        prep.draw(gl,&mut drawable)?;
        self.tools.finish_preparation(gl.canvas_store_mut(),&drawable,prep)?;
        self.flats = Some(drawable);
        Ok(())
    }

    pub(crate) fn add_shape(&mut self, gl: &mut WebGlGlobal, shape: GLShape) -> Result<(),Message> {
        let (layer, tools, canvas_builder) = (&mut self.main_layer,&mut self.tools,&self.flats.as_ref().unwrap());
        add_shape_to_layer(layer,gl,tools,canvas_builder,shape)
    }

    pub(crate) fn build(mut self, gl: &mut WebGlGlobal) -> Result<Drawing,Message> {
        let (_tools, mut flats_builder) = (&mut self.tools, self.flats.take().unwrap());
        let flats = flats_builder.built();
        let mut processes = vec![];
        self.main_layer.build(gl,&mut processes,&flats)?;
        Ok(Drawing::new_real(processes,flats,self.tools.zmenus.build())?)
    }
}

pub(crate) struct Drawing {
    processes: Vec<Process>,
    canvases: DrawingFlats,
    zmenus: DrawingZMenus
}

impl Drawing {
    pub(crate) fn new(shapes: ShapeList, gl: &mut WebGlGlobal, left: f64) -> Result<Drawing,Message> {
        let mut drawing = DrawingBuilder::new(gl,left)?;
        let allotter = shapes.allotter();
        let mut preparations =shapes.shapes().iter().map(|s| drawing.prepare_shape(s,&allotter)).collect::<Result<Vec<_>,_>>()?;
        drawing.finish_preparation(gl)?;
        for mut shapes in preparations.drain(..) {
            for shape in shapes.drain(..) {
                drawing.add_shape(gl,shape)?;
            }
        }
        drawing.build(gl)    
    }

    fn new_real(processes: Vec<Process>, canvases: DrawingFlats, zmenus: DrawingZMenus) -> Result<Drawing,Message> {
        Ok(Drawing {
            processes,
            canvases,
            zmenus
        })
    }

    pub(crate) fn intersects(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<Option<ZMenuEvent>,Message> {
        self.zmenus.intersects(stage,mouse)
    }

    pub(crate) fn intersects_fast(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<bool,Message> {
        self.zmenus.intersects_fast(stage,mouse)
    }

    pub(crate) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &DrawingSession, opacity: f64) -> Result<(),Message> {
        for process in &mut self.processes {
            session.run_process(gl,stage,process,opacity)?;
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        for process in &mut self.processes {
            process.discard(gl)?;
        }
        self.canvases.discard(gl.canvas_store_mut())?;
        Ok(())
    }
}