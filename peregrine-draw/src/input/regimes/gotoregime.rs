use crate::input::translate::measure::Measure;
use super::regime::{RegimeCreator, RegimeTrait, TickResult, TickUpdate};

/* See Wijk and Nuij, IEEE Trans. Vis. Comput. Graph. 10(4), for algorithm, symbol meanings, and motivation.
 * The unsusual single letter variable names are per that paper.
 */

pub(super) struct GotoRegimeCreator {
    pub max_s: f64,
    pub rho: f64,
    pub v: f64
}

impl RegimeCreator for GotoRegimeCreator {
    type Object = GotoRegime;

    fn create(&self) -> Self::Object {
        GotoRegime::new(self.rho,self.v,self.max_s)
    }
}

trait GotoAlgortihm {
    fn total_distance(&self) -> f64;
    fn tick(&self, s: f64) -> (f64,f64,bool);
}

struct SimpleGoto {
    x: f64,
    bp: f64
}

impl GotoAlgortihm for SimpleGoto {
    fn total_distance(&self) -> f64 { 0. }
    fn tick(&self, _s: f64) -> (f64,f64,bool) { (self.x,self.bp,true) }
}

struct FullGoto {
    rho: f64,
    alpha: f64, /* (w0/rho^2)*cosh(r0) */
    beta: f64, /* (w0/rho^2)*sinh(r0)-u0 */
    gamma: f64, /* w0*cosh(r0) */
    r0: f64,
    s: f64,
    reverse: bool
}

impl FullGoto {
    fn b(u_diff: f64, w: (f64,f64), w_i: f64, rho: f64, sign: f64) -> f64 {
        let numer = (w.1*w.1) - (w.0*w.0) + sign*(rho*rho*rho*rho)*(u_diff*u_diff);
        let denom = 2.0*w_i*(rho*rho)*u_diff;
        numer/denom
    }

    fn r(b: f64) -> f64 {
        /* top approximates bottom, but top form avoids big-big underflow, 
         * important when taking ln straight afterwards! Approximation should be out by less than 
         * 1ppm when b>1000., thankfully! When less than 1000, the bottom does fine. There is 
         * probably cinsiderable leaway in the choice of 1000 as a cutoff.
         * 
         * At 1000 exact formula gives -7.60090271, approximation gives -7.60090246. THis is the
         * point of maximum percentage disparity.
         */
        if b > 1000. {
            -((2.*b).ln())
        } else {
            ((b*b+1.).sqrt()-b).ln()
        }
    }

    fn new(regime: &GotoRegime, x: (f64,f64), bp: (f64,f64)) -> FullGoto {
        let reverse = x.0 > x.1;
        let (x,bp) = if reverse { ((x.1,x.0),(bp.1,bp.0)) } else { (x,bp) };
        let r0 = FullGoto::r(FullGoto::b(x.1-x.0,bp,bp.0,regime.rho,1.));
        let r1 = FullGoto::r(FullGoto::b(x.1-x.0,bp,bp.1,regime.rho,-1.));
        let w0_over_rho2 = bp.0 / regime.rho / regime.rho;
        FullGoto {
            rho: regime.rho,
            alpha: w0_over_rho2 * r0.cosh(),
            beta: w0_over_rho2 * r0.sinh() - x.0,
            gamma: bp.0*r0.cosh(),
            r0,
            s: (r1-r0)/regime.rho,
            reverse
        }
    }
}

impl GotoAlgortihm for FullGoto {
    fn total_distance(&self) -> f64 { self.s }

    fn tick(&self, s: f64) -> (f64,f64,bool) {
        let s = if self.reverse { self.s-s } else { s };
        let r = self.rho*s+self.r0;
        (self.alpha * r.tanh() - self.beta , self.gamma / r.cosh() , false)
    }
}

struct ZoomOnlyGoto {
    k_rho: f64,
    u: f64,
    w0: f64,
    s: f64
}

impl ZoomOnlyGoto {
    fn new(regime: &GotoRegime, x: f64, bp: (f64,f64)) -> ZoomOnlyGoto {
        ZoomOnlyGoto {
            k_rho: regime.rho * (bp.1-bp.0).signum(),
            u: x,
            w0: bp.0,
            s: (bp.1/bp.0).ln().abs()/regime.rho
        }
    }
}

impl GotoAlgortihm for ZoomOnlyGoto {
    fn total_distance(&self) -> f64 { self.s }
    fn tick(&self, s: f64) -> (f64,f64,bool) { (self.u,self.w0*((self.k_rho * s).exp()),false) }
}

struct GotoInstance {
    algorithm: Box<dyn GotoAlgortihm>,
    v: f64,
    s: f64,
    t_seen: f64
}

impl GotoInstance {
    fn new(regime: &GotoRegime, x: (f64,f64), bp: (f64,f64)) -> GotoInstance {
        let mut algorithm : Box<dyn GotoAlgortihm> = if x.0 == x.1 {
            Box::new(ZoomOnlyGoto::new(regime,x.0,bp))
        } else {
            Box::new(FullGoto::new(regime,x,bp))
        };
        if algorithm.total_distance() > regime.max_s {
            algorithm = Box::new(SimpleGoto { x: x.1, bp: bp.1 });
        }
        GotoInstance {
            t_seen: 0.,
            s: algorithm.total_distance(),
            v: regime.v,
            algorithm
        }
    }

    fn tick(&mut self, total_dt: f64) -> (TickResult,bool) {
        self.t_seen += total_dt;
        let s = self.t_seen * self.v;
        let (x,bp,force_fade) = self.algorithm.tick(s.min(self.s));
        let finished = s >= self.s;
        (TickResult::Update(TickUpdate { x: Some(x), bp: Some(bp), force_fade }),finished)
    }
}

pub(crate) struct GotoRegime {
    rho: f64,
    v: f64,
    max_s: f64,
    start: Option<(f64,f64)>,
    goto: Option<GotoInstance>
}

impl GotoRegime {
    pub(crate) fn new(rho: f64, v: f64, max_s: f64) -> GotoRegime {
        GotoRegime {
            goto: None,
            start: None,
            rho, v, max_s
        }
    }

    pub(crate) fn goto(&mut self, x: Option<f64>, bp: Option<f64>) {        
        if let Some((start_x,start_bp)) = &self.start {
            let x = x.unwrap_or(*start_x);
            let bp = bp.unwrap_or(*start_bp);
            let instance = GotoInstance::new(self,(*start_x,x),(*start_bp,bp));
            self.goto = Some(instance);
        }
    }
}

impl RegimeTrait for GotoRegime {
    fn set_size(&mut self, measure: &Measure, _size: Option<f64>) {
        self.start = Some((measure.x_bp,measure.bp_per_screen));
    }

    fn tick(&mut self, _measure: &Measure, total_dt: f64) -> TickResult {
        let (res,done) = if let Some(instance) = &mut self.goto {
            instance.tick(total_dt)
        } else {
            (TickResult::Finished,true)
        };
        if done {
            self.goto = None;
        }
        res
    }

    fn update_settings(&mut self, measure: &Measure) {
        self.start = Some((measure.x_bp,measure.bp_per_screen));
    }
}
