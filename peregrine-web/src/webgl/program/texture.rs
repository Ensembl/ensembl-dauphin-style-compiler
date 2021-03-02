use crate::webgl::canvas::flatstore::FlatId;
use crate::webgl::global::WebGlGlobal;
use super::uniform::UniformHandle;

pub(crate) struct TextureValues {
    uniform_handle: UniformHandle,
    flat_id: FlatId
}

impl TextureValues {
    pub(super) fn new(uniform_handle: &UniformHandle, canvas: &FlatId) -> anyhow::Result<TextureValues> {
        Ok(TextureValues { uniform_handle: uniform_handle.clone(), flat_id: canvas.clone() })
    }

    pub(super) fn apply(&self, gl: &mut WebGlGlobal) -> anyhow::Result<(&UniformHandle,u32)> {
        let index = gl.bindery().allocate(&self.flat_id)?.apply(gl)?;
        Ok((&self.uniform_handle,index))
    }

    pub fn discard(&mut self, gl: &mut WebGlGlobal) -> anyhow::Result<()> {
        gl.bindery().free(&self.flat_id)?.apply(gl)?;
        Ok(())
    }
}
