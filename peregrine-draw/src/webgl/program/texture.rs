use crate::webgl::FlatId;
use crate::webgl::global::WebGlGlobal;
use super::uniform::UniformHandle;
use crate::util::message::Message;

pub(crate) struct TextureValues {
    uniform_handle: UniformHandle,
    flat_id: FlatId
}

impl TextureValues {
    pub(super) fn new(uniform_handle: &UniformHandle, canvas: &FlatId) -> Result<TextureValues,Message> {
        Ok(TextureValues { uniform_handle: uniform_handle.clone(), flat_id: canvas.clone() })
    }

    pub(super) fn apply(&self, gl: &mut WebGlGlobal) -> Result<(&UniformHandle,u32),Message> {
        let index = gl.bindery().allocate(&self.flat_id)?.apply(gl)?;
        Ok((&self.uniform_handle,index))
    }

    pub fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        gl.bindery().free(&self.flat_id)?.apply(gl)?;
        Ok(())
    }
}
