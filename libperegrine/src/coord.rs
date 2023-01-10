use eachorevery::EachOrEvery;
use eard_interp::{GlobalBuildContext, GlobalContext, Return, HandleStore, Value};
use peregrine_data::SpaceBase;

fn coord_to_eoe(ctx: &GlobalContext, reg: usize) -> Result<EachOrEvery<f64>,String> {
    Ok(if ctx.is_finite(reg)? {
        EachOrEvery::each(ctx.force_finite_number(reg)?.to_vec())
    } else {
        EachOrEvery::every(ctx.force_infinite_number(reg)?)
    })
}

pub(crate) fn op_coord(gctx: &GlobalBuildContext) -> Result<Box<dyn Fn(&mut GlobalContext,&[usize]) -> Result<Return,String>>,String> {
    let coords = gctx.patterns.lookup::<HandleStore<SpaceBase<f64,()>>>("coords")?;
    Ok(Box::new(move |ctx,regs| {
        let b = coord_to_eoe(ctx,regs[1])?;
        let n = coord_to_eoe(ctx,regs[2])?;
        let t = coord_to_eoe(ctx,regs[3])?;
        let coords = ctx.context.get_mut(&coords);
        let sb = SpaceBase::new(&b, &n, &t,&EachOrEvery::every(()));
        if sb.is_none() {
            return Err(format!("coordinates had incompatible lengths"));
        }
        let h = coords.push(sb.unwrap());
        ctx.set(regs[0],Value::Number(h as f64))?;
        Ok(Return::Sync)
    }))
}
