use std::marker;

use crate::{Message, webgl::{ global::WebGlGlobal}};

use super::packer::{allocate_areas, allocate_horizontal, allocate_vertical};

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
    pub(crate) fn pack(&self, sizes: &[(u32,u32)], gl: &WebGlGlobal) -> Result<(Vec<(u32,u32)>,u32,u32),Message> {
        let gpu_spec = gl.gpuspec();
        match self {
            CanvasWeave::HorizStack => allocate_horizontal(&sizes,gpu_spec),
            CanvasWeave::VertStack => allocate_vertical(&sizes,gpu_spec),
            _ =>  allocate_areas(&sizes,gl.gpuspec())
        }
    }
}
