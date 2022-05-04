use peregrine_data::{Assets, Scale};

use crate::{shape::{core::{text::DrawingText, bitmap::DrawingBitmap}, heraldry::heraldry::DrawingHeraldry}, webgl::{global::WebGlGlobal, canvas::flatplotallocator::FlatPositionManager, CanvasWeave, DrawingAllFlatsBuilder, FlatStore}, Message};

use super::drawingzmenus::{DrawingHotspotsBuilder, DrawingHotspots};

pub(crate) struct ToolPreparations {
    crisp: FlatPositionManager,
    heraldry_h: FlatPositionManager,
    heraldry_v: FlatPositionManager
}

impl ToolPreparations {
    fn new() -> ToolPreparations {
        ToolPreparations {
            crisp: FlatPositionManager::new(&CanvasWeave::Crisp,"uSampler"),
            heraldry_h: FlatPositionManager::new(&CanvasWeave::HorizStack,"uSampler"),
            heraldry_v: FlatPositionManager::new(&CanvasWeave::VertStack,"uSampler"),
        }
    }

    pub(crate) fn crisp_manager(&mut self) -> &mut FlatPositionManager { &mut self.crisp }
    pub(crate) fn heraldry_h_manager(&mut self) -> &mut FlatPositionManager { &mut self.heraldry_h }
    pub(crate) fn heraldry_v_manager(&mut self) -> &mut FlatPositionManager { &mut self.heraldry_v }

    pub(super) fn allocate(&mut self, gl: &mut WebGlGlobal, drawable: &mut DrawingAllFlatsBuilder) -> Result<(),Message> {
        self.crisp.make(gl,drawable)?;
        self.heraldry_h.make(gl,drawable)?;
        self.heraldry_v.make(gl,drawable)?;
        Ok(())
    }
}

pub(crate) struct DrawingTools {
    pub zmenus: DrawingHotspots
}

pub(crate) struct DrawingToolsBuilder {
    text: DrawingText,
    bitmap: DrawingBitmap,
    heraldry: DrawingHeraldry,
    zmenus: DrawingHotspotsBuilder
}

impl DrawingToolsBuilder {
    pub(super) fn new(assets: &Assets, scale: Option<&Scale>, left: f64) -> DrawingToolsBuilder {
        DrawingToolsBuilder {
            text: DrawingText::new(),
            bitmap: DrawingBitmap::new(assets),
            heraldry: DrawingHeraldry::new(),
            zmenus: DrawingHotspotsBuilder::new(scale, left)
        }
    }

    pub(crate) fn text(&mut self) -> &mut DrawingText { &mut self.text }
    pub(crate) fn bitmap(&mut self) -> &mut DrawingBitmap { &mut self.bitmap }
    pub(crate) fn heraldry(&mut self) -> &mut DrawingHeraldry { &mut self.heraldry }
    pub(crate) fn zmenus(&mut self) -> &mut DrawingHotspotsBuilder { &mut self.zmenus }

    pub(crate) fn build(self) -> DrawingTools {
        DrawingTools {
            zmenus: self.zmenus.build()
        }
    }

    pub(crate) fn start_preparation(&mut self, gl: &mut WebGlGlobal) -> Result<ToolPreparations,Message> {
        let mut preparations = ToolPreparations::new();
        self.text.calculate_requirements(gl,&mut preparations.crisp)?;
        self.bitmap.calculate_requirements(gl, &mut preparations.crisp)?;
        self.heraldry.calculate_requirements(gl,&mut preparations)?;
        Ok(preparations)
    }

    pub(crate) fn finish_preparation(&mut self, canvas_store: &mut FlatStore, mut preparations: ToolPreparations) -> Result<(),Message> {
        self.text.manager().draw_at_locations(canvas_store,&mut preparations.crisp)?;
        self.bitmap.manager().draw_at_locations(canvas_store,&mut preparations.crisp)?;
        self.heraldry.draw_at_locations(canvas_store,&mut preparations)?;
        Ok(())
    }
}
