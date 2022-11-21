use peregrine_toolkit::error::Error;

use crate::{webgl::{GPUSpec}};

use super::tessellate::packer::{allocate_areas, allocate_horizontal, allocate_vertical};

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
    pub(crate) fn tessellate(&self, sizes: &[(u32,u32)], gpu_spec: &GPUSpec) -> Result<(Vec<(u32,u32)>,u32,u32),Error> {
        match self {
            CanvasWeave::HorizStack => allocate_horizontal(&sizes,gpu_spec),
            CanvasWeave::VertStack => allocate_vertical(&sizes,gpu_spec),
            _ =>  allocate_areas(&sizes,gpu_spec)
        }
    }

    pub(crate) fn expand_size(&self, size: &(u32,u32), canvas_size: &(u32,u32)) -> (u32,u32) {
        let mut size = *size;
        match self {
            CanvasWeave::HorizStack => { size.1 = canvas_size.1 },
            CanvasWeave::VertStack => { size.0 = canvas_size.0 },
            _ =>  {}
        }
        size
    }

    pub(crate) fn round_up(&self) -> bool {
        match self {
            CanvasWeave::HorizStack | CanvasWeave::VertStack => false,
            _ => true
        }
    }
}
