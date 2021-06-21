use super::layer::Layer;
use peregrine_data::{ Shape, Allotter, ShapeList };
use super::super::core::prepareshape::{ prepare_shape_in_layer };
use super::super::core::drawshape::{ add_shape_to_layer, GLShape };
use crate::shape::core::heraldry::DrawingHeraldry;
use crate::webgl::canvas::flatplotallocator::FlatPositionManager;
//use crate::shape::core::heraldry::DrawingHeraldry;
use crate::webgl::{CanvasWeave, DrawingAllFlats, DrawingAllFlatsBuilder, DrawingSession, FlatStore, Process};
use super::super::core::text::DrawingText;
use crate::webgl::global::WebGlGlobal;
use super::drawingzmenus::{ DrawingZMenusBuilder, DrawingZMenus, ZMenuEvent };
use crate::stage::stage::ReadStage;
use crate::util::message::Message;

pub(crate) struct ToolPreparations {
    crisp: FlatPositionManager,
    heraldry_h: FlatPositionManager,
    heraldry_v: FlatPositionManager
}

impl ToolPreparations {
    fn new() -> ToolPreparations {
        ToolPreparations {
            crisp: FlatPositionManager::new(&CanvasWeave::Crisp,"uSampler"),
            heraldry_h: FlatPositionManager::new(&CanvasWeave::HorizStack,"uSampler"),
            heraldry_v: FlatPositionManager::new(&CanvasWeave::VertStack,"uSampler"),
        }
    }

    pub(crate) fn crisp_manager(&mut self) -> &mut FlatPositionManager { &mut self.crisp }
    pub(crate) fn heraldry_h_manager(&mut self) -> &mut FlatPositionManager { &mut self.heraldry_h }
    pub(crate) fn heraldry_v_manager(&mut self) -> &mut FlatPositionManager { &mut self.heraldry_v }

    fn allocate(&mut self, gl: &mut WebGlGlobal, drawable: &mut DrawingAllFlatsBuilder) -> Result<(),Message> {
        self.crisp.make(gl,drawable)?;
        self.heraldry_h.make(gl,drawable)?;
        self.heraldry_v.make(gl,drawable)?;
        Ok(())
    }
}

pub(crate) struct DrawingTools {
    text: DrawingText,
    heraldry: DrawingHeraldry,
    zmenus: DrawingZMenusBuilder
}

impl DrawingTools {
    fn new() -> DrawingTools {
        DrawingTools {
            text: DrawingText::new(),
            heraldry: DrawingHeraldry::new(),
            zmenus: DrawingZMenusBuilder::new()
        }
    }

    pub(crate) fn text(&mut self) -> &mut DrawingText { &mut self.text }
    pub(crate) fn heraldry(&mut self) -> &mut DrawingHeraldry { &mut self.heraldry }
    pub(crate) fn zmenus(&mut self) -> &mut DrawingZMenusBuilder { &mut self.zmenus }

    pub(crate) fn start_preparation(&mut self, gl: &mut WebGlGlobal) -> Result<ToolPreparations,Message> {
        let mut preparations = ToolPreparations::new();
        self.text.calculate_requirements(gl,&mut preparations.crisp)?;
        self.heraldry.calculate_requirements(gl,&mut preparations)?;
        Ok(preparations)
    }

    pub(crate) fn finish_preparation(&mut self, canvas_store: &mut FlatStore, mut preparations: ToolPreparations) -> Result<(),Message> {
        self.text.manager().draw_at_locations(canvas_store,&mut preparations.crisp)?;
        self.heraldry.draw_at_locations(canvas_store,&mut preparations)?;
        Ok(())
    }
}

pub(crate) struct DrawingBuilder {
    main_layer: Layer,
    tools: DrawingTools,
    flats: Option<DrawingAllFlatsBuilder>
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

    pub(crate) fn prepare_tools(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        let mut prep = self.tools.start_preparation(gl)?;
        let mut drawable = DrawingAllFlatsBuilder::new();
        prep.allocate(gl,&mut drawable)?;
        self.tools.finish_preparation(gl.canvas_store_mut(),prep)?;
        self.flats = Some(drawable);
        Ok(())
    }

    pub(crate) fn add_shape(&mut self, gl: &mut WebGlGlobal, shape: GLShape) -> Result<(),Message> {
        let (layer, tools,) = (&mut self.main_layer,&mut self.tools);
        add_shape_to_layer(layer,gl,tools,shape)
    }

    pub(crate) fn build(mut self, gl: &mut WebGlGlobal) -> Result<Drawing,Message> {
        let flats = self.flats.take().unwrap().built();
        let processes = self.main_layer.build(gl,&flats)?;
        Ok(Drawing::new_real(processes,flats,self.tools.zmenus.build())?)
    }
}

pub(crate) struct Drawing {
    processes: Vec<Process>,
    canvases: DrawingAllFlats,
    zmenus: DrawingZMenus
}

impl Drawing {
    pub(crate) fn new(shapes: ShapeList, gl: &mut WebGlGlobal, left: f64) -> Result<Drawing,Message> {
        /* convert core shape data model into gl shapes */
        let mut drawing = DrawingBuilder::new(gl,left)?;
        let allotter = shapes.allotter();
        let mut prepared_shapes = shapes.shapes().iter().map(|s| drawing.prepare_shape(s,&allotter)).collect::<Result<Vec<_>,_>>()?;
        /* gather and allocate aux requirements (2d canvas space etc) */
        drawing.prepare_tools(gl)?;
        /* draw shapes (including any 2d work) */
        for mut shapes in prepared_shapes.drain(..) {
            for shape in shapes.drain(..) {
                drawing.add_shape(gl,shape)?;
            }
        }
        /* convert stuff to WebGL processes */
        drawing.build(gl)
    }

    fn new_real(processes: Vec<Process>, canvases: DrawingAllFlats, zmenus: DrawingZMenus) -> Result<Drawing,Message> {
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
