use std::f64::consts::PI;
use peregrine_data::{SpaceBase, LeafStyle };
use peregrine_toolkit::eachorevery::EachOrEvery;
use peregrine_toolkit::error::Error;
use peregrine_toolkit::log;
use super::super::layers::layer::{ Layer };
use super::directcolourdraw::{DirectColourDraw, DirectProgram};
use crate::shape::layers::geometry::{GeometryProcessName};
use crate::shape::layers::patina::{PatinaProcessName};
use crate::shape::triangles::drawgroup::DrawGroup;
use crate::shape::triangles::triangleadder::TriangleAdder;
use crate::webgl::{ProcessBuilder, ProcessStanzaElements};

fn radius_to_segments(r: f64) -> usize { ((r/6.).max(3.).min(20.).round() as usize)*2+1 }

/*
fn add_patina(elements: &mut ProcessStanzaElements, patina: &DrawingShapePatina, count: &[usize]) -> Result<(),Error> {
    match patina {
        DrawingShapePatina::Solid(direct,colours) |
        DrawingShapePatina::Hollow(direct,colours) => {
            direct.direct_variable(elements,&colours,count)?;
        },
        _ => {}
    }
    Ok(())
}

#[derive(Debug)]
struct CircleCampaign {
    index: Vec<u16>,
    data: Vec<f32>,
    count: Vec<usize>
}

impl CircleCampaign {
    fn new() -> CircleCampaign {
        CircleCampaign {
            index: vec![],
            data: vec![],
            count: vec![]
        }
    }

    fn data(&mut self, x: f64, y: f64, b: f64, d: i8) {
        self.data.push(x as f32);
        self.data.push(y as f32);
        self.data.push(b as f32);
        self.data.push((1.0 - (d as f64+128.) / 255.) as f32);
    }

    fn draw_one_circle(&mut self, base: f64, normal: f64, tangent: f64, radius: f64, depth: i8) -> bool {
        let segs = radius_to_segments(radius);
        if self.index.len() + segs > 1000 { return false; }
        self.data(tangent,normal,base,depth); // centre
        self.index.push(0); // start-cap
        for i in 0..segs {
            let t = (i as f64)*PI*2./(segs as f64);
            self.index.push(0);
            self.index.push(1+i as u16);
            self.data(tangent+radius*t.cos(),normal+radius*t.sin(),base,depth);
        }
        self.index.push(segs as u16); // end-cap
        self.count.push(segs+1);
        true
    }

    fn close(&mut self, builder: &mut ProcessBuilder, adder: &TriangleAdder, patina: &mut DrawingShapePatina) -> Result<(),Error> {
        if self.index.len() == 0 { return Ok(()); }
        /*
        log!("{:?} -> {:?}",self.index,self.data);
        let mut elements = builder.get_stanza_builder().make_elements(1,&self.index)?;
        add_patina(&mut elements,patina,&self.count)?;
        adder.add_data4(&mut elements,data,depth)?;
        if self.program.origin_coords.is_some() {
            let (data,_)= add_spacebase4(&PartialSpaceBase::from_spacebase(area.top_left().clone()),&depth_in,&self.kind,self.left,self.width,None)?;
            self.program.add_origin_data4(&mut elements,data)?;
        }
        if self.program.run_coords.is_some() {
            let run = run.unwrap_or(area.top_left().map_allotments(|_| ()));
            let (data,_)= add_spacebase4(&PartialSpaceBase::from_spacebase(run),&depth_in,&self.kind,self.left,self.width,None)?;
            self.program.add_run_data4(&mut elements,data)?;
        }
        */


        //add_colour(campaign,&drawing_shape_patina,area.len())?;
        //campaign.close()?;
        //Ok(ShapeToAdd::Dynamic(Box::new(Rectangles::new(circles,&gl))))


        Ok(())
    }
}

pub(crate) fn make_circle(layer: &mut Layer,
            geometry_process: &GeometryProcessName, patina: &mut DrawingShapePatina,
            position: SpaceBase<f64,LeafStyle>, radius: EachOrEvery<f64>,
            depth: EachOrEvery<i8>, left: f64, draw_group: &DrawGroup
        ) -> Result<(),Error> {
//        )-> Result<(ProcessStanzaArray,usize),Error> {
    if position.len() == 0 { return Ok(()); }
    match patina.yielder_mut() {
        PatinaTarget::Direct(patina_yielder) => {
            let builder = layer.get_process_builder(geometry_process,&PatinaProcessName::Direct)?;
            let draw = DirectColourDraw::new(&DirectProgram::new(builder.program_builder())?)?;
            let adder = TriangleAdder::new(builder)?;
            /* set up webgl */
            let pos_iter = position.iter();
            let radius_iter = radius.iter(position.len()).expect("circle size mismatch");
            let depth_iter = depth.iter(position.len()).unwrap();
            let mut campaign = CircleCampaign::new();
            for ((pos,radius),depth) in pos_iter.zip(radius_iter).zip(depth_iter) {
                log!("base {} normal {} tangent {} radius {} depth {}",pos.base,pos.normal,pos.tangent,radius,depth);
                if !campaign.draw_one_circle(*pos.base,*pos.normal,*pos.tangent,*radius,*depth) {
                    campaign.close(builder,&adder,patina)?;
                    campaign = CircleCampaign::new();
                    campaign.draw_one_circle(*pos.base,*pos.normal,*pos.tangent,*radius,*depth);
                }
            }
            campaign.close(builder,&adder,patina)?;
        },
        _ => {}
    }
    Ok(())
}
*/