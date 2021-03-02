use peregrine_core::{ Shape, SingleAnchor, SeaEnd, Patina, Colour, AnchorPair, SeaEndPair, Plotter, DirectColour };
use super::text::TextHandle;
use super::super::layers::layer::{ Layer };
use super::super::layers::patina::PatinaProcessName;
use super::super::layers::geometry::GeometryProcessName;
use crate::webgl::{ ProcessStanzaElements, ProcessStanzaArray, ProcessStanzaAddable };
use super::super::layers::drawing::DrawingTools;
use crate::webgl::canvas::drawingflats::DrawingFlatsDrawable;
use crate::webgl::canvas::bindery::TextureBindery;

pub enum PreparedShape {
    SingleAnchorRect(SingleAnchor,Patina,Vec<String>,Vec<f64>,Vec<f64>),
    DoubleAnchorRect(AnchorPair,Patina,Vec<String>),
    Text(SingleAnchor,Vec<TextHandle>,Vec<String>),
    Wiggle((f64,f64),Vec<Option<f64>>,Plotter,String)
}

fn colour_to_patina(colour: Colour) -> PatinaProcessName {
    match colour {
        Colour::Direct(_) => PatinaProcessName::Direct,
        Colour::Spot(c) => PatinaProcessName::Spot(c)
    }
}

fn add_rectangle<'a>(layer: &'a mut Layer, anchor: SingleAnchor, skin: &PatinaProcessName, _allotment: Vec<String>, x_size: Vec<f64>, y_size: Vec<f64>, hollow: bool) -> anyhow::Result<(ProcessStanzaElements,GeometryProcessName)> {
    match ((anchor.0).0,(anchor.0).1,(anchor.1).0,(anchor.1).1) {
        (SeaEnd::Paper(xx),ship_x,SeaEnd::Paper(yy),ship_y) => {
            Ok((layer.get_pin(skin)?.add_rectangles(layer,xx,yy,ship_x,ship_y,x_size,y_size,hollow)?,GeometryProcessName::Pin))
        },
        (SeaEnd::Screen(sea_x),ship_x,SeaEnd::Screen(sea_y),ship_y) => {
            Ok((layer.get_fix(skin)?.add_rectangles(layer,sea_x,sea_y,ship_x,ship_y,x_size,y_size,hollow)?,GeometryProcessName::Fix))
        },
        (SeaEnd::Paper(xx),ship_x,SeaEnd::Screen(sea_y),ship_y) => {
            Ok((layer.get_tape(skin)?.add_rectangles(layer,xx,sea_y,ship_x,ship_y,x_size,y_size,hollow)?,GeometryProcessName::Tape))         
        },
        (SeaEnd::Screen(sea_x),ship_x,SeaEnd::Paper(yy),ship_y) => {
            Ok((layer.get_page(skin)?.add_rectangles(layer,sea_x,yy,ship_x,ship_y,x_size,y_size,hollow)?,GeometryProcessName::Page))
        }
    }
}

fn add_stretchtangle<'a>(layer: &'a mut Layer, anchors: AnchorPair, skin: &PatinaProcessName, _allotment: Vec<String>, hollow: bool) -> anyhow::Result<(ProcessStanzaElements,GeometryProcessName)> {
    let anchors_x = anchors.0;
    let anchors_y = anchors.1;
    let anchor_sea_x = anchors_x.0;
    let pxx1 = anchors_x.1;
    let pxx2 = anchors_x.2;
    let anchor_sea_y = anchors_y.0;
    let pyy1 = anchors_y.1;
    let pyy2 = anchors_y.2;
    match (anchor_sea_x,anchor_sea_y) {
        (SeaEndPair::Paper(axx1,axx2),SeaEndPair::Paper(ayy1,ayy2)) => {
            Ok((layer.get_pin(skin)?.add_stretchtangle(layer,axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,hollow)?,GeometryProcessName::Pin))
        },
        (SeaEndPair::Screen(axx1,axx2),SeaEndPair::Screen(ayy1,ayy2)) => {
            Ok((layer.get_fix(skin)?.add_stretchtangle(layer,axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,hollow)?,GeometryProcessName::Fix))
        },
        (SeaEndPair::Paper(axx1,axx2),SeaEndPair::Screen(ayy1,ayy2)) => {
            Ok((layer.get_tape(skin)?.add_stretchtangle(layer,axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,hollow)?,GeometryProcessName::Tape))
        },
        (SeaEndPair::Screen(axx1,axx2),SeaEndPair::Paper(ayy1,ayy2)) => {
            Ok((layer.get_page(skin)?.add_stretchtangle(layer,axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,hollow)?,GeometryProcessName::Page))
        }
    }
}

fn add_wiggle<'a>(layer: &'a mut Layer, start: f64, end: f64, y: Vec<Option<f64>>, height: f64, patina: &PatinaProcessName, _allotment: String) -> anyhow::Result<(ProcessStanzaArray,GeometryProcessName)> {    
    let stanza_builder = layer.get_wiggle(patina)?.add_wiggle(layer,start,end,y,height)?;
    Ok((stanza_builder,GeometryProcessName::Pin))
}

