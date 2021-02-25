use std::collections::HashMap;
use crate::shape::canvas::weave::CanvasWeave;
use crate::util::keyed::{ KeyedOptionalValues };
use web_sys::{ Document };
use super::flat::CanvasElement;
use crate::keyed_handle;
use crate::webgl::{ GPUSpec, Process, ProtoProcess };
use anyhow::bail;

// TODO discard webgl buffers etc
// TODO document etc to common data structure

pub struct CanvasStore {
    document: Document, // XXX elevate
    scratch: HashMap<CanvasWeave,CanvasElement>,
    main_canvases: KeyedOptionalValues<CanvasElementId,CanvasElement>
}

impl CanvasStore {
    pub(crate) fn new(document: &Document) -> CanvasStore {
        CanvasStore {
            document: document.clone(),
            scratch: HashMap::new(),
            main_canvases: KeyedOptionalValues::new()
        }
    }

    pub(crate) fn get_scratch_context(&mut self, weave: &CanvasWeave, size: (u32,u32)) -> anyhow::Result<&mut CanvasElement> {
        let mut use_cached = false;
        if let Some(existing) = self.scratch.get(weave) {
            let ex_size = existing.size();
            if ex_size.0 >= size.0 && ex_size.1 >= size.1 {
                use_cached = true;
            }
        }
        if !use_cached {
            let canvas = CanvasElement::new(&self.document,&CanvasWeave::Crisp,size)?;
            self.scratch.insert(weave.clone(),canvas);
        }
        Ok(self.scratch.get_mut(weave).unwrap())
    }

    fn allocate_main(&mut self, weave: &CanvasWeave, size: (u32,u32)) -> anyhow::Result<CanvasElementId> {
        Ok(self.main_canvases.add(CanvasElement::new(&self.document,weave,size)?))
    }

    pub(crate) fn get_main_canvas(&self, id: &CanvasElementId) -> anyhow::Result<&CanvasElement> {
        self.main_canvases.get(id)
    }

    fn discard(&mut self, id: &CanvasElementId) -> anyhow::Result<()> {
        self.main_canvases.get_mut(id)?.discard()?;
        self.main_canvases.remove(id);
        Ok(())
    }

    pub(crate) fn discard_all(&mut self) -> anyhow::Result<()> {
        for canvas in self.main_canvases.values_mut() {
            canvas.discard()?;
        }
        self.main_canvases = KeyedOptionalValues::new();
        for (_,mut canvas) in self.scratch.drain() {
            canvas.discard()?;
        }
        Ok(())
    }
}

impl Drop for CanvasStore {
    fn drop(&mut self) {
        self.discard_all();
    }
}

pub struct DrawingCanvas {
    id: CanvasElementId,
    gl_index: u32
}

impl DrawingCanvas {
    fn new(id: CanvasElementId, gl_index: u32) -> DrawingCanvas {
        DrawingCanvas { id, gl_index }
    }

    fn add_process(&self, canvas_store: &mut CanvasStore, process: &mut ProtoProcess) -> anyhow::Result<()> {
        process.add_texture(canvas_store,self.gl_index,&self.id)?;
        Ok(())
    }
}

keyed_handle!(CanvasElementId);

pub struct DrawingCanvases {
    main_canvases: Vec<DrawingCanvas>,
    max_textures: u32
}

impl DrawingCanvases {
    pub(crate) fn new(gpu_spec: &GPUSpec) -> DrawingCanvases {
        DrawingCanvases {
            main_canvases: vec![],
            max_textures: gpu_spec.max_textures()
        }
    }

    pub(crate) fn allocate_main(&mut self, store: &mut CanvasStore, weave: &CanvasWeave, size: (u32,u32)) -> anyhow::Result<CanvasElementId> {
        let id = store.allocate_main(weave,size)?;
        let gl_index = self.main_canvases.len() as u32;
        if gl_index > self.max_textures {
            bail!("too many textures!");
        }
        self.main_canvases.push(DrawingCanvas::new(id.clone(),gl_index));
        Ok(id)
    }

    pub(crate) fn add_process(&self, canvas_store: &mut CanvasStore, process: &mut ProtoProcess) -> anyhow::Result<()> {
        for texture in &self.main_canvases {
            texture.add_process(canvas_store,process)?;
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self, store: &mut CanvasStore) -> anyhow::Result<()> {
        for canvas in self.main_canvases.drain(..) {
            store.discard(&canvas.id)?;
        }
        Ok(())
    }
}
