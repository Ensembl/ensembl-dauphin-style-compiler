/* Consider a set of objects T0, T1, T2, ..., indexed by specifications x0, x1, x2.
 * After startup, exactly one Tk is "current" at any moment in time. External calls set
 * the desired current object from time to time by speficfying an xk. However, creating
 * an object Tk from xk and making such an object current is not an instantaneous 
 * operation. In this case the current object retains its current value until the new
 * object is ready. During this time, a new x, say xm, may be specified. In this case the 
 * work currently in progess on Tk is abandoned and the new Tm is prepared. In addition,
 * even when an object is ready to be current, the transition itself may not be
 * instantaneous, however at this point it must be carried on to completion before the
 * new current is esrablished. It could also be that this process overwhelms execution
 * with rapidly changing requests, which needs to be guarded against. These tasks are the
 * job of Switcher. Siwtcher only performs operations on a regular clock-tick to ping().
 * This helps keep the many concurrent processes under some control.
 * 
 * Some Tm, known as "milestones", may be temporarily "good enough" replacements for a
 * set of other {Ta,Tb,...}. When very busy, the switcher can decide to temporarily render
 * Tm instead of Ta,Tb,... in order to reduce workload. When Tm is being made current,
 * requests for Ta, Tb, etc can be regarded as no-ops until the workload decreases. If
 * the external code sets the variable "sketchy" they are aware of a great deal of
 * activity, and milestones are sufficient until this ends.
 *
 * The following traits must be implemented:
 * SwitcherManager -- a means of creating objects, Tk, from specitications, xk.
 * SwitcherExtent -- a specification, xk.
 * SwitcherObject -- an object Tk.
 * 
 * Throughout the code in this file, the following terminology is used:
 *
 * current: the current object (or transitioning, the object transitioning out).
 * future: the object transitioniing in, if any.
 * wanted: the object currently being constructed, if any.
 * target: the train extent requested by the user.
 * 
 */

 /* The SwitcherManager provides the instance-specific details of a particular
  * implementation. It is also accessible by Switcher's manager() and manager_mut()
  * so can include any instance-specific functionality.
  */
pub(crate) trait SwitcherManager {
    type Type: SwitcherObject;
    type Extent: SwitcherExtent;
    type Error;

    /* create an instance Tk from a spec xk */ 
    fn create(&mut self, extent: &Self::Extent) -> Result<Self::Type,Self::Error>;

    /* callback to the manager when the Party is busy, ie creating or transitioning. */
    fn busy(&self, yn: bool);
}

pub(crate) trait SwitcherExtent : Eq {
    type Type: SwitcherObject;
    type Extent: SwitcherExtent;

    /* convert given specification to its corresponding milestone */
    fn to_milestone(&self) -> Self::Extent;

    /* test whether self serves as a milestone for what */
    fn is_milestone_for(&self, what: &Self::Extent) -> bool;
}

pub(crate) trait SwitcherObject {
    type Extent: SwitcherExtent;
    type Type;
    type Speed;

    /*** Queries made by Party of a SwitcherObject ***/

    /* Return spec, xk, corresponding to this Tk. */
    fn extent(&self) -> Self::Extent;

    /* An object is "half-ready" if it is ready enough to be the _first_ current item.
     * This is usually a lower bar than subsequent current items.
     */
    fn half_ready(&self) -> bool;

    /* An object is "ready" if it can be made current */
    fn ready(&self) -> bool;

    /* An object which is broken must never become current due to a problem during build. */
    fn broken(&self) -> bool;

    /* Transitions can occur at different speeds (speed type is implementation-specific).
     * By comparing the ingoing and outgoing object (if any), a Speed is returned.
     */
    fn speed(&self, source: Option<&Self::Type>) -> Self::Speed;

    /*** Events raised by Party to manipulate a SwitcherObject ***/

    /* Object is now to be the current object, transitioning at the given speed */
    fn live(&mut self, speed: &Self::Speed) -> bool;

    /* Object is no longer current and may be deallocated */
    fn dead(&mut self);
}

