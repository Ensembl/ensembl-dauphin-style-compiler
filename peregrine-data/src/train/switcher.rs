/* Switcher abstracts the train-change operations using type polymorphism. The categories
 * are:
 * 
 * current: the currently displayed train (or fading out, if fading)
 * future: the fading in train, if fading
 * wanted: the train we are currently collecting data for
 * target: the train extent which would be ideal for the given coordinates
 */

pub(super) trait SwitcherManager {
    type Type: SwitcherObject;
    type Extent: SwitcherExtent;
    type Error;

    fn create(&mut self, extent: &Self::Extent) -> Result<Self::Type,Self::Error>;
    fn busy(&self, yn: bool);
}

pub(super) trait SwitcherExtent : Eq {
    type Type: SwitcherObject;
    type Extent: SwitcherExtent;

    fn to_milestone(&self) -> Self::Extent;
    fn is_milestone_for(&self, what: &Self::Extent) -> bool;
}

pub(super) trait SwitcherObject {
    type Extent: SwitcherExtent;
    type Type;
    type Speed;

    /* queries */
    fn extent(&self) -> Self::Extent;
    fn half_ready(&self) -> bool;
    fn ready(&self) -> bool;
    fn broken(&self) -> bool;
    fn speed(&self, source: Option<&Self::Type>) -> Self::Speed;

    /* events */
    fn live(&mut self, speed: &Self::Speed);
    fn dead(&mut self);
}

pub(super) struct Switcher<M: SwitcherManager<Extent=X,Type=T,Error=E>,
                           X: SwitcherExtent<Extent=X,Type=T>+Clone,
                           T: SwitcherObject<Extent=X,Type=T>,
                           E> {
    sketchy: bool,
    was_busy: bool,
    manager: Box<M>,
    current: Option<T>,
    future: Option<T>,
    wanted: Option<T>,
    target: Option<X>
}

impl<M:SwitcherManager<Extent=X,Type=T,Error=E>,
     X:SwitcherExtent<Extent=X,Type=T>+Clone, 
     T:SwitcherObject<Extent=X,Type=T>,
     E> Switcher<M,X,T,E> {
    pub(super) fn new(manager: M) -> Switcher<M,X,T,E> {
        Switcher {
            current: None, future: None, wanted: None, target: None,
            was_busy: false, sketchy: false, 
            manager: Box::new(manager)
        }
    }

    /* The quiescent train is the train which, barring this and any future changes will ultimately be displayed. */
    pub(super) fn quiescent(&self) -> Option<&T> {
        if let Some(wanted) = &self.wanted { return Some(wanted); }
        if let Some(future) = &self.future { return Some(future); }
        if let Some(current) = &self.current { return Some(current); }
        None
    }

    pub(super) fn displaying(&self) -> Option<&T> {
        if let Some(future) = &self.future { return Some(future); }
        if let Some(current) = &self.current { return Some(current); }
        None
    }

    pub(super) fn each_mut<F>(&mut self, cb: &F) where F: Fn(&mut T) {
        if let Some(wanted) = &mut self.wanted { cb(wanted); }
        if let Some(future) = &mut self.future { cb(future); }
        if let Some(current) = &mut self.current { cb(current); }
    }

    pub(super) fn each<F>(&self, cb: &F) where F: Fn(&T) {
        if let Some(wanted) = &self.wanted { cb(wanted); }
        if let Some(future) = &self.future { cb(future); }
        if let Some(current) = &self.current { cb(current); }
    }

    /* something has happened which may cause maneouverability */
    pub(super) fn ping(&mut self) -> Result<(),E> {
        /* try wanted->future in case that is required for target->wanted */
        self.try_wanted_to_future();
        self.try_target_to_wanted()?;
        /* Maybe wanted can fall straight through? */
        self.try_wanted_to_future();
        /* Are we busy? */
        let now_busy = self.future.is_some() || self.wanted.is_some();
        if now_busy != self.was_busy {
            self.was_busy = now_busy;
            self.manager.busy(now_busy);
        }
        Ok(())
    }

    pub(super) fn get_target(&self) -> Option<&X> { self.target.as_ref() }
    pub(super) fn set_target(&mut self, extent: &X) -> Result<(),E> {
        if self.target.is_none() || self.target.as_ref().unwrap() != extent {
            self.target = Some(extent.clone());
            self.ping()?;
        }
        Ok(())
    }

    fn target_matches_wanted(&mut self) -> bool {
        /* No target so say match to avoid trying further */
        if self.target.is_none() { return true; }
        let target = self.target.as_ref().unwrap();
        /* Perfect */
        if let Some(quiescent) = self.quiescent() {
            if &quiescent.extent() == target { return true; }
        }
        /* Good enough for milestones */
        if let Some(wanted) = &self.wanted {
            if wanted.extent().is_milestone_for(target) { return true; }
        }
        false
    }

    fn try_target_to_wanted(&mut self) -> Result<(),E> {
        /* We're heading somewhere already, is it the target? */
        if self.target_matches_wanted() { return Ok(()); }
        let target = self.target.as_ref().unwrap();
        /* Aim direct, or to milestone? */
        let extent = if self.wanted.is_some() || self.sketchy {
            target.to_milestone()
        } else {
            target.clone()
        };
        /* drop old wanted and make milestone, if necessary */      
        if let Some(mut wanted) = self.wanted.take() { wanted.dead(); }
        /* do it */
        self.wanted = Some(self.manager.create(&extent)?);
        Ok(())
    }

    fn try_wanted_to_future(&mut self) {
        /* given the general circumstances, are we good to go? */
        let desperate = self.future.is_none() && self.current.is_none();
        let good_enough = self.wanted.as_ref().map(|x| {
            let ready_enough = x.ready() || (desperate && x.half_ready());
            ready_enough && !x.broken()
        }).unwrap_or(false);
        /* bail unless we can, and want to commit */
        if !good_enough || self.future.is_some() { return; }
        /* do it */
        let wanted = self.wanted.take().unwrap();
        let speed = wanted.speed(self.current.as_ref());
        self.future = Some(wanted);
        self.future.as_mut().unwrap().live(&speed);
    }

    pub(super) fn live_done(&mut self) -> Result<(),E> {
        /* retire current and make future current */
        if let Some(mut current) = self.current.take() {
            current.dead();
        }
        self.current = self.future.take();
        /* now future is free, maybe wanted can go there? */
        self.ping()
    }

    pub(super) fn set_sketchy(&mut self, yn: bool) -> Result<(),E> {
        self.sketchy = yn;
        self.ping()
    }

    pub(super) fn manager(&self) -> &M { &self.manager }
    pub(super) fn manager_mut(&mut self) -> &mut M { &mut self.manager }
}
