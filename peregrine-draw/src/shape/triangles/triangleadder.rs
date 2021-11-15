use crate::{Message, webgl::{AttribHandle, ProcessStanzaAddable, ProcessStanzaElements, ProgramBuilder, UniformHandle}};

#[derive(Clone)]
pub struct TriangleAdder {
    pub coords: AttribHandle,
    pub origin_coords: Option<AttribHandle>,
    pub depth: AttribHandle,
    pub transform: Option<UniformHandle>
}

impl TriangleAdder {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<TriangleAdder,Message> {
        Ok(TriangleAdder {
            coords: builder.get_attrib_handle("aCoords")?,
            origin_coords: builder.try_get_attrib_handle("aOriginCoords"),
            depth: builder.get_attrib_handle("aDepth")?,
            transform: builder.try_get_uniform_handle("uTransform")
        })
    }

    pub(super) fn add_data4(&self, elements: &mut ProcessStanzaElements, data: Vec<f32>, depth: i8) -> Result<(),Message> {
        let gl_depth = 1.0 - (depth as f32+128.) / 255.;
        elements.add(&self.depth, vec![gl_depth;data.len()/4], 1)?;
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
