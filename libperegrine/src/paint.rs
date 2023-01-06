use eard_interp::{GlobalBuildContext, GlobalContext, HandleStore, Value, Return};
use peregrine_data::{Colour, DirectColour, Patina, DrawnType};
use peregrine_toolkit::eachorevery::EachOrEvery;

fn to_u8(v: f64) -> u8 { (v*255.) as u8 }

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
