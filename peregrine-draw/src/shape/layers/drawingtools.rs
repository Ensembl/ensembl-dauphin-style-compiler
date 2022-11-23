use peregrine_data::{Assets, Scale};
use peregrine_toolkit::{error::Error};
use crate::{shape::{core::{text::DrawingText, bitmap::DrawingBitmap}}, webgl::{global::WebGlGlobal, CanvasWeave, DrawingCanvasesBuilder, canvas::{imagecache::ImageCache, composition::compositionbuilder::CompositionBuilder}}, util::fonts::Fonts, hotspots::drawinghotspots::{DrawingHotspots, DrawingHotspotsBuilder}};

const CANVAS_TYPE_LEN : usize = 3;

#[derive(Debug)]
pub(crate) enum CanvasType {
    Crisp ,
    HeraldryHoriz, // Horizontal lines
    HeraldryVert // Vertical Lines
}

impl CanvasType {
    fn from_index(index: usize) -> CanvasType {
        match index {
            0 => CanvasType::Crisp,
            1 => CanvasType::HeraldryHoriz,
            2 => CanvasType::HeraldryVert ,
            _ => panic!("bad canvastype index")           
        }
    }

    fn index(&self) -> usize {
        match self {
            CanvasType::Crisp => 0,
            CanvasType::HeraldryHoriz => 1,
            CanvasType::HeraldryVert => 2
        }
    }

    fn to_weave(&self) -> CanvasWeave {
        match self {
            CanvasType::Crisp => CanvasWeave::Crisp,
            CanvasType::HeraldryHoriz => CanvasWeave::VertStack, /* gets horizontal lines */
            CanvasType::HeraldryVert => CanvasWeave::HorizStack, /* gets vertical lines */
        }
    }
}

pub(crate) struct DrawingTools {
    pub hotspots: DrawingHotspots
}

pub(crate) struct DrawingToolsBuilder {
    text: DrawingText,
    bitmap: DrawingBitmap,
    manager: Vec<CompositionBuilder>,
    hotspots: DrawingHotspotsBuilder
}

impl DrawingToolsBuilder {
    pub(super) fn new(fonts: &Fonts, assets: &Assets, image_cache: &ImageCache, scale: Option<&Scale>, left: f64, bitmap_multiplier: f64) -> DrawingToolsBuilder {
        DrawingToolsBuilder {
            manager: (0..CANVAS_TYPE_LEN).map(|x| CompositionBuilder::new(&CanvasType::from_index(x).to_weave())).collect(),
            text: DrawingText::new(fonts,bitmap_multiplier),
            bitmap: DrawingBitmap::new(assets,image_cache),
            hotspots: DrawingHotspotsBuilder::new(scale, left)
        }
    }

    pub(crate) fn composition_builder(&mut self, type_: &CanvasType) -> &mut CompositionBuilder { &mut self.manager[type_.index()] }
    pub(crate) fn text(&mut self) -> &mut DrawingText { &mut self.text }
    pub(crate) fn bitmap(&mut self) -> &mut DrawingBitmap { &mut self.bitmap }
    pub(crate) fn hotspots(&mut self) -> &mut DrawingHotspotsBuilder { &mut self.hotspots }

    pub(crate) fn build(self) -> Result<DrawingTools,Error> {
        Ok(DrawingTools {
            hotspots: self.hotspots.build()?
        })
    }

    pub(crate) async fn preprep(&mut self) -> Result<(),Error> {
        self.text.prepare_for_allocation().await?;
        Ok(())
    }

    pub(crate) fn prepare(&mut self, gl: &mut WebGlGlobal, drawable: &mut DrawingCanvasesBuilder) -> Result<(),Error> {
        for i in 0..CANVAS_TYPE_LEN {
            let canvas = self.manager[i].draw_on_bitmap(gl)?;
            drawable.add_canvas(&canvas,"uSampler");
        }
        Ok(())
    }
}
