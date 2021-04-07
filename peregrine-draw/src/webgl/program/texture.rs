use crate::webgl::FlatId;
use crate::webgl::global::WebGlGlobal;
use super::uniform::UniformHandle;
use crate::util::message::Message;

pub(crate) struct TextureValues {
    uniform_handle: UniformHandle,
    flat_id: FlatId,
    bound: bool
}

impl TextureValues {
    pub(super) fn new(uniform_handle: &UniformHandle, canvas: &FlatId) -> Result<TextureValues,Message> {
        Ok(TextureValues { uniform_handle: uniform_handle.clone(), flat_id: canvas.clone(), bound: false })
    }

    pub(super) fn apply(&mut self, gl: &mut WebGlGlobal) -> Result<(&UniformHandle,u32),Message> {
        let index = gl.bindery().allocate(&self.flat_id)?.apply(gl)?;
        self.bound = true;
        Ok((&self.uniform_handle,index))
    }

    pub fn discard(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        if self.bound {
            gl.bindery().free(&self.flat_id)?.apply(gl)?;
        }
        Ok(())
    }
}
