use peregrine_toolkit::error::Error;

use crate::{webgl::{GPUSpec}};

use super::tessellate::{packer::{allocate_areas, allocate_horizontal, allocate_vertical}, canvastessellator::CanvasTessellationPrepare};

#[allow(dead_code)]
#[derive(Clone,PartialEq,Eq,Hash,Debug)]
pub(crate) enum CanvasWeave {
    Crisp,
    Fuzzy,
    Heraldry,
    VertStack,
    HorizStack
}

impl CanvasWeave {
    pub(crate) fn tessellate(&self, prepare: &mut CanvasTessellationPrepare, gpu_spec: &GPUSpec) -> Result<(u32,u32),Error> {
        let (origins,width,height) = match self {
            CanvasWeave::HorizStack => allocate_horizontal(prepare.size(),gpu_spec),
            CanvasWeave::VertStack => allocate_vertical(prepare.size(),gpu_spec),
            _ =>  allocate_areas(&prepare.size(),gpu_spec)
        }?;
        for origin in &origins {
            prepare.add_origin(*origin);
        }
        let (x,y) = match self {
            CanvasWeave::VertStack => (Some(width),None),
            CanvasWeave::HorizStack => (None,Some(height)),
            _ => (None,None)
        };
        prepare.expand_to_canvas(x,y);
        Ok((width,height))
    }

    pub(crate) fn round_up(&self) -> bool {
        match self {
            CanvasWeave::HorizStack | CanvasWeave::VertStack => false,
            _ => true
        }
    }
}
