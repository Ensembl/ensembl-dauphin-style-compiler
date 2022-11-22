use peregrine_toolkit::error::Error;

use crate::{webgl::{GPUSpec}};

use super::tessellate::{packer::{allocate_areas, allocate_linear}, canvastessellator::CanvasTessellationPrepare};

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
        let sizes = prepare.items().iter().map(|item| item.size_with_padding()).collect::<Result<Vec<_>,_>>()?;
        let (width,height) = match self {
            CanvasWeave::HorizStack => allocate_linear(prepare,gpu_spec,true)?,
            CanvasWeave::VertStack => allocate_linear(prepare,gpu_spec,false)?,
            _ =>  {
                let (origins,width,height) = allocate_areas(&sizes,gpu_spec)?;
                for origin in &origins {
                    prepare.add_origin(*origin);
                }
                (width,height)
            }
        };
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
