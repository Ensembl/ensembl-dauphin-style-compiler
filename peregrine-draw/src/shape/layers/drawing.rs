use std::rc::Rc;
use super::layer::Layer;
use peregrine_data::{Assets, Scale, Shape, ShapeList, VariableValues, ZMenuProxy};
use peregrine_toolkit::sync::needed::Needed;
use super::super::core::prepareshape::{ prepare_shape_in_layer };
use super::super::core::drawshape::{ add_shape_to_layer, GLShape };
use crate::shape::core::bitmap::DrawingBitmap;
use crate::shape::core::drawshape::ShapeToAdd;
use crate::shape::heraldry::heraldry::DrawingHeraldry;
use crate::webgl::canvas::flatplotallocator::FlatPositionManager;
use crate::webgl::{CanvasWeave, DrawingAllFlats, DrawingAllFlatsBuilder, DrawingSession, FlatStore, Process};
use super::super::core::text::DrawingText;
use crate::webgl::global::WebGlGlobal;
use super::drawingzmenus::{ DrawingZMenusBuilder, DrawingZMenus };
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
    bitmap: DrawingBitmap,
    heraldry: DrawingHeraldry,
    zmenus: DrawingZMenusBuilder
}

impl DrawingTools {
    fn new(assets: &Assets, scale: Option<&Scale>, left: f64) -> DrawingTools {
        DrawingTools {
            text: DrawingText::new(),
            bitmap: DrawingBitmap::new(assets),
            heraldry: DrawingHeraldry::new(),
            zmenus: DrawingZMenusBuilder::new(scale, left)
        }
    }

    pub(crate) fn text(&mut self) -> &mut DrawingText { &mut self.text }
    pub(crate) fn bitmap(&mut self) -> &mut DrawingBitmap { &mut self.bitmap }
    pub(crate) fn heraldry(&mut self) -> &mut DrawingHeraldry { &mut self.heraldry }
    pub(crate) fn zmenus(&mut self) -> &mut DrawingZMenusBuilder { &mut self.zmenus }

    pub(crate) fn start_preparation(&mut self, gl: &mut WebGlGlobal) -> Result<ToolPreparations,Message> {
        let mut preparations = ToolPreparations::new();
        self.text.calculate_requirements(gl,&mut preparations.crisp)?;
        self.bitmap.calculate_requirements(gl, &mut preparations.crisp)?;
        self.heraldry.calculate_requirements(gl,&mut preparations)?;
        Ok(preparations)
    }

    pub(crate) fn finish_preparation(&mut self, canvas_store: &mut FlatStore, mut preparations: ToolPreparations) -> Result<(),Message> {
        self.text.manager().draw_at_locations(canvas_store,&mut preparations.crisp)?;
        self.bitmap.manager().draw_at_locations(canvas_store,&mut preparations.crisp)?;
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
    pub(crate) fn new(scale: Option<&Scale>, gl: &mut WebGlGlobal, assets: &Assets, variables: &VariableValues<f64>, left: f64) -> Result<DrawingBuilder,Message> {
        let gl_ref = gl.refs();
        Ok(DrawingBuilder {
            main_layer: Layer::new(gl_ref.program_store,left)?,
            tools: DrawingTools::new(assets,scale,left),
            flats: None,
            variables: variables.clone(),
            dynamic_shapes: vec![]
        })
    }

    pub(crate) fn prepare_shape(&mut self, shape: &Shape) -> Result<Vec<GLShape>,Message> {
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
            ShapeToAdd::ZMenu(area,allotments,zmenu,values) => {
                self.tools.zmenus.add_rectangle(area,allotments,zmenu,values);
            },
            ShapeToAdd::None => {}
        }
        Ok(())
    }

    pub(crate) fn build(mut self, gl: &mut WebGlGlobal) -> Result<Drawing,Message> {
        let flats = self.flats.take().unwrap().built();
        let processes = self.main_layer.build(gl,&flats)?;
        Ok(Drawing::new_real(processes,flats,self.tools.zmenus.build(),self.dynamic_shapes,&self.variables)?)
    }
}

pub(crate) struct Drawing {
    processes: Vec<Process>,
    canvases: DrawingAllFlats,
    variables: VariableValues<f64>,
    zmenus: DrawingZMenus,
    dynamic_shapes: Vec<Box<dyn DynamicShape>>,
    recompute: Needed
}

impl Drawing {
    pub(crate) fn new(scale: Option<&Scale>, shapes: ShapeList, gl: &mut WebGlGlobal, left: f64, variables: &VariableValues<f64>, assets: &Assets) -> Result<Drawing,Message> {
        /* convert core shape data model into gl shapes */
        let mut drawing = DrawingBuilder::new(scale,gl,assets,variables,left)?;
        let mut prepared_shapes = shapes.shapes().iter().map(|s| drawing.prepare_shape(s)).collect::<Result<Vec<_>,_>>()?;
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

    fn new_real(processes: Vec<Process>, canvases: DrawingAllFlats, zmenus: DrawingZMenus, dynamic_shapes: Vec<Box<dyn DynamicShape>>, variables: &VariableValues<f64>) -> Result<Drawing,Message> {
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

    pub(crate) fn set_zmenu_px_per_screen(&mut self, px_per_screen: f64) {
        self.zmenus.set_px_per_screen(px_per_screen);
    }

    pub(crate) fn get_hotspot(&self, stage: &ReadStage, position: (f64,f64)) -> Result<Vec<Rc<ZMenuProxy>>,Message> {
        self.zmenus.get_hotspot(stage,position)
    }

    pub(crate) fn draw(&mut self, gl: &mut WebGlGlobal, stage: &ReadStage, session: &mut DrawingSession, opacity: f64) -> Result<(),Message> {
        let recompute =  self.recompute.is_needed();
        for process in &mut self.processes {
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
        for process in &mut self.processes {
            process.discard(gl)?;
        }
        let gl = gl.refs();
        self.canvases.discard(gl.flat_store,gl.bindery)?;
        Ok(())
    }
}
