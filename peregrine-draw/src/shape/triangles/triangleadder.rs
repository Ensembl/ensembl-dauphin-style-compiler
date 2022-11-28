use peregrine_toolkit::error::Error;
use crate::{webgl::{AttribHandle, ProcessStanzaAddable, ProcessStanzaElements, UniformHandle, ProcessBuilder}};

#[derive(Clone)]
pub struct TriangleAdder {
    pub coords: AttribHandle,
    pub origin_coords: Option<AttribHandle>,
    pub run_coords: Option<AttribHandle>,
    pub depth: Option<AttribHandle>,
    pub use_vertical: Option<UniformHandle>
}

impl TriangleAdder {
    pub(crate) fn new(process: &mut ProcessBuilder) -> Result<TriangleAdder,Error> {
        let builder = process.program_builder();
        let is_vertical = process.geometry_name().is_vertical();
        let handle = builder.try_get_uniform_handle("uUseVertical");
        drop(builder);
        if let Some(use_vertical) = handle {
            let value = if is_vertical { 1. } else { 0. };
            process.set_uniform(&use_vertical,vec![value])?;
        }
        let builder = process.program_builder();
        Ok(TriangleAdder {
            coords: builder.get_attrib_handle("aCoords")?,
            origin_coords: builder.try_get_attrib_handle("aOriginCoords"),
            run_coords: builder.try_get_attrib_handle("aRunCoords"),
            depth: builder.try_get_attrib_handle("aDepth"),
            use_vertical: builder.try_get_uniform_handle("uUseVertical")
        })
    }

    pub(crate) fn add_data4(&self, elements: &mut ProcessStanzaElements, data: Vec<f32>, depths: Vec<f32>) -> Result<(),Error> {
        if let Some(depth) = &self.depth {
            elements.add(depth, depths, 1)?;
        }
        elements.add(&self.coords,data,4)?;
        Ok(())
    }

    pub(crate) fn add_origin_data4(&self, elements: &mut ProcessStanzaElements, data: Vec<f32>) -> Result<(),Error> {
        if let Some(origin_delta_handle) = &self.origin_coords {
            elements.add(origin_delta_handle,data,4)?;
        }
        Ok(())
    }

    pub(crate) fn add_run_data4(&self, elements: &mut ProcessStanzaElements, data: Vec<f32>) -> Result<(),Error> {
        if let Some(run_delta_handle) = &self.run_coords {
            elements.add(run_delta_handle,data,4)?;
        }
        Ok(())
    }

    pub(crate) fn set_use_vertical(&self, builder: &mut ProcessBuilder, value: f32) -> Result<(),Error> {
        if let Some(use_vertical) = &self.use_vertical {
            builder.set_uniform(use_vertical,vec![value])?;
        }
        Ok(())
    }
}
