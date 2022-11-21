use keyed::KeyedData;
use peregrine_toolkit::error::Error;
use crate::webgl::{global::WebGlGlobal, CanvasWeave, CanvasInUse, DrawingCanvasesBuilder};
use keyed::keyed_handle;

keyed_handle!(TessellationGroupHandle);

struct Campaign {
    origin: Vec<(u32,u32)>,
    sizes: Vec<(u32,u32)>
}

pub(crate) struct CanvasTessellator {
    uniform_name: String,
    requests: KeyedData<TessellationGroupHandle,Option<Campaign>>,
    weave: CanvasWeave,
    canvas: Option<CanvasInUse>
}

impl CanvasTessellator {
    pub(crate) fn new(weave: &CanvasWeave, uniform_name: &str) -> CanvasTessellator {
        CanvasTessellator {
            uniform_name: uniform_name.to_string(),
            weave: weave.clone(),
            requests: KeyedData::new(),
            canvas: None
        }
    }

    pub(crate) fn insert(&mut self, sizes: &[(u32,u32)]) -> TessellationGroupHandle {
        self.requests.add(Some(Campaign {
            sizes: sizes.to_vec(), origin: vec![]
        }))
    }

    fn allocate(&mut self, gl: &mut WebGlGlobal, drawing_canvases: &mut DrawingCanvasesBuilder) -> Result<(),Error> {
        let mut sizes = vec![];
        let ids : Vec<_> = self.requests.keys().collect();
        for req_id in &ids {
            let req = self.requests.get(req_id).as_ref().unwrap();
            sizes.extend(req.sizes.iter());
        }
        if sizes.len() == 0 { return Ok(()); }
        let (mut origins,width,height) = self.weave.tessellate(&sizes,&gl.gpu_spec())?;
        let mut origins_iter = origins.drain(..);
        for req_id in &ids {
            let req = self.requests.get_mut(req_id).as_mut().unwrap();
            for size in req.sizes.iter_mut() {
                req.origin.push(origins_iter.next().unwrap());
                *size = self.weave.expand_size(size,&(width,height));
            }
        }
        let canvas = gl.canvas_source().make(&self.weave,(width,height))?;
        drawing_canvases.make_canvas(&canvas,&self.uniform_name);
        self.canvas = Some(canvas);
        Ok(())
    }

    pub(crate) fn origins(&self, id: &TessellationGroupHandle) -> Vec<(u32,u32)> {
        self.requests.get(id).as_ref().unwrap().origin.clone()
    }

    pub(crate) fn sizes(&self, id: &TessellationGroupHandle) -> Vec<(u32,u32)> {
        self.requests.get(id).as_ref().unwrap().sizes.clone()
    }

    pub(crate) fn canvas(&self) -> Result<Option<&CanvasInUse>,Error> { Ok(self.canvas.as_ref()) }

    pub(crate) fn make(&mut self, gl: &mut WebGlGlobal, drawable: &mut DrawingCanvasesBuilder) -> Result<(),Error> {
        self.allocate(gl,drawable)?;
        for (id,_) in self.requests.items() {
            if let Some(canvas) = &self.canvas {
                drawable.add(id,canvas);
            }
        }
        Ok(())
    }
}
