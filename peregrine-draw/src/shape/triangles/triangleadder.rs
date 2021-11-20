use crate::{Message, webgl::{AttribHandle, ProcessStanzaAddable, ProcessStanzaElements, ProgramBuilder, UniformHandle}};

#[derive(Clone)]
pub struct TriangleAdder {
    pub coords: AttribHandle,
    pub origin_coords: Option<AttribHandle>,
    pub depth: Option<AttribHandle>
}

impl TriangleAdder {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<TriangleAdder,Message> {
        Ok(TriangleAdder {
            coords: builder.get_attrib_handle("aCoords")?,
            origin_coords: builder.try_get_attrib_handle("aOriginCoords"),
            depth: builder.try_get_attrib_handle("aDepth")
        })
    }

    pub(super) fn add_data4(&self, elements: &mut ProcessStanzaElements, data: Vec<f32>, depths: Vec<f32>) -> Result<(),Message> {
        if let Some(depth) = &self.depth {
            elements.add(depth, depths, 1)?;
        }
        elements.add(&self.coords,data,4)?;
        Ok(())
    }

    pub(super) fn add_origin_data4(&self, elements: &mut ProcessStanzaElements, data: Vec<f32>) -> Result<(),Message> {
        if let Some(origin_delta_handle) = &self.origin_coords {
            elements.add(origin_delta_handle,data,4)?;
        }
        Ok(())
    }
}
