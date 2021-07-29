use std::collections::BTreeMap;
use super::layer::Layer;
use peregrine_data::{Allotter, Shape, ShapeList, VariableValues};
use peregrine_toolkit::sync::needed::Needed;
use super::super::core::prepareshape::{ prepare_shape_in_layer };
use super::super::core::drawshape::{ add_shape_to_layer, GLShape };
use crate::shape::heraldry::heraldry::DrawingHeraldry;
use crate::webgl::canvas::flatplotallocator::FlatPositionManager;
use crate::webgl::{CanvasWeave, DrawingAllFlats, DrawingAllFlatsBuilder, DrawingSession, FlatStore, Process};
use super::super::core::text::DrawingText;
use crate::webgl::global::WebGlGlobal;
use super::drawingzmenus::{ DrawingZMenusBuilder, DrawingZMenus, ZMenuEvent };
use crate::stage::stage::ReadStage;
use crate::util::message::Message;

pub(crate) trait DynamicShape {
    fn recompute(&mut self, variables: &VariableValues<f64>) -> Result<(),Message>;
}

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
    variables: VariableValues<f64>,
    flats: Option<DrawingAllFlatsBuilder>,
    dynamic_shapes: Vec<Box<dyn DynamicShape>>
}

impl DrawingBuilder {
    pub(crate) fn new(gl: &WebGlGlobal, variables: &VariableValues<f64>, left: f64) -> Result<DrawingBuilder,Message> {
        Ok(DrawingBuilder {
            main_layer: Layer::new(gl.program_store(),left)?,
            tools: DrawingTools::new(),
            flats: None,
            variables: variables.clone(),
            dynamic_shapes: vec![]
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
        self.tools.finish_preparation(gl.flat_store_mut(),prep)?;
        self.flats = Some(drawable);
        Ok(())
    }

    pub(crate) fn add_shape(&mut self, gl: &mut WebGlGlobal, shape: GLShape) -> Result<(),Message> {
        let (layer, tools,) = (&mut self.main_layer,&mut self.tools);
        let mut dynamic = add_shape_to_layer(layer,gl,tools,shape)?;
        self.dynamic_shapes.append(&mut dynamic);
        Ok(())
    }

    pub(crate) fn build(mut self, gl: &mut WebGlGlobal) -> Result<Drawing,Message> {
        let flats = self.flats.take().unwrap().built();
        let processes = self.main_layer.build(gl,&flats)?;
        Ok(Drawing::new_real(processes,flats,self.tools.zmenus.build(),self.dynamic_shapes,&self.variables)?)
    }
}

pub(crate) struct Drawing {
    processes: BTreeMap<i8,Vec<Process>>,
    canvases: DrawingAllFlats,
    variables: VariableValues<f64>,
    zmenus: DrawingZMenus,
    dynamic_shapes: Vec<Box<dyn DynamicShape>>,
    recompute: Needed
}

impl Drawing {
    pub(crate) fn new(shapes: ShapeList, gl: &mut WebGlGlobal, left: f64, variables: &VariableValues<f64>) -> Result<Drawing,Message> {
        /* convert core shape data model into gl shapes */
        let mut drawing = DrawingBuilder::new(gl,variables,left)?;
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

    pub fn priority_range(&self) -> (i8,i8) {
        let first = self.processes.iter().next().map(|x| *x.0);
        let last = self.processes.iter().rev().next().map(|x| *x.0);
        if let (Some(first),Some(last)) = (first,last) {
            (first,last)
        } else {
            (0,0)
        }
    }

    fn new_real(mut processes_in: Vec<(Process,i8)>, canvases: DrawingAllFlats, zmenus: DrawingZMenus, dynamic_shapes: Vec<Box<dyn DynamicShape>>, variables: &VariableValues<f64>) -> Result<Drawing,Message> {
        let mut processes = BTreeMap::new();
        for (proc,prio) in processes_in.drain(..) {
            processes.entry(prio).or_insert_with(|| vec![]).push(proc);
        }
        let mut out = Drawing {
            processes,
            canvases,
            zmenus,
            variables: variables.clone(),
            dynamic_shapes,
            recompute: Needed::new()
        };
        out.recompute()?;
        Ok(out)
    }

    pub(crate) fn intersects(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<Option<ZMenuEvent>,Message> {
        self.zmenus.intersects(stage,mouse)
    }

    pub(crate) fn intersects_fast(&self, stage: &ReadStage, mouse: (u32,u32)) -> Result<bool,Message> {
        self.zmenus.intersects_fast(stage,mouse)
    }

    pub(crate) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &DrawingSession, opacity: f64, priority: i8) -> Result<(),Message> {
        let recompute =  self.recompute.is_needed();
        for process in self.processes.get_mut(&priority).unwrap_or(&mut vec![]) {
            if recompute {
                process.update_attributes(gl)?;
            }
            session.run_process(gl,stage,process,opacity)?;
        }
        Ok(())
    }

    pub(crate) fn recompute(&mut self) -> Result<(),Message> {
        for shape in &mut self.dynamic_shapes {
            shape.recompute(&self.variables)?;
        }
        self.recompute.set();
        Ok(())
    }

    pub(crate) fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        for processes in self.processes.values_mut() {
            for process in processes {
                process.discard(gl)?;
            }
        }
        let gl = gl.refs();
        self.canvases.discard(gl.flat_store,gl.bindery)?;
        Ok(())
    }
}
