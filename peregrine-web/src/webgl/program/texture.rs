use crate::webgl::canvas::flatstore::FlatId;
use crate::webgl::global::WebGlGlobal;

pub(crate) struct TextureValues {
    flat_id: FlatId
}

impl TextureValues {
    pub(super) fn new(canvas: &FlatId) -> anyhow::Result<TextureValues> {
        Ok(TextureValues { flat_id: canvas.clone() })
    }

    pub(super) fn activate(&self, gl: &mut WebGlGlobal) -> anyhow::Result<()> {
        gl.bindery().allocate(&self.flat_id)?.apply(gl)?;
        Ok(())
    }

    pub fn discard(&mut self, gl: &mut WebGlGlobal) -> anyhow::Result<()> {
        gl.bindery().free(&self.flat_id)?.apply(gl)?;
        Ok(())
    }
}
