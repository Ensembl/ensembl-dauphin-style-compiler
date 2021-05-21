use peregrine_data::Viewport;

use crate::webgl::{ SourceInstrs, UniformProto, GLArity, UniformHandle, ProgramBuilder, Process };
use crate::shape::layers::consts::{ PR_DEF, PR_LOW };
use crate::shape::core::redrawneeded::RedrawNeeded;
use crate::util::message::Message;
use super::axis::{ StageAxis, ReadStageAxis };

#[derive(Clone)]
pub(crate) struct ProgramStage {
    hpos: UniformHandle,
    vpos: UniformHandle,
    bp_per_screen: UniformHandle,
    size: UniformHandle,
    opacity: UniformHandle,
    model: UniformHandle
}

impl ProgramStage {
    pub fn new(builder: &ProgramBuilder) -> Result<ProgramStage,Message> {
        Ok(ProgramStage {
            hpos: builder.get_uniform_handle("uStageHpos")?,
            vpos: builder.get_uniform_handle("uStageVpos")?,
            bp_per_screen: builder.get_uniform_handle("uStageZoom")?,
            size: builder.get_uniform_handle("uSize")?,
            opacity: builder.get_uniform_handle("uOpacity")?,
            model: builder.get_uniform_handle("uModel")?
        })
    }

    fn model_matrix(&self, stage: &ReadStage) -> Result<Vec<f32>,Message> {
        let x = stage.x.scale_shift()?;
        let y = stage.y.scale_shift()?;
        Ok(vec![
            x.0, 0.,  0., 0.,
            0.,  y.0, 0., 0.,
            0.,  0.,  1., 0.,
            x.1,-y.1, 0., 1.
        ])
    }

    pub fn apply(&self, stage: &ReadStage, left: f64, opacity: f64, process: &mut Process) -> Result<(),Message> {
        /*
        use web_sys::console;
        let size = (stage.x.size()?,stage.y.size()?);
        console::log_1(&format!("
            hpos={:?} vpos={:?} 2./bp_per_screen={:?} size={:?} opacity={:?} model={:?}
        ",
        vec![stage.x.position()?-left],
        vec![stage.y.position()?],
        vec![2./stage.x.bp_per_screen()?],
        vec![size.0/2.,size.1/2.],
        vec![opacity],
        self.model_matrix(stage)).into());  
        */
        process.set_uniform(&self.hpos,&[(stage.x.position()?-left) as f32])?;
        process.set_uniform(&self.vpos,&[stage.y.position()? as f32])?;
        process.set_uniform(&self.bp_per_screen,&[2./stage.x.bp_per_screen()? as f32])?;
        /* uSize gets drawable_size because it's later scaled by size/drawable_size */
        let size = (stage.x.drawable_size()?,stage.y.drawable_size()?);
        process.set_uniform(&self.size,&[(size.0/2.) as f32,(size.1/2.) as f32])?;
        process.set_uniform(&self.opacity,&[opacity as f32])?;
        process.set_uniform(&self.model, &self.model_matrix(stage)?)?;
        Ok(())
    }
}

// TODO greedy canvas size changes
pub struct Stage {
    x: StageAxis,
    y: StageAxis,
    redraw_needed: RedrawNeeded,
}

pub struct ReadStage {
    x: Box<dyn ReadStageAxis>,
    y: Box<dyn ReadStageAxis>    
}

impl ReadStage {
    pub fn x(&self) -> &dyn ReadStageAxis { self.x.as_ref() }
    pub fn y(&self) -> &dyn ReadStageAxis { self.y.as_ref() }
    pub fn ready(&self) -> bool { self.x.ready() && self.y.ready() }
}

impl Clone for ReadStage {
    fn clone(&self) -> Self {
        ReadStage {
            x: Box::new(self.x.copy()),
            y: Box::new(self.y.copy())
        }
    }
}

impl Stage {
    pub fn new() -> Stage {
        let redraw_needed = RedrawNeeded::new();
        let mut out = Stage {
            x: StageAxis::new(&redraw_needed),
            y: StageAxis::new(&redraw_needed),
            redraw_needed
        };
        out.y.set_position(0.);
        out.y.set_bp_per_screen(1.);
        out
    }

    pub fn ready(&self) -> bool { self.x.ready() && self.y.ready() }

    pub fn redraw_needed(&self) -> RedrawNeeded { self.redraw_needed.clone() }

    pub fn x(&self) -> &StageAxis { &self.x }
    pub fn y(&self) -> &StageAxis { &self.y }
    pub fn x_mut(&mut self) -> &mut StageAxis { &mut self.x }
    pub fn y_mut(&mut self) -> &mut StageAxis { &mut self.y }

    pub fn notify_current(&mut self, viewport: &Viewport) {
        let position = viewport.position().unwrap();
        let bp_per_pixel = viewport.bp_per_screen().unwrap();        
        self.x_mut().set_position(position);
        self.x_mut().set_bp_per_screen(bp_per_pixel);
    }

    pub fn read_stage(&self) -> ReadStage {
        ReadStage {
            x: Box::new(self.x.copy()),
            y: Box::new(self.y.copy())
        }
    }
}

pub(crate) fn get_stage_source() -> SourceInstrs {
    SourceInstrs::new(vec![
        UniformProto::new_vertex(PR_DEF,GLArity::Scalar,"uStageHpos"),
        UniformProto::new_vertex(PR_DEF,GLArity::Scalar,"uStageVpos"),
        UniformProto::new_vertex(PR_DEF,GLArity::Scalar,"uStageZoom"),
        UniformProto::new_vertex(PR_DEF,GLArity::Vec2,"uSize"),
        UniformProto::new_vertex(PR_DEF,GLArity::Matrix4,"uModel"),
        UniformProto::new_fragment(PR_LOW,GLArity::Scalar,"uOpacity")
    ])
}
