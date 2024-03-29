use std::sync::Arc;

use eachorevery::{EachOrEvery, eoestruct::{StructTemplate, StructValue, StructBuilt}};
use eard_interp::{GlobalBuildContext, GlobalContext, HandleStore, Value, Return, ContextItem};
use peregrine_data::{Colour, DirectColour, Patina, DrawnType, Plotter, Pen, AttachmentPoint, HotspotPatina, ShapeRequest, DataRequest};
use crate::{util::eoe_from_handle};

fn to_u8(v: f64) -> u8 { v as u8 }

pub(crate) fn op_colour(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    Ok(Box::new(move |ctx,regs| {
        let r = to_u8(ctx.force_number(regs[1])?);
        let g = to_u8(ctx.force_number(regs[2])?);
        let b = to_u8(ctx.force_number(regs[3])?);
        let a = to_u8(ctx.force_number(regs[4])?);
        let colours = ctx.context.get_mut(&colours);
        let h = colours.push(Colour::Direct(DirectColour(r,g,b,a)));
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_solid(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    Ok(Box::new(move |ctx,regs| {
        let h = ctx.force_number(regs[1])? as usize;
        let colours = ctx.context.get_mut(&colours);
        let colour = colours.get(h)?.clone();
        let paint = Patina::Drawn(DrawnType::Fill,EachOrEvery::every(colour));
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(paint);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_hollow(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    Ok(Box::new(move |ctx,regs| {
        let width = ctx.force_number(regs[2])?;
        let h = ctx.force_number(regs[1])? as usize;
        let colours = ctx.context.get_mut(&colours);
        let colour = colours.get(h)?.clone();
        let paint = Patina::Drawn(DrawnType::Stroke(width),EachOrEvery::every(colour));
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(paint);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_solid_s(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    Ok(Box::new(move |ctx,regs| {
        let value = if ctx.is_finite(regs[1])? {
            let h = ctx.force_finite_number(regs[1])?;
            let colours = ctx.context.get(&colours);
            let colour = h.iter().map(|h| {
                colours.get(*h as usize).cloned()
            }).collect::<Result<Vec<_>,_>>()?;
            EachOrEvery::each(colour)
        } else {
            let h = ctx.force_infinite_number(regs[1])? as usize;
            let colours = ctx.context.get(&colours);
            let colour = colours.get(h)?.clone();
            EachOrEvery::every(colour)
        };
        let paint = Patina::Drawn(DrawnType::Fill,value);
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(paint);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_hollow_s(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    Ok(Box::new(move |ctx,regs| {
        let value = if ctx.is_finite(regs[1])? {
            let h = ctx.force_finite_number(regs[1])?;
            let colours = ctx.context.get(&colours);
            let colour = h.iter().map(|h| {
                colours.get(*h as usize).cloned()
            }).collect::<Result<Vec<_>,_>>()?;
            EachOrEvery::each(colour)
        } else {
            let h = ctx.force_infinite_number(regs[1])? as usize;
            let colours = ctx.context.get(&colours);
            let colour = colours.get(h)?.clone();
            EachOrEvery::every(colour)
        };
        let width = ctx.force_number(regs[2])?;
        let paint = Patina::Drawn(DrawnType::Stroke(width),value);
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(paint);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_special(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    Ok(Box::new(move |ctx,regs| {
        let special = ctx.force_string(regs[1])?;
        let hover = ctx.force_boolean(regs[2])?;
        let paint = Patina::Hotspot(HotspotPatina::Special(EachOrEvery::every(special.to_string())),hover);
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(paint);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

fn to_direct(colour: &Colour) -> Result<&DirectColour,String> {
    match colour {
        Colour::Direct(c) => Ok(c),
        _ => Err(format!("graph colour must be simple colour"))
    }
}

fn to_direct_seq(ctx: &GlobalContext, input: &HandleStore<Colour>, reg: usize) -> Result<EachOrEvery<DirectColour>,String> {
    Ok(eoe_from_handle(ctx,input,reg)?.map_results(|c| to_direct(c).cloned())?)
}

pub(crate) fn op_graph_type(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let graph_types = gctx.patterns.lookup::<HandleStore<Plotter>>("graph-types")?;
    Ok(Box::new(move |ctx,regs| {
        let height = ctx.force_number(regs[1])?;
        let colour_handle = ctx.force_number(regs[2])? as usize;
        let colours = ctx.context.get(&colours);
        let colour = to_direct(colours.get(colour_handle)?)?.clone();
        let graph_type = Plotter(height, colour);
        let graph_types = ctx.context.get_mut(&graph_types);
        let h = graph_types.push(graph_type);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_pen(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let pens = gctx.patterns.lookup::<HandleStore<Pen>>("pens")?;
    Ok(Box::new(move |ctx,regs| {
        let font = ctx.force_string(regs[1])?.to_string();
        let size = ctx.force_number(regs[2])?;
        let (size,attachment) = if size < 0. {
            (-size,AttachmentPoint::Right)
        } else {
            (size,AttachmentPoint::Left)
        };
        let colours = ctx.context.get(&colours);
        let fgd = to_direct_seq(ctx,colours,regs[3])?.clone();
        let bgd = to_direct_seq(ctx,colours,regs[4])?.clone();
        let pen = Pen::new(&font,size as u32,&fgd,&bgd,&attachment);
        let pens = ctx.context.get_mut(&pens);
        let h = pens.push(pen);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_zmenu(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    Ok(Box::new(move |ctx,regs| {
        let templates = ctx.context.get(&templates);
        let variety_h = ctx.force_number(regs[1])? as usize;
        let variety = templates.get(variety_h)?.clone();
        let content_h = ctx.force_number(regs[2])? as usize;
        let content = templates.get(content_h)?.clone();
        let hover = ctx.force_boolean(regs[3])?;
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(Patina::Hotspot(HotspotPatina::Click(Arc::new(variety),Arc::new(content)),hover));
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_dotted(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let colours = gctx.patterns.lookup::<HandleStore<Colour>>("colours")?;
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    Ok(Box::new(move |ctx,regs| {
        let colours = ctx.context.get(&colours);
        let length = ctx.force_number(regs[3])?;
        let width = ctx.force_number(regs[4])?;
        let prop = ctx.force_number(regs[5])?;
        let colour_a = to_direct_seq(ctx,colours,regs[1])?;
        let colour_b= to_direct_seq(ctx,colours,regs[2])?;
        let colour = colour_a.zip(&colour_b,|a,b| {
            Colour::Bar(a.clone(),b.clone(),(length as u32,length as u32),prop)
        });
        let paint = Patina::Drawn(DrawnType::Stroke(width),colour);
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(paint);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_metadata(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    Ok(Box::new(move |ctx,regs| {
        let templates = ctx.context.get(&templates);
        let key = ctx.force_string(regs[1])?.to_string();
        let values_id = ctx.force_finite_string(regs[2])?;
        let values_h = ctx.force_finite_number(regs[3])?;
        let values = values_h.iter().zip(values_id.iter()).map(|(h,id)| {
            let build = templates.get(*h as usize)?.build()?;
            let value = StructValue::new_expand(&build,None)?;
            Ok::<_,String>((id.to_string(),value))
        }).collect::<Result<Vec<_>,_>>()?;
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(Patina::Metadata(key.to_string(),EachOrEvery::each(values)));
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}

pub(crate) fn op_paint_setting(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let paints = gctx.patterns.lookup::<HandleStore<Patina>>("paint")?;
    let templates = gctx.patterns.lookup::<HandleStore<StructTemplate>>("eoetemplates")?;
    let shape_request = gctx.patterns.lookup::<ShapeRequest>("shape-request")?;
    Ok(Box::new(move |ctx,regs| {
        let setting = ctx.force_string(regs[1])?;
        let templates = ctx.context.get(&templates);
        let shape_request = ctx.context.get(&shape_request);
        let hover = ctx.force_boolean(regs[4])?;
        let paint = if let Some(switch) = shape_request.track().underlying_switch(setting) {
            let value = ctx.force_finite_number(regs[3])?.iter().map(|h| {
                templates.get(*h as usize)
            });
            let updates = ctx.force_finite_string(regs[2])?
                .iter().zip(value).map(|(key,value)| {
                    Ok::<_,String>((key.to_string(),value?.build()?))
            }).collect::<Result<Vec<_>,String>>()?;
            Patina::Hotspot(HotspotPatina::Setting(switch.clone(),EachOrEvery::each(updates)),hover)
        } else {
            Patina::Hotspot(HotspotPatina::Setting(vec![],EachOrEvery::each(vec![])),hover)
        };
        let paints = ctx.context.get_mut(&paints);
        let h = paints.push(paint);
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}
