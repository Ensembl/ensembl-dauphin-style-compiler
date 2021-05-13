use peregrine_data::{
    Allotment, AllotmentHandle, Allotter, AnchorPair, Colour, DirectColour, Patina, Plotter, ScreenEdge, SeaEnd,
    SeaEndPair, Shape, SingleAnchor, SpaceBaseArea
};
use web_sys::WebGlRenderingContext;
use super::text::TextHandle;
use super::fixgeometry::FixData;
use super::pagegeometry::PageData;
use super::pingeometry::PinData;
use super::tapegeometry::TapeData;
use super::super::layers::layer::{ Layer };
use super::super::layers::patina::PatinaProcessName;
use super::super::layers::geometry::GeometryProgramName;
use crate::webgl::{DrawingFlatsDrawable, ProcessStanzaAddable, ProcessStanzaArray, ProcessStanzaElements };
use crate::webgl::global::WebGlGlobal;
use super::super::layers::drawing::DrawingTools;
use crate::util::message::Message;

pub enum PreparedShape {
    SingleAnchorRect(SingleAnchor,Patina,Vec<Allotment>,Vec<f64>,Vec<f64>),
    DoubleAnchorRect(AnchorPair,Patina,Vec<Allotment>),
    Text(SingleAnchor,Vec<TextHandle>,Vec<Allotment>),
    Wiggle((f64,f64),Vec<Option<f64>>,Plotter,Allotment),
    SpaceBaseRect(SpaceBaseArea,Patina,Vec<Allotment>)
}

fn colour_to_patina(colour: Colour) -> PatinaProcessName {
    match colour {
        Colour::Direct(_) => PatinaProcessName::Direct,
        Colour::Spot(c) => PatinaProcessName::Spot(c)
    }
}

fn rectangle_to_geometry(anchor: &SingleAnchor) -> GeometryProgramName {
    match (&(anchor.0).0,&(anchor.0).1,&(anchor.1).0,&(anchor.1).1) {
        (SeaEnd::Paper(_),_,SeaEnd::Paper(_),_) => {
            GeometryProgramName::Pin
        },
        (SeaEnd::Screen(_),_,SeaEnd::Screen(_),_) => {
            GeometryProgramName::Fix
        },
        (SeaEnd::Paper(_),_,SeaEnd::Screen(_),_) => {
            GeometryProgramName::Tape
        },
        (SeaEnd::Screen(_),_,SeaEnd::Paper(_),_) => {
            GeometryProgramName::Page
        }
    }
}

fn apply_allotments(y: &[f64], allotment: &[Allotment]) -> Vec<f64> {
    // XXX yuk!
    let len = if y.len() != allotment.len() {
        y.len() * allotment.len()
    } else {
        y.len()
    };
    let mut iter = allotment.iter().cycle().zip(y.iter().cycle());
    (0..len).map(|_| {
        let (allotment,y) = iter.next().unwrap();
        let offset = allotment.position().offset() as f64;
        *y+offset
    }).collect()
}

fn apply_allotments_se(y: &ScreenEdge, allotment: &[Allotment]) -> ScreenEdge {
    y.transform(|data| {
        apply_allotments(data,allotment)
    })
}

// TODO allotments to gl variables for collapsing, moving, etc.
fn add_rectangle<'a>(layer: &'a mut Layer, anchor: SingleAnchor, skin: &PatinaProcessName, allotment: Vec<Allotment>, x_size: Vec<f64>, y_size: Vec<f64>, hollow: bool) -> Result<ProcessStanzaElements,Message> {
    match ((anchor.0).0,(anchor.0).1,(anchor.1).0,(anchor.1).1) {
        (SeaEnd::Paper(xx),ship_x,SeaEnd::Paper(yy),ship_y) => {
            let yy = apply_allotments(&yy,&allotment);
            let pin_data = PinData::add_rectangles(layer,xx,yy,ship_x,ship_y,x_size,y_size,hollow);
            let pin = layer.get_pin(skin)?;
            let process = layer.get_process_mut(&GeometryProgramName::Pin, skin)?;
            Ok(pin.add(process,pin_data)?)
        },
        (SeaEnd::Screen(sea_x),ship_x,SeaEnd::Screen(sea_y),ship_y) => {
            let sea_y = apply_allotments_se(&sea_y,&allotment);
            let fix_data = FixData::add_rectangles(sea_x,sea_y,ship_x,ship_y,x_size,y_size,hollow);
            let fix = layer.get_fix(skin)?;
            let process = layer.get_process_mut(&GeometryProgramName::Fix,skin)?;
            Ok(fix.add(process,fix_data)?)
        },
        (SeaEnd::Paper(xx),ship_x,SeaEnd::Screen(sea_y),ship_y) => {
            let sea_y = apply_allotments_se(&sea_y,&allotment);
            let tape_data = TapeData::add_rectangles(layer,xx,sea_y,ship_x,ship_y,x_size,y_size,hollow);
            let tape = layer.get_tape(skin)?;
            let process = layer.get_process_mut(&GeometryProgramName::Tape,skin)?;
            Ok(tape.add(process,tape_data)?)         
        },
        (SeaEnd::Screen(sea_x),ship_x,SeaEnd::Paper(yy),ship_y) => {
            let yy = apply_allotments(&yy,&allotment);
            let page_data = PageData::add_rectangles(sea_x,yy,ship_x,ship_y,x_size,y_size,hollow);
            let page = layer.get_page(skin)?;
            let process = layer.get_process_mut(&GeometryProgramName::Page,skin)?;
            Ok(page.add(process,page_data)?)
        }
    }
}

