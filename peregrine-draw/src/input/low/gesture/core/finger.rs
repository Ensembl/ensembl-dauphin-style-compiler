#[derive(Copy,Clone)]
pub(super) struct OneFingerAxis {
    start: f64,
    current: f64
}

impl OneFingerAxis {
    pub(super) fn new(position: f64) -> OneFingerAxis {
        OneFingerAxis {
            start: position,
            current: position
        }
    }

    pub(super) fn start(&self) -> f64 { self.start }
    pub(super) fn current(&self) -> f64 { self.current }
    pub(super) fn set(&mut self, position: f64) { self.current = position; }
    pub(super) fn reset(&mut self) { self.start = self.current; }
    pub(super) fn delta(&self) -> f64 { self.current - self.start }
}

#[derive(Clone)]
struct OneFingerDelta(OneFingerAxis,OneFingerAxis);

impl OneFingerDelta {
    fn new(position: (f64,f64)) -> OneFingerDelta {
        OneFingerDelta(OneFingerAxis::new(position.0),OneFingerAxis::new(position.1))
    }

    fn start(&self) -> (f64,f64) { (self.0.start(),self.1.start()) }
    fn current(&self) -> (f64,f64) { (self.0.current(),self.1.current()) }
    fn set(&mut self, position: (f64,f64)) { self.0.set(position.0); self.1.set(position.1); }
    fn reset(&mut self) { self.0.reset(); self.1.reset(); }
    fn delta(&self) -> (f64,f64) { (self.0.delta(),self.1.delta()) }
}

#[derive(Clone)]
pub(crate) struct OneFinger {
    overall: OneFingerDelta,
    incremental: OneFingerDelta
}

impl OneFinger {
    pub(super) fn new(pos: (f64,f64)) -> OneFinger {
        OneFinger {
            overall: OneFingerDelta::new(pos),
            incremental: OneFingerDelta::new(pos)
        }
    }

    pub(crate) fn start(&self) -> (f64,f64) { self.overall.start() }
    pub(crate) fn current(&self) -> (f64,f64) { self.overall.current() }

    pub(crate) fn total_delta(&self) -> (f64,f64) {
        self.overall.delta()
    }

    pub(super) fn set(&mut self, position: (f64,f64)) {
        self.overall.set(position);
        self.incremental.set(position);
    }

    pub(crate) fn total_distance(&self) -> f64 {
        let total_delta = self.total_delta();
        total_delta.0.abs() + total_delta.1.abs()
    }

    pub(crate) fn take_delta(&mut self) -> (f64,f64) {
        let delta = self.incremental.delta();
        self.incremental.reset();
        delta
    }
}

#[derive(Clone)]
pub(crate) struct TwoFingers(OneFinger,OneFinger);

impl TwoFingers {
    pub(crate) fn new(primary: (f64,f64), secondary: (f64,f64)) -> TwoFingers {
        TwoFingers(OneFinger::new(primary),OneFinger::new(secondary))
    }

    pub(crate) fn set_position(&mut self, primary: (f64,f64), secondary: (f64,f64)) {
        let order_before = self.0.current().0 > self.1.current().0;
        let order_after = primary.0 > secondary.0;
        let (mut x,mut y) = (&mut self.0,&mut self.1);
        if order_before != order_after {
            (x,y) = (y,x);
        }
        x.set(primary);
        y.set(secondary);
    }

    pub(crate) fn first(&self) -> &OneFinger { &self.0 }
    pub(super) fn second(&self) -> &OneFinger { &self.1 }
    pub(super) fn first_mut(&mut self) -> &mut OneFinger { &mut self.0 }
    pub(super) fn second_mut(&mut self) -> &mut OneFinger { &mut self.1 }

    fn calculate<F,G,X>(&self, access: G, cb: F) -> (X,X) 
            where F: Fn(f64,f64) -> X, G: Fn(&OneFinger) -> (f64,f64) {
        let p = (access)(&self.0);
        let s = (access)(&self.1);
        ((cb)(p.0,s.0),(cb)(p.1,s.1))
    }

    pub(crate) fn current_separation(&self) -> (f64,f64) { 
        self.calculate(|p| p.current(),|p,s| s-p)
    }

    pub(crate) fn start_separation(&self) -> (f64,f64) {
        self.calculate(|p| p.start(),|p,s| s-p)
    }

    pub(crate) fn current_mean(&self) -> (f64,f64) {
        self.calculate(|p| p.current(),|p,s| (s+p)/2.)
    }

    pub(crate) fn start_mean(&self) -> (f64,f64) {
        self.calculate(|p| p.start(),|p,s| (s+p)/2.)
    }
}

pub(crate) enum OneOrTwoFingersInner {
    One(OneFinger),
    Two(TwoFingers)
}

pub(crate) struct OneOrTwoFingers {
    inner: OneOrTwoFingersInner,
    new_secondary: bool
}

impl OneOrTwoFingers {
    pub(super) fn new(primary: (f64,f64), secondary: Option<(f64,f64)>) -> OneOrTwoFingers {
        if let Some(secondary) = secondary {
            OneOrTwoFingers {
                inner: OneOrTwoFingersInner::Two(TwoFingers::new(
                    primary, secondary
                )),
                new_secondary: true
        }
        } else {
            OneOrTwoFingers {
                inner: OneOrTwoFingersInner::One(OneFinger::new(primary)),
                new_secondary: false
            }
        }
    }

    pub(crate) fn primary(&self) -> &OneFinger {
        match &self.inner {
            OneOrTwoFingersInner::One(x) => x,
            OneOrTwoFingersInner::Two(x) => x.first()
        }
    }

    pub(crate) fn primary_mut(&mut self) -> &mut OneFinger {
        match &mut self.inner {
            OneOrTwoFingersInner::One(x) => x,
            OneOrTwoFingersInner::Two(x) => x.first_mut()
        }
    }

    pub(crate) fn secondary(&self) -> Option<&OneFinger> {
        match &self.inner {
            OneOrTwoFingersInner::One(_) => None,
            OneOrTwoFingersInner::Two(x) => Some(x.second())
        }
    }

    pub(crate) fn secondary_mut(&mut self) -> Option<&mut OneFinger> {
        match &mut self.inner {
            OneOrTwoFingersInner::One(_) => None,
            OneOrTwoFingersInner::Two(x) => Some(x.second_mut())
        }
    }

    pub(crate) fn two(&self) -> Option<&TwoFingers> {
        match &self.inner {
            OneOrTwoFingersInner::One(_) => None,
            OneOrTwoFingersInner::Two(x) => Some(x)
        }
    }

    #[allow(unused)]
    pub(crate) fn two_mut(&mut self) -> Option<&mut TwoFingers> {
        match &mut self.inner {
            OneOrTwoFingersInner::One(_) => None,
            OneOrTwoFingersInner::Two(x) => Some(x)
        }
    }

    pub(crate) fn take_upgraded(&mut self) -> bool {
        let upgraded = self.new_secondary;
        self.new_secondary = false;
        upgraded
    }

    pub(super) fn set(&mut self, primary: (f64,f64), secondary: Option<(f64,f64)>) {
        self.primary_mut().set(primary);
        if let Some(secondary) = secondary {
            let new = match &self.inner {
                OneOrTwoFingersInner::One(one) => {
                    self.new_secondary = true;
                    Some(OneOrTwoFingersInner::Two(TwoFingers::new(one.current().clone(),secondary)))
                },
                OneOrTwoFingersInner::Two(_) => { None }
            };
            if let Some(new) = new { self.inner = new; }
            self.secondary_mut().unwrap().set(secondary);
        }
    }
}