pub(crate) struct Switcher<M: SwitcherManager<Extent=X,Type=T,Error=E>,
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
    pub(crate) fn new(manager: M) -> Switcher<M,X,T,E> {
        Switcher {
            current: None, future: None, wanted: None, target: None,
            was_busy: false, sketchy: false, 
            manager: Box::new(manager)
        }
    }

    /* The quiescent object is the object which, barring this and any future changes will
     * ultimately become current.
     */
    pub(crate) fn quiescent(&self) -> Option<&T> {
        if let Some(wanted) = &self.wanted { return Some(wanted); }
        if let Some(future) = &self.future { return Some(future); }
        if let Some(current) = &self.current { return Some(current); }
        None
    }

    /* Call given callback on all objects currently displayed or being prepared. */
    pub(crate) fn each_mut<F>(&mut self, cb: &F) where F: Fn(&mut T) {
        if let Some(wanted) = &mut self.wanted { cb(wanted); }
        if let Some(future) = &mut self.future { cb(future); }
        if let Some(current) = &mut self.current { cb(current); }
    }

    /* Call given callback on all objects currently displayed. */
    pub(crate) fn each_displayed_mut<F>(&mut self, cb: &F) where F: Fn(&mut T) {
        if let Some(future) = &mut self.future { cb(future); }
        if let Some(current) = &mut self.current { cb(current); }
    }

    /* Part of the ping process. A predicate which determines whether we should carry
     * on building "wanted". We are fine if the quiescent value will match the target
     * exactly or if wanted is a milestone for our target and there is currently something
     * in wanted. (Once that has cleared from wanted, this will cause it to skip from the
     * milestone to the actual target: milestones are only acceptable while there is
     * something already filling "wanted").
     */
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

    /* Part of the ping process. The "user" has set the target. If we are not happy
     * with the current value of wanted (assessed by target_matches_wanted), then
     * build a new wanted.
     */
    fn try_target_to_wanted(&mut self) -> Result<(),E> {
        /* Exit if we are happy with the way things are */
        if self.target_matches_wanted() { return Ok(()); }
        let target = self.target.as_ref().unwrap();
        /* Where do we aim: direct, or to milestone? We aim for milestones if
         * there is already something in wanted which we are evicting (thrashing) or if
         * the user has set "sketchy" to indicate there is a great deal of interaction
         * going on.
         */
        let extent = if self.wanted.is_some() || self.sketchy {
            target.to_milestone()
        } else {
            target.clone()
        };
        /* drop old wanted, if necessary */      
        if let Some(mut wanted) = self.wanted.take() { wanted.dead(); }
        /* create a new one */
        self.wanted = Some(self.manager.create(&extent)?);
        Ok(())
    }

    /* Part of the ping process. Is the object currently being constructed in wanted
     * now in a ready-enough state that we can start the transition? Is so, do so.
     */
    fn try_wanted_to_future(&mut self) {
        /* Is wanted ready and is there space in "future"? */
        let desperate = self.future.is_none() && self.current.is_none();
        let good_enough = self.wanted.as_ref().map(|x| {
            let ready_enough = x.ready() || (desperate && x.half_ready());
            ready_enough && !x.broken()
        }).unwrap_or(false);
        if !good_enough || self.future.is_some() { return; }
        /* do it */
        let wanted = self.wanted.take().unwrap();
        let speed = wanted.speed(self.current.as_ref());
        self.future = Some(wanted);
        if self.future.as_mut().unwrap().live(&speed) {
            self.live_done();
        }
    }

    /* Call periodically on a clock "beat" to cause Switcher to act. See the
     * comments on the individual try_* methods for a secription of the process.
     */
    pub(crate) fn ping(&mut self) -> Result<(),E> {
        /* try wanted->future in case that is required for target->wanted */
        self.try_wanted_to_future();
        self.try_target_to_wanted()?;
        /* Maybe wanted can fall straight through? */
        self.try_wanted_to_future();
        /* Update busy in manager */
        let now_busy = self.future.is_some() || self.wanted.is_some();
        if now_busy != self.was_busy {
            self.was_busy = now_busy;
            self.manager.busy(now_busy);
        }
        Ok(())
    }

    /* Utility method for an external caller to see the current target (which they will
     * have set with set_target()) */
    pub(crate) fn get_target(&self) -> Option<&X> { self.target.as_ref() }

    /* Set a new current target */
    pub(crate) fn set_target(&mut self, extent: &X) -> Result<(),E> {
        if self.target.is_none() || self.target.as_ref().unwrap() != extent {
            self.target = Some(extent.clone());
            self.ping()?;
        }
        Ok(())
    }

    /* Called by the object_being made current to indicate that this process is now
     * complete.
     */
    pub(crate) fn live_done(&mut self) -> Result<(),E> {
        /* retire current and make future current */
        if let Some(mut current) = self.current.take() {
            current.dead();
        }
        self.current = self.future.take();
        /* now future is free, maybe wanted can go there? */
        self.ping()
    }

    /* Set "sketchy" mode which is a teporary mode whereby milestones are preferred */
    pub(crate) fn set_sketchy(&mut self, yn: bool) -> Result<(),E> {
        self.sketchy = yn;
        self.ping()
    }

    /* Retrieve manager reference for implementation-specific calls */
    pub(crate) fn manager(&self) -> &M { &self.manager }

    /* Retrieve mutable manager reference  for implementation-specific calls */
    pub(crate) fn manager_mut(&mut self) -> &mut M { &mut self.manager }
}
