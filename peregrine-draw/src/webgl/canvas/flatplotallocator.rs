use keyed::KeyedData;
use crate::webgl::{FlatId };
use super::weave::{ CanvasWeave };
use super::drawingflats::{ DrawingAllFlatsBuilder };
use crate::webgl::global::WebGlGlobal;
use keyed::keyed_handle;
use crate::util::message::Message;

keyed_handle!(FlatPositionCampaignHandle);

struct Campaign {
    origin: Vec<(u32,u32)>,
    sizes: Vec<(u32,u32)>
}

pub(crate) struct FlatPositionManager {
    uniform_name: String,
    requests: KeyedData<FlatPositionCampaignHandle,Option<Campaign>>,
    weave: CanvasWeave,
    canvas: Option<FlatId>
}

impl FlatPositionManager {
    pub(crate) fn new(weave: &CanvasWeave, uniform_name: &str) -> FlatPositionManager {
        FlatPositionManager {
            uniform_name: uniform_name.to_string(),
            weave: weave.clone(),
            requests: KeyedData::new(),
            canvas: None
        }
    }

    pub(crate) fn insert(&mut self, sizes: &[(u32,u32)]) -> FlatPositionCampaignHandle {
        self.requests.add(Some(Campaign {
            sizes: sizes.to_vec(), origin: vec![]
        }))
    }

    fn allocate(&mut self, gl: &mut WebGlGlobal, builder: &mut DrawingAllFlatsBuilder) -> Result<(),Message> {
        let mut sizes = vec![];
        let ids : Vec<_> = self.requests.keys().collect();
        for req_id in &ids {
            let req = self.requests.get(req_id).as_ref().unwrap();
            sizes.extend(req.sizes.iter());
        }
        if sizes.len() == 0 { return Ok(()); }
        let (mut origins,width,height) = self.weave.pack(&sizes,gl)?;
        let mut origins_iter = origins.drain(..);
        for req_id in &ids {
            let req = self.requests.get_mut(req_id).as_mut().unwrap();
            for _ in 0..req.sizes.len() {
                req.origin.push(origins_iter.next().unwrap());
            }
        }
        self.canvas = Some(builder.make_canvas(gl,&self.weave,(width,height),&self.uniform_name)?);
        Ok(())
    }

    pub(crate) fn origins(&self, id: &FlatPositionCampaignHandle) -> Vec<(u32,u32)> {
        self.requests.get(id).as_ref().unwrap().origin.clone()
    }

    pub(crate) fn canvas(&self) -> Result<Option<&FlatId>,Message> { Ok(self.canvas.as_ref()) }

    pub(crate) fn make(&mut self, gl: &mut WebGlGlobal, drawable: &mut DrawingAllFlatsBuilder) -> Result<(),Message> {
        self.allocate(gl,drawable)?;
        for (id,_) in self.requests.items() {
            if let Some(canvas) = &self.canvas {
                drawable.add(id,canvas);
            }
        }
        Ok(())
    }
}
