use crate::{Message, webgl::{AttribHandle, ProcessStanzaAddable, ProcessStanzaElements, ProgramBuilder}};

#[derive(Clone)]
pub struct TrianglesProgramLink {
    pub base: AttribHandle,
    pub delta: AttribHandle,
    pub origin_base: Option<AttribHandle>,
    pub origin_delta: Option<AttribHandle>,
}

impl TrianglesProgramLink {
    pub(crate) fn new(builder: &ProgramBuilder) -> Result<TrianglesProgramLink,Message> {
        Ok(TrianglesProgramLink {
            base: builder.get_attrib_handle("aBase")?,
            delta: builder.get_attrib_handle("aDelta")?,
            origin_base: builder.try_get_attrib_handle("aOriginBase"),
            origin_delta: builder.try_get_attrib_handle("aOriginDelta")
        })
    }

    pub(super) fn add_data(&self, elements: &mut ProcessStanzaElements, base: Vec<f32>, delta: Vec<f32>) -> Result<(),Message> {
        elements.add(&self.delta,delta,2)?;
        elements.add(&self.base,base,2)?;
        Ok(())
    }
    pub(super) fn add_origin_data(&self, elements: &mut ProcessStanzaElements, origin_base: Vec<f32>, origin_delta: Vec<f32>) -> Result<(),Message> {
        if let Some(origin_base_handle) = &self.origin_base {
            elements.add(origin_base_handle,origin_base,2)?;
        }
        if let Some(origin_delta_handle) = &self.origin_delta {
            elements.add(origin_delta_handle,origin_delta,2)?;
        }
        Ok(())
    }
}