fn add_colour(addable: &mut dyn ProcessStanzaAddable, layer: &mut Layer, geometry: &GeometryProcessName, colour: &Colour, vertexes: usize) -> anyhow::Result<()> {
    match colour {
        Colour::Direct(d) => {
            let direct = layer.get_direct(geometry)?;
            direct.direct(addable,d,vertexes)?;
        },
        Colour::Spot(colour) => {
            let spot = layer.get_spot(geometry,colour)?;
            let mut process = layer.get_process_mut(geometry,&PatinaProcessName::Spot(colour.clone()))?;
            spot.spot(&mut process)?;
        }
    }
    Ok(())
}

pub(crate) fn prepare_shape_in_layer(_layer: &mut Layer, tools: &mut DrawingTools, shape: Shape) -> anyhow::Result<PreparedShape> {
    Ok(match shape {
        Shape::SingleAnchorRect(anchor,patina,allotment,x_size,y_size) => {
            PreparedShape::SingleAnchorRect(anchor,patina,allotment,x_size,y_size)
        },
        Shape::DoubleAnchorRect(anchors,patina,allotment) => {
            PreparedShape::DoubleAnchorRect(anchors,patina,allotment)
        },
        Shape::Wiggle(range,y,plotter,allotment) => {
            PreparedShape::Wiggle(range,y,plotter,allotment)
        },
        Shape::Text(anchor,mut pen,texts,allotment) => {
            let drawing_text = tools.text();
            if pen.2.len() == 0 { pen.2.push(DirectColour(0,0,0)); }
            let colours_iter = pen.2.iter().cycle();
            let handles : Vec<_> = texts.iter().zip(colours_iter).map(|(text,colour)| drawing_text.add_text(&pen,text,colour)).collect();
            PreparedShape::Text(anchor,handles,allotment)
        }
    })
}

pub(crate) fn add_shape_to_layer(layer: &mut Layer, tools: &mut DrawingTools, canvas_builder: &DrawingFlatsDrawable, bindery: &TextureBindery, shape: PreparedShape) -> anyhow::Result<()> {
    match shape {
        PreparedShape::SingleAnchorRect(anchor,patina,allotment,x_size,y_size) => {
            match patina {
                Patina::Filled(colour) => {
                    let patina = colour_to_patina(colour.clone());
                    let (mut campaign,geometry) = add_rectangle(layer,anchor,&patina,allotment,x_size,y_size,false)?;
                    add_colour(&mut campaign,layer,&geometry,&colour,4)?;
                    campaign.close();
                },
                Patina::Hollow(colour) => {
                    let patina = colour_to_patina(colour.clone());
                    let (mut campaign,geometry) = add_rectangle(layer,anchor,&patina,allotment,x_size,y_size,true)?;
                    add_colour(&mut campaign,layer,&geometry,&colour,4)?;
                    campaign.close();
                },
                _ => {}
            }
        },
        PreparedShape::DoubleAnchorRect(anchors,patina,allotment) => {
            match patina {
                Patina::Filled(colour) => {
                    let patina = colour_to_patina(colour.clone());
                    let (mut campaign,geometry) = add_stretchtangle(layer,anchors,&patina,allotment,false)?;
                    add_colour(&mut campaign,layer,&geometry,&colour,4)?;
                    campaign.close();
                },
                Patina::Hollow(colour) => {
                    let patina = colour_to_patina(colour.clone());
                    let (mut campaign,geometry) = add_stretchtangle(layer,anchors,&patina,allotment,true)?;
                    add_colour(&mut campaign,layer,&geometry,&colour,4)?;
                    campaign.close();
                },
                _ => {}
            }
        },
        PreparedShape::Wiggle((start,end),y,Plotter(height,colour),allotment) => {
            let patina = colour_to_patina(Colour::Spot(colour.clone()));
            let (mut array,geometry) = add_wiggle(layer,start,end,y,height,&patina,allotment)?;
            let spot = layer.get_spot(&geometry,&colour)?;
            let mut process = layer.get_process_mut(&geometry,&patina)?;
            spot.spot(&mut process)?;
            array.close();
        },
        PreparedShape::Text(anchor,handles,allotments) => {
            // TODO factor
            let text = tools.text();
            let mut dims = vec![];
            let mut x_sizes = vec![];
            let mut y_sizes = vec![];
            let canvas = text.canvas_id(canvas_builder)?;
            for handle in &handles {
                let texture_areas = text.get_texture_areas(handle)?;
                let size = texture_areas.size();
                x_sizes.push(size.0 as f64);
                y_sizes.push(size.1 as f64);
                dims.push(texture_areas);
            }
            let (mut campaign,geometry) = add_rectangle(layer,anchor,&PatinaProcessName::Texture(canvas.clone()),allotments,x_sizes,y_sizes,false)?;
            let patina = layer.get_texture(&geometry,&canvas)?;
            let mut process = layer.get_process_mut(&geometry,&PatinaProcessName::Texture(canvas.clone()))?;
            patina.add_rectangle(&mut process,&mut campaign,bindery,&canvas,&dims)?;
            campaign.close();
        }
    }
    Ok(())
}