fn stretchtangle_to_geometry(anchors: &AnchorPair) -> GeometryProgramName {
    match (&(anchors.0).0,&(anchors.1).0) {
        (SeaEndPair::Paper(_,_),SeaEndPair::Paper(_,_)) => {
            GeometryProgramName::Pin
        },
        (SeaEndPair::Screen(_,_),SeaEndPair::Screen(_,_)) => {
            GeometryProgramName::Fix
        },
        (SeaEndPair::Paper(_,_),SeaEndPair::Screen(_,_)) => {
            GeometryProgramName::Tape
        },
        (SeaEndPair::Screen(_,_),SeaEndPair::Paper(_,_)) => {
            GeometryProgramName::Page
        }
    }
}

fn add_stretchtangle<'a>(layer: &'a mut Layer, anchors: AnchorPair, skin: &PatinaProcessName, allotment: Vec<Allotment>, hollow: bool) -> Result<ProcessStanzaElements,Message> {
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
            let ayy1 = apply_allotments(&ayy1,&allotment);
            let ayy2 = apply_allotments(&ayy2,&allotment);
            let pin_data = PinData::add_stretchtangle(layer,axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,hollow);
            let pin = layer.get_pin(skin)?;
            let process = layer.get_process_mut(&GeometryProgramName::Pin, skin)?;
            Ok(pin.add(process,pin_data)?)
        },
        (SeaEndPair::Screen(axx1,axx2),SeaEndPair::Screen(ayy1,ayy2)) => {
            let ayy1 = apply_allotments_se(&ayy1,&allotment);
            let ayy2 = apply_allotments_se(&ayy2,&allotment);
            let fix_data = FixData::add_stretchtangle(axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,hollow);
            let fix = layer.get_fix(skin)?;
            let process = layer.get_process_mut(&GeometryProgramName::Fix,skin)?;
            Ok(fix.add(process,fix_data)?)
        },
        (SeaEndPair::Paper(axx1,axx2),SeaEndPair::Screen(ayy1,ayy2)) => {
            let ayy1 = apply_allotments_se(&ayy1,&allotment);
            let ayy2 = apply_allotments_se(&ayy2,&allotment);
            let tape_data = TapeData::add_stretchtangle(layer,axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,hollow);
            let tape = layer.get_tape(skin)?;
            let process = layer.get_process_mut(&GeometryProgramName::Tape,&skin)?;
            Ok(tape.add(process,tape_data)?)
        },
        (SeaEndPair::Screen(axx1,axx2),SeaEndPair::Paper(ayy1,ayy2)) => {
            let ayy1 = apply_allotments(&ayy1,&allotment);
            let ayy2 = apply_allotments(&ayy2,&allotment);
            let page_data = PageData::add_stretchtangle(axx1,ayy1,axx2,ayy2,pxx1,pyy1,pxx2,pyy2,hollow);
            let page = layer.get_page(skin)?;
            let process = layer.get_process_mut(&GeometryProgramName::Page,skin)?;
            Ok(page.add(process,page_data)?)
        }
    }
}

fn add_wiggle<'a>(layer: &'a mut Layer, start: f64, end: f64, y: Vec<Option<f64>>, height: f64, patina: &PatinaProcessName, _allotment: Allotment) -> Result<(ProcessStanzaArray,GeometryProgramName),Message> {    
    let wiggle = layer.get_wiggle(patina)?;
    let left = layer.left();
    let process = layer.get_process_mut(&GeometryProgramName::Page,patina)?;
    let stanza_builder = wiggle.add_wiggle(process,start,end,y,height,left)?;
    Ok((stanza_builder,GeometryProgramName::Pin))
}

