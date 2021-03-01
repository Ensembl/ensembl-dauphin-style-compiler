use crate::webgl::canvas::weave::CanvasWeave;
use crate::util::keyed::KeyedData;
use crate::webgl::{ GPUSpec, ProtoProcess };
use anyhow::bail;
use super::flatstore::{ FlatId, FlatStore };
use super::flatplotallocator::FlatPlotRequestHandle;
use crate::webgl::global::WebGlGlobal;

struct DrawingFlat {
    id: FlatId,
    gl_index: u32
}

impl DrawingFlat {
    fn new(id: FlatId, gl_index: u32) -> DrawingFlat {
        DrawingFlat { id, gl_index }
    }

    fn add_process(&self, canvas_store: &mut FlatStore, process: &mut ProtoProcess) -> anyhow::Result<()> {
        process.add_texture(canvas_store,self.gl_index,&self.id)?;
        Ok(())
    }
}

pub struct DrawingFlats {
    main_canvases: Vec<DrawingFlat>,
    id_map: KeyedData<FlatId,Option<usize>>,
    max_textures: u32
}

impl DrawingFlats {
     fn new(gpu_spec: &GPUSpec) -> DrawingFlats {
        DrawingFlats {
            main_canvases: vec![],
            id_map: KeyedData::new(),
            max_textures: gpu_spec.max_textures()
        }
    }

    fn allocate(&mut self, gl: &mut WebGlGlobal, weave: &CanvasWeave, size: (u32,u32)) -> anyhow::Result<FlatId> {
        let document = gl.document().clone();
        let id = gl.canvas_store_mut().allocate(&document,weave,size)?;
        let gl_index = self.main_canvases.len();
        if gl_index as u32 > self.max_textures {
            bail!("too many textures!");
        }
        self.main_canvases.push(DrawingFlat::new(id.clone(),gl_index as u32));
        self.id_map.insert(&id,gl_index);
        Ok(id)
    }

    fn gl_index(&self, id: &FlatId) -> anyhow::Result<usize> {
        Ok(self.id_map.get(id).clone().unwrap())
    }

    pub(crate) fn add_process(&self, canvas_store: &mut FlatStore, process: &mut ProtoProcess) -> anyhow::Result<()> {
        for texture in &self.main_canvases {
            texture.add_process(canvas_store,process)?;
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self, store: &mut FlatStore) -> anyhow::Result<()> {
        for canvas in self.main_canvases.drain(..) {
            store.discard(&canvas.id)?;
        }
        Ok(())
    }
}

struct FlatPlotResponse {
    canvas: FlatId,
    origin: Vec<(u32,u32)>
}

pub struct DrawingFlatsDrawable {
    responses: KeyedData<FlatPlotRequestHandle,Option<FlatPlotResponse>>,
    drawing_flats: DrawingFlats
}

impl DrawingFlatsDrawable {
    pub(super) fn new(gpuspec: &GPUSpec) -> DrawingFlatsDrawable {
        DrawingFlatsDrawable {
            responses: KeyedData::new(),
            drawing_flats: DrawingFlats::new(gpuspec)
        }
    }

    pub(super) fn add(&mut self, id: FlatPlotRequestHandle, canvas: &FlatId, origin: Vec<(u32,u32)>) {
        self.responses.insert(&id,FlatPlotResponse {
            canvas: canvas.clone(),
            origin
        });
    }

    pub(super) fn make_canvas(&mut self, gl: &mut WebGlGlobal, weave: &CanvasWeave, size: (u32,u32)) -> anyhow::Result<FlatId> {
        self.drawing_flats.allocate(gl,weave,size)
    }

    pub(crate) fn gl_index(&self, id: &FlatId) -> anyhow::Result<usize> {
        self.drawing_flats.gl_index(id)
    }

    pub(crate) fn origins(&self, id: &FlatPlotRequestHandle) -> Vec<(u32,u32)> {
        self.responses.get(id).as_ref().map(|a| &a.origin).unwrap().to_vec()
    }

    pub(crate) fn canvas(&self, id: &FlatPlotRequestHandle) -> FlatId {
        self.responses.get(id).as_ref().map(|a| a.canvas.clone()).unwrap()
    }

    pub(crate) fn built(self) -> DrawingFlats { self.drawing_flats }
}
