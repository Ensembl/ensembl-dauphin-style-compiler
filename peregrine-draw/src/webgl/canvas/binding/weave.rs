use peregrine_toolkit::error::Error;
use crate::{webgl::{GPUSpec, canvas::composition::{areabuilder::CanvasItemAreaBuilder, packer::{allocate_areas, allocate_linear}}}};

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
    pub(crate) fn tessellate(&self, items: &mut [&mut CanvasItemAreaBuilder], gpu_spec: &GPUSpec) -> Result<Option<(u32,u32)>,Error> {
        Ok(match self {
            CanvasWeave::HorizStack => allocate_linear(items,gpu_spec,true)?,
            CanvasWeave::VertStack => allocate_linear(items,gpu_spec,false)?,
            _ =>  allocate_areas(items,gpu_spec)?
        })
    }

    pub(crate) fn force_size(&self, width: u32, height: u32) -> (Option<u32>,Option<u32>) {
        match self {
            CanvasWeave::VertStack => (Some(width),None),
            CanvasWeave::HorizStack => (None,Some(height)),
            _ => (None,None)
        }
    }

    pub(crate) fn round_up(&self) -> bool {
        match self {
            CanvasWeave::HorizStack | CanvasWeave::VertStack => false,
            _ => true
        }
    }
}