fn add_colour(addable: &mut dyn ProcessStanzaAddable, layer: &mut Layer, geometry: &GeometryProgramName, colour: &Colour, vertexes: usize) -> Result<(),Message> {
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

// XXX not a new one for each!
fn allotments(allotter: &Allotter, allotments: &[AllotmentHandle]) -> Result<Vec<Allotment>,Message> {
    allotments.iter().map(|handle| {
        allotter.get(handle).map(|a| a.clone())
    }).collect::<Result<Vec<_>,_>>().map_err(|e| Message::DataError(e))
}

pub(crate) fn prepare_shape_in_layer(_layer: &mut Layer, tools: &mut DrawingTools, shape: Shape, allotter: &Allotter) -> Result<PreparedShape,Message> {
    Ok(match shape {
        Shape::SingleAnchorRect(anchor,patina,allotment,x_size,y_size) => {
            let allotment = allotments(allotter,&allotment)?;
            PreparedShape::SingleAnchorRect(anchor,patina,allotment,x_size,y_size)
        },
        Shape::DoubleAnchorRect(anchors,patina,allotment) => {
            let allotment = allotments(allotter,&allotment)?;
            PreparedShape::DoubleAnchorRect(anchors,patina,allotment)
        },
        Shape::Wiggle(range,y,plotter,allotment) => {
            let allotment = allotments(allotter,&[allotment])?;
            PreparedShape::Wiggle(range,y,plotter,allotment[0].clone())
        },
        Shape::Text(anchor,mut pen,texts,allotment) => {
            let allotment = allotments(allotter,&allotment)?;
            let drawing_text = tools.text();
            if pen.2.len() == 0 { pen.2.push(DirectColour(0,0,0)); }
            let colours_iter = pen.2.iter().cycle();
            let handles : Vec<_> = texts.iter().zip(colours_iter).map(|(text,colour)| drawing_text.add_text(&pen,text,colour)).collect();
            PreparedShape::Text(anchor,handles,allotment)
        },
        Shape::SpaceBaseRect(area,patina,allotment) => {
            let allotment = allotments(allotter,&allotment)?;
            PreparedShape::SpaceBaseRect(area,patina,allotment)
        }
    })
}

pub(crate) fn add_shape_to_layer(layer: &mut Layer, gl: &WebGlGlobal,  tools: &mut DrawingTools, canvas_builder: &DrawingFlatsDrawable, shape: PreparedShape) -> Result<(),Message> {
    match shape {
        PreparedShape::SingleAnchorRect(anchor,patina,allotment,x_size,y_size) => {
            match patina {
                Patina::Filled(colour) => {
                    let geometry = rectangle_to_geometry(&anchor);
                    let patina = colour_to_patina(colour.clone());
                    let mut campaign = add_rectangle(layer,anchor,&patina,allotment,x_size,y_size,false)?;
                    add_colour(&mut campaign,layer,&geometry,&colour,4)?;
                    campaign.close();
                },
                Patina::Hollow(colour) => {
                    let geometry = rectangle_to_geometry(&anchor);
                    let patina = colour_to_patina(colour.clone());
                    let mut campaign = add_rectangle(layer,anchor,&patina,allotment,x_size,y_size,true)?;
                    add_colour(&mut campaign,layer,&geometry,&colour,4)?;
                    campaign.close();
                },
                Patina::ZMenu(zmenu,values) =>{
                    tools.zmenus().add_rectangle(layer,zmenu,values,anchor,allotment,x_size,y_size);
                }
            }
        },
        PreparedShape::DoubleAnchorRect(anchors,patina,allotment) => {
            match patina {
                Patina::Filled(colour) => {
                    let geometry = stretchtangle_to_geometry(&anchors);
                    let patina = colour_to_patina(colour.clone());
                    let mut campaign = add_stretchtangle(layer,anchors,&patina,allotment,false)?;
                    add_colour(&mut campaign,layer,&geometry,&colour,4)?;
                    campaign.close();
                },
                Patina::Hollow(colour) => {
                    let geometry = stretchtangle_to_geometry(&anchors);
                    let patina = colour_to_patina(colour.clone());
                    let mut campaign = add_stretchtangle(layer,anchors,&patina,allotment,true)?;
                    add_colour(&mut campaign,layer,&geometry,&colour,4)?;
                    campaign.close();
                },
                Patina::ZMenu(zmenu,values) =>{
                    tools.zmenus().add_stretchtangle(layer,zmenu,values,anchors,allotment);
                }            }
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
            let geometry = rectangle_to_geometry(&anchor);
            let mut campaign= add_rectangle(layer,anchor,&PatinaProcessName::Texture(canvas.clone()),allotments,x_sizes,y_sizes,false)?;
            let patina = layer.get_texture(&geometry,&canvas)?;
            let mut process = layer.get_process_mut(&geometry,&PatinaProcessName::Texture(canvas.clone()))?;
            patina.add_rectangle(&mut process,&mut campaign,gl.bindery(),&canvas,&dims,gl.flat_store())?;
            campaign.close();
        },
        PreparedShape::SpaceBaseRect(area,patina,allotment) => {
            use web_sys::console;
            for ((top_left,bottom_right),allotment) in area.iter().zip(allotment.iter().cycle()) {
                console::log_1(&format!("spacebasearea({:?},{:?},{:?})",top_left,bottom_right,allotment).into());
            }
        }
    }
    Ok(())
}