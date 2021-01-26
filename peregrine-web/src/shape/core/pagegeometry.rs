use super::super::layers::layer::{ Layer };
use super::super::layers::geometry::GeometryProcessName;
use super::super::layers::patina::PatinaProcessName;
use super::super::layers::arrayutil::{ add_fixed_sea_box, ship_box, interleave_one };
use crate::webgl::{ AttribHandle, ProtoProcess, AccumulatorCampaign, Program };
use peregrine_core::{ ShipEnd, ScreenEdge };

#[derive(Clone)]
pub struct PageProgram {
    vertexes: AttribHandle,
    signs: AttribHandle,
}

impl PageProgram {
    pub(crate) fn new(program: &Program) -> anyhow::Result<PageProgram> {
        Ok(PageProgram {
            vertexes: program.get_attrib_handle("aVertexPosition")?,
            signs: program.get_attrib_handle("aSign")?
        })
    }
}

#[derive(Clone)]
pub struct PageGeometry {
    variety: PageProgram,
    skin: PatinaProcessName
}

impl PageGeometry {
    pub(crate) fn new(process: &ProtoProcess, skin: &PatinaProcessName, variety: &PageProgram) -> anyhow::Result<PageGeometry> {
        Ok(PageGeometry { variety: variety.clone(), skin: skin.clone() })
    }

    pub(crate) fn add_solid_rectangles(&self, layer: &mut Layer,
                                        sea_x: ScreenEdge, yy: Vec<f64>,
                                        ship_x: ShipEnd, ship_y: ShipEnd,
                                        size_x: Vec<f64>, size_y: Vec<f64>) -> anyhow::Result<AccumulatorCampaign> {
        let mut campaign = layer.make_campaign(&GeometryProcessName::Page,&self.skin,yy.len(),&[0,3,1,2,1,3])?;
        let len = yy.len();
        let sign_x = match sea_x { ScreenEdge::Max(_) => -1., _ => 1. };
        let signs = interleave_one(sign_x,1.,len)?;
        let mut vertexes = ship_box(ship_x,size_x,ship_y,size_y,len);
        add_fixed_sea_box(&mut vertexes,false,sea_x);
        add_fixed_sea_box(&mut vertexes,true,ScreenEdge::Min(yy));
        campaign.add(&self.variety.signs,signs)?;
        campaign.add(&self.variety.vertexes,vertexes)?;
        Ok(campaign)
    }
}
