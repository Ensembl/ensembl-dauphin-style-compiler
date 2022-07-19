use peregrine_data::{StickId, Viewport, PlayingField};
use peregrine_toolkit::log;
use peregrine_toolkit_async::sync::needed::Needed;

use crate::{ webgl::{ SourceInstrs, UniformProto, GLArity, UniformHandle, ProgramBuilder, Process }};
use crate::shape::layers::consts::{ PR_DEF, PR_LOW };
use crate::util::message::Message;
use super::axis::{ StageAxis, ReadStageAxis };

#[derive(Clone)]
pub(crate) struct ProgramStage {
    hpos: UniformHandle,
    vpos: UniformHandle,
    bp_per_screen: UniformHandle,
    full_size: UniformHandle,
    size: UniformHandle,
    opacity: UniformHandle,
    model: UniformHandle,
    left_rail: UniformHandle,
}

impl ProgramStage {
    pub fn new(builder: &ProgramBuilder) -> Result<ProgramStage,Message> {
        Ok(ProgramStage {
            hpos: builder.get_uniform_handle("uStageHpos")?,
            vpos: builder.get_uniform_handle("uStageVpos")?,
            bp_per_screen: builder.get_uniform_handle("uStageZoom")?,
            size: builder.get_uniform_handle("uSize")?,
            full_size: builder.get_uniform_handle("uFullSize")?,
            opacity: builder.get_uniform_handle("uOpacity")?,
            model: builder.get_uniform_handle("uModel")?,
            left_rail: builder.get_uniform_handle("uLeftRail")?
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

    pub fn apply(&self, stage: &ReadStage, left: f64, opacity: f64, dpr: f64, process: &mut Process) -> Result<(),Message> {
        let mut position = stage.x.position()?;
        let mut bp_per_screen = stage.x.bp_per_screen()? as f64;
        /* allow for squeeze */
        let x_size = stage.x.drawable_size()?;
        let squeeze = stage.x.squeeze()?;
        let invisible_prop = (squeeze.0+squeeze.1) as f64/x_size;
        bp_per_screen /= 1.0-invisible_prop;
        position += (squeeze.1-squeeze.0) as f64/2.0/x_size*bp_per_screen;
        /**/
        process.set_uniform(&self.hpos,&[(position-left) as f32])?;
        process.set_uniform(&self.vpos,&[stage.y.position()? as f32])?;
        process.set_uniform(&self.bp_per_screen,&[2./bp_per_screen as f32])?;
        /* uSize gets drawable_size because it's later scaled by size/drawable_size */
        let size = (stage.x.drawable_size()?,stage.y.drawable_size()?);
        let full_size = (stage.x.container_size()?,stage.y.container_size()?);
        process.set_uniform(&self.size,&[(size.0/2.) as f32,(size.1/2.) as f32])?;
        process.set_uniform(&self.full_size,&[(full_size.0*dpr/2.) as f32,(full_size.1*dpr/2.) as f32])?;
        process.set_uniform(&self.opacity,&[opacity as f32])?;
        process.set_uniform(&self.model, &self.model_matrix(stage)?)?;
        process.set_uniform(&self.left_rail,&[(stage.x.squeeze()?.0/(size.0/2.) as f32)-1.])?;
        Ok(())
    }
}

// TODO greedy canvas size changes
pub struct Stage {
    stick: Option<StickId>,
    x: StageAxis,
    y: StageAxis,
    redraw_needed: Needed,
}

pub struct ReadStage {
    stick: Option<StickId>,
    x: Box<dyn ReadStageAxis>,
    y: Box<dyn ReadStageAxis>    
}

impl ReadStage {
    pub fn stick(&self) -> Option<&StickId> { self.stick.as_ref() }
    pub fn x(&self) -> &dyn ReadStageAxis { self.x.as_ref() }
    pub fn y(&self) -> &dyn ReadStageAxis { self.y.as_ref() }
    pub fn ready(&self) -> bool { self.x.ready() && self.y.ready() }
}

impl Clone for ReadStage {
    fn clone(&self) -> Self {
        ReadStage {
            stick: self.stick.clone(),
            x: Box::new(self.x.copy()),
            y: Box::new(self.y.copy())
        }
    }
}

impl Stage {
    pub fn new(redraw_needed: &Needed) -> Stage {
        let mut out = Stage {
            stick: None,
            x: StageAxis::new(&redraw_needed),
            y: StageAxis::new(&redraw_needed),
            redraw_needed: redraw_needed.clone()
        };
        out.y.set_position(0.);
        out.y.set_bp_per_screen(1.);
        out
    }

    pub fn ready(&self) -> bool { self.stick.is_some() && self.x.ready() && self.y.ready() }
    
    pub fn soon_stick(&mut self, stick: &StickId) {
        if let Some(old_stick) = &self.stick {
            if stick != old_stick {
                self.stick = None;
            }
        }
    }
    
    pub fn redraw_needed(&self) -> Needed { self.redraw_needed.clone() }

    pub fn x(&self) -> &StageAxis { &self.x }
    pub fn y(&self) -> &StageAxis { &self.y }
    pub fn x_mut(&mut self) -> &mut StageAxis { &mut self.x }
    pub fn y_mut(&mut self) -> &mut StageAxis { &mut self.y }

    pub fn notify_playingfield(&mut self, playing_field: &PlayingField) {
        let squeeze = playing_field.squeeze;
        self.x.set_squeeze((squeeze.0 as f32, squeeze.1 as f32));
    }

    pub fn notify_current(&mut self, viewport: &Viewport) {
        let position = viewport.position().unwrap();
        let bp_per_pixel = viewport.bp_per_screen().unwrap();        
        if let Ok(layout) = viewport.layout() {
            self.stick = Some(layout.stick().clone());
        }
        self.x_mut().set_position(position);
        self.x_mut().set_bp_per_screen(bp_per_pixel);
    }

    pub fn read_stage(&self) -> ReadStage {
        ReadStage {
            stick: self.stick.clone(),
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
        UniformProto::new_vertex(PR_DEF,GLArity::Scalar,"uLeftRail"),
        UniformProto::new_vertex(PR_DEF,GLArity::Vec2,"uFullSize"),
        UniformProto::new_fragment(PR_DEF,GLArity::Vec2,"uFullSize"),
        UniformProto::new_vertex(PR_DEF,GLArity::Vec2,"uSize"),
        UniformProto::new_vertex(PR_DEF,GLArity::Matrix4,"uModel"),
        UniformProto::new_fragment(PR_LOW,GLArity::Scalar,"uOpacity")
    ])
}
