use crate::webgl::canvas::weave::CanvasWeave;
use crate::util::keyed::KeyedData;
use crate::webgl::ProtoProcess;
use super::flatstore::{ FlatId, FlatStore };
use super::flatplotallocator::FlatPlotRequestHandle;
use crate::webgl::global::WebGlGlobal;

pub struct DrawingFlats {
    uniform_name: String,
    main_canvases: Vec<FlatId>,
}

impl DrawingFlats {
     fn new(uniform_name: &str) -> DrawingFlats {
        DrawingFlats {
            uniform_name: uniform_name.to_string(),
            main_canvases: vec![],
        }
    }

    fn allocate(&mut self, gl: &mut WebGlGlobal, weave: &CanvasWeave, size: (u32,u32)) -> anyhow::Result<FlatId> {
        let document = gl.document().clone();
        let id = gl.canvas_store_mut().allocate(&document,weave,size)?;
        self.main_canvases.push(id.clone());
        Ok(id)
    }

    pub(crate) fn add_process(&self, process: &mut ProtoProcess) -> anyhow::Result<()> {
        for id in &self.main_canvases {
            process.add_texture(&self.uniform_name,id)?;
        }
        Ok(())
    }

    pub(crate) fn discard(&mut self, store: &mut FlatStore) -> anyhow::Result<()> {
        for id in self.main_canvases.drain(..) {
            store.discard(&id)?;
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
    pub(super) fn new(uniform_name: &str) -> DrawingFlatsDrawable {
        DrawingFlatsDrawable {
            responses: KeyedData::new(),
            drawing_flats: DrawingFlats::new(uniform_name)
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

    pub(crate) fn origins(&self, id: &FlatPlotRequestHandle) -> Vec<(u32,u32)> {
        self.responses.get(id).as_ref().map(|a| &a.origin).unwrap().to_vec()
    }

    pub(crate) fn canvas(&self, id: &FlatPlotRequestHandle) -> FlatId {
        self.responses.get(id).as_ref().map(|a| a.canvas.clone()).unwrap()
    }

    pub(crate) fn built(self) -> DrawingFlats { self.drawing_flats }
}
