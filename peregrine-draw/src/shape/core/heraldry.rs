use keyed::KeyedData;
use peregrine_data::{ DirectColour };
use keyed::keyed_handle;
use crate::webgl::{ CanvasWeave, DrawingFlatsDrawable, FlatId, FlatStore, Flat, FlatPlotAllocator, FlatPlotRequestHandle };
use crate::webgl::global::WebGlGlobal;
use super::texture::CanvasTextureAreas;
use crate::util::message::Message;


keyed_handle!(HeraldryHandle);

pub(crate) enum HeraldrySpec {
    Stripe(DirectColour,DirectColour)
}

struct Heraldry {
    spec: HeraldrySpec,
    pic_origin: Option<(u32,u32)>,
    mask_origin: Option<(u32,u32)>,
    size: Option<(u32,u32)>
}

impl Heraldry {
    fn new(spec: HeraldrySpec) -> Heraldry {
        Heraldry {
            spec,
            pic_origin: None,
            mask_origin: None,
            size: None
        }
    }

    fn calc_size(&mut self, _gl: &mut WebGlGlobal) -> Result<(),Message> {
        self.size = Some((32,32));
        Ok(())
    }

    fn build(&mut self, canvas: &Flat, pic_origin: (u32,u32), mask_origin: (u32,u32)) -> Result<(),Message> {
        let size = self.size.unwrap();
        self.pic_origin = Some(pic_origin);
        self.mask_origin = Some(mask_origin);
        // XXX draw it
        /*
        canvas.set_font(&self.pen)?;
        canvas.text(&self.text,text_origin,size,&self.colour)?;
        canvas.text(&self.text,mask_origin,size,&DirectColour(0,0,0))?;
        */
        Ok(())
    }

    fn get_texture_areas(&self) -> Result<CanvasTextureAreas,Message> {
        Ok(CanvasTextureAreas::new(
            self.pic_origin.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed("texture packing failure, t origin".to_string()))?,
            self.mask_origin.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed("texture packing failure. m origin".to_string()))?,
            self.size.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed("texture packing failure. size A".to_string()))?
        ))
    }
}

pub struct DrawingHeraldry {
    pics: KeyedData<HeraldryHandle,Heraldry>,
    request: Option<FlatPlotRequestHandle>
}

impl DrawingHeraldry {
    pub fn new() -> DrawingHeraldry {
        DrawingHeraldry {
            pics: KeyedData::new(),
            request: None
        }
    }

    pub(crate) fn add(&mut self, spec: HeraldrySpec) -> HeraldryHandle {
        self.pics.add(Heraldry::new(spec))
    }

    fn calc_sizes(&mut self, gl: &mut WebGlGlobal) -> Result<(),Message> {
        for pic in self.pics.values_mut() {
            pic.calc_size(gl)?;
        }
        Ok(())
    }

    pub(crate) fn start_preparation(&mut self, gl: &mut WebGlGlobal, allocator: &mut FlatPlotAllocator,uniform_name: &str) -> Result<(),Message> {
        self.calc_sizes(gl)?;
        let mut sizes = vec![];
        for pic in self.pics.values_mut() {
            let size = pic.size.as_mut().unwrap().clone();
            /* mask and text */
            sizes.push(size);
            sizes.push(size);
        }
        self.request = Some(allocator.allocate(&CanvasWeave::Crisp,&sizes,uniform_name));
        Ok(())
    }

    pub(crate) fn finish_preparation(&mut self, store: &mut FlatStore, builder: &DrawingFlatsDrawable) -> Result<(),Message> {
        let mut origins = builder.origins(self.request.as_ref().unwrap());
        let mut origins_iter = origins.drain(..);
        let canvas_id = builder.canvas(self.request.as_ref().unwrap());
        let canvas = store.get_mut(&canvas_id)?;
        for text in self.pics.values_mut() {
            let mask_origin = origins_iter.next().unwrap();
            let text_origin = origins_iter.next().unwrap();
            text.build(canvas,text_origin,mask_origin)?;
        }
        Ok(())
    }

    pub(crate) fn canvas_id(&self, builder: &DrawingFlatsDrawable) -> Result<FlatId,Message> {
        let request = self.request.as_ref().cloned().ok_or_else(|| Message::CodeInvariantFailed(format!("missing canvas id")))?;
        Ok(builder.canvas(&request))
    }

    pub(crate) fn get_texture_areas(&self, handle: &HeraldryHandle) -> Result<CanvasTextureAreas,Message> {
        self.pics.get(handle).get_texture_areas()
    }
}
