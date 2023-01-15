use eachorevery::EachOrEvery;
use eard_interp::{GlobalContext, HandleStore};

pub(crate) fn eoe_from_handle<T: Clone>(ctx: &GlobalContext, input: &HandleStore<T>, reg: usize) -> Result<EachOrEvery<T>,String> {
    Ok(if !ctx.is_finite(reg)? {
        EachOrEvery::every(input.get(ctx.force_infinite_number(reg)? as usize)?.clone())
    } else if ctx.is_atomic(reg)? {
        EachOrEvery::every(input.get(ctx.force_number(reg)? as usize)?.clone())
    } else {
        EachOrEvery::each(
            ctx.force_finite_number(reg)?.iter().map(|h| {
                input.get(*h as usize).cloned()
            }).collect::<Result<Vec<_>,_>>()?
        )
    })
}
