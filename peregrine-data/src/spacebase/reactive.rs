use std::{sync::{Arc, Mutex, Weak}, borrow::{BorrowMut}, mem::replace};

use peregrine_toolkit::lock;

/* Variable: a value which can be changed
 * Variable.set(X) -- set new Value
 * Variable.observable() -- make observable for this variable
 *
 * Observable: a thing which can be observed
 * Observable.get() -> X -- current value
 *
 * Observer -- wraps a callback to be called when an Observable changes
 * Observer::new() -- create
 * Observer.observe(observable) -- trigger on this observable
 * 
 * Reactive -- top level object
 * Reactive::new() -- create
 * Reactive.observable() -- create new observable
 * Reactive.run_observers() -- trigger anything updated since last run
 * 
 * Internally, Observers "hold" the callback and are in turn held by any Observables which they observe.
 * This means that when all an Observer's go out of scope, the Observable is tidied as well.
 * An Observable also holds a weak reference to ObserverRunner, the data of the Reactive object. Reactive itself holds
 * the strong reference. If all the Reactives go out of scope, we don't need to worry about updating as nothing can
 * be triggered! Otherwise a set causes the Observable to push all its Observers onto the stack. Observers have a count
 * which prevents multiple pushes. 
 * 
 * The reason that Variables and Observables are separable is the potential for a reference loop. We typically want to
 * get a value in the callback. If Observable were merged into Variable, The closure would contain a reference to
 * Variable, Variable contain a reference to the Observer and the Observer a reference to the closure. Observable only
 * has a weak reference to the list of Observers, weakening that loop. The strong reference is held by Variable. If this
 * goes out of scope then it's ok to do nothing on an observe call as it can never change.
 */

struct TriggerList<'a> {
    observers: Vec<Weak<Mutex<ObserverData<'a>>>>
}

impl<'a> TriggerList<'a> {
    fn new() -> TriggerList<'a> {
        TriggerList {
            observers: vec![]
        }
    }

    fn observe(&mut self, observer: &Weak<Mutex<ObserverData<'a>>>) {
        self.observers.push(observer.clone());
    }

    fn tidy(&mut self) {
        self.observers = self.observers.drain(..)
            .filter(|x| Weak::upgrade(x).is_some())
            .collect::<Vec<_>>();
    }

    fn trigger(&mut self, runner: &Weak<Mutex<ObserverRunner<'a>>>) {
        let mut needs_tidying = false;
        if let Some(runner) = Weak::upgrade(&runner) {
            for observer in self.observers.iter_mut() {
                if let Some(observer) = Weak::upgrade(observer) {
                    lock!(observer).trigger(&runner);
                } else {
                    needs_tidying = true;
                }
            }
        }
        if needs_tidying {
            self.tidy();
        }
    }

    #[cfg(debug_assertions)]
    fn count_observers(&self) -> usize {
        self.observers.len()
    }
}

 #[derive(Clone)]
 pub struct Variable<'a,X> {
    runner: Weak<Mutex<ObserverRunner<'a>>>,
    value: Arc<Mutex<X>>,
    triggers: Arc<Mutex<TriggerList<'a>>>
 }

impl<'a,X: Clone> Variable<'a,X> {
    fn new(value: X, runner: &Weak<Mutex<ObserverRunner<'a>>>) -> Variable<'a,X> {
        Variable {
            runner: runner.clone(),
            value: Arc::new(Mutex::new(value)),
            triggers: Arc::new(Mutex::new(TriggerList::new()))
        }
    }

    fn trigger(&self) {
        lock!(self.triggers).trigger(&self.runner);
    }

    pub fn set(&mut self, value: X) {
        *lock!(self.value) = value;
        self.trigger();
    }

    pub fn observable(&self) -> Observable<'a,X> {
        Observable::new(&self.value,&Arc::downgrade(&self.triggers),&self.runner)
    }
}

#[derive(Clone)]
pub struct Observable<'a,X: Clone> {
    runner: Weak<Mutex<ObserverRunner<'a>>>,
    triggers: Weak<Mutex<TriggerList<'a>>>,
    value: Arc<Mutex<X>>,
}

#[cfg(debug_assertions)]
impl<'a,X: Clone+std::fmt::Debug> std::fmt::Debug for Observable<'a,X> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let observers = Weak::upgrade(&self.triggers).map(|t| lock!(t).count_observers()).unwrap_or(0);
        write!(f,"Observable({:?} [{} observers])",lock!(self.value),observers)
    }
}

impl<'a,X: Clone> Observable<'a,X> {
    fn new(value: &Arc<Mutex<X>>, triggers: &Weak<Mutex<TriggerList<'a>>>, runner: &Weak<Mutex<ObserverRunner<'a>>>) -> Observable<'a,X> {
        Observable {
            value: value.clone(),
            runner: runner.clone(),
            triggers: triggers.clone()
        }
    }

    pub fn constant(value: X) -> Observable<'a,X> {
        Observable {
            value: Arc::new(Mutex::new(value)),
            runner: Weak::new(),
            triggers: Weak::new()
        }
    }

    fn observe(&self, observer: &Weak<Mutex<ObserverData<'a>>>) {
        if let Some(triggers) = Weak::upgrade(&self.triggers) {
            lock!(triggers).observe(observer);
        }
        if let Some(runner) = Weak::upgrade(&self.runner) {
            if let Some(observer) = Weak::upgrade(observer) {
                lock!(observer).trigger(&runner);
            }
        }
    }

    pub fn get(&self) -> X {
        lock!(self.value).clone()
    }
}

struct ObserverData<'a> {
    cb: Arc<Mutex<Box<dyn FnMut() + 'a>>>,
    index: u64
}

impl<'a> ObserverData<'a> {
    fn trigger(&mut self, runner: &Arc<Mutex<ObserverRunner<'a>>>) {
        lock!(runner).trigger(self.borrow_mut());
    }
}

#[cfg(debug_assertions)]
impl<'a> std::fmt::Debug for Observer<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f,"Observer(...)")
    }
}

#[derive(Clone)]
pub struct Observer<'a>(Arc<Mutex<ObserverData<'a>>>);

impl<'a> Observer<'a> {
    pub fn new<F>(cb: F) -> Observer<'a> where F: FnMut() + 'a {
        Self::new_boxed(Box::new(cb))
    }

    pub fn new_boxed(cb: Box<dyn FnMut() + 'a>) -> Observer<'a> {
        Observer(Arc::new(Mutex::new(ObserverData { cb: Arc::new(Mutex::new(cb)), index: 0 })))
    }

    pub fn observe<X: Clone>(&mut self, observable: &Observable<'a,X>) {
        observable.observe(&Arc::downgrade(&self.0));
    }
}

struct ObserverRunner<'a> {
    current_trigger: u64,
    pending: Vec<Arc<Mutex<Box<dyn FnMut() + 'a>>>>
}

impl<'a> ObserverRunner<'a> {
    fn new() -> ObserverRunner<'a> {
        ObserverRunner {
            current_trigger: 1,
            pending: vec![]
        }
    }

    fn trigger(&mut self, runnable: &mut ObserverData<'a>) {
        if runnable.index < self.current_trigger {
            runnable.index = self.current_trigger;
            self.pending.push(runnable.cb.clone());
        }
    }

    fn get_triggered(&mut self) -> Vec<Arc<Mutex<Box<dyn FnMut() + 'a>>>> {
        self.current_trigger += 1;
        replace(&mut self.pending, vec![])
    }
}

#[derive(Clone)]
pub struct Reactive<'a> {
    runner: Arc<Mutex<ObserverRunner<'a>>>
}

impl<'a> Reactive<'a> {
    pub fn new() -> Reactive<'a> {
        Reactive {
            runner: Arc::new(Mutex::new(ObserverRunner::new()))
        }
    }

    pub fn variable<X: Clone>(&self, value: X) -> Variable<'a,X> {
        Variable::new(value,&Arc::downgrade(&self.runner))
    }

    pub fn run_observers(&self) {
        let triggered = lock!(self.runner).get_triggered();
        for cb in &triggered {
            (lock!(cb))();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct Counter<X: Clone + 'static>(Observer<'static>,Arc<Mutex<(usize,Option<X>)>>,Dropped);

    impl<X: Clone> Counter<X> {
        fn new(variable: &Variable<'static,X>) -> Counter<X> {
            let data = Arc::new(Mutex::new((0,None)));
            let data2 = data.clone();
            let (dropped,canary) = Dropped::new();
            let ob = variable.observable();
            let ob2 = ob.clone();
            let observer = Observer::new(move || {
                let v = ob2.get();
                let mut data = lock!(data2);
                data.0 += 1;
                data.1 = Some(v);
                canary.refer();
            });
            Counter(observer,data,dropped)
        }

        fn observe(&mut self, variable: &Variable<'static,X>) {
            self.0.observe(&variable.observable());
        }

        fn count(&self) -> usize { lock!(self.1).0 }
        fn dropped(&self) -> Dropped { self.2.clone() }
    }

    struct DropCanary(Arc<Mutex<bool>>);

    impl DropCanary {
        fn refer(&self) {}
    }

    impl Drop for DropCanary {
        fn drop(&mut self) {
            *lock!(self.0) = true;
        }
    }

    #[derive(Clone)]
    struct Dropped(Arc<Mutex<bool>>);

    impl Dropped {
        fn new() -> (Dropped,DropCanary) {
            let flag = Arc::new(Mutex::new(false));
            let canary = DropCanary(flag.clone());
            (Dropped(flag),canary)
        }
        fn assert(&self) -> bool { *lock!(self.0) }
    }

    #[test]
    fn reactive_smoke() {
        let reactive = Reactive::new();
        let mut v1 = reactive.variable(0);
        let mut v2 = reactive.variable(1);
        let mut c1 = Counter::new(&v1);
        let mut c2 = Counter::new(&v2);
        let c1_dropped = c1.dropped();
        c1.observe(&v1);
        c1.observe(&v2);
        c2.observe(&v1);
        reactive.run_observers();
        assert_eq!(1,c1.count());
        assert_eq!(1,c2.count());
        v1.set(42);
        assert_eq!(1,c1.count());
        assert_eq!(1,c2.count());
        reactive.run_observers();
        assert_eq!(2,c1.count());
        assert_eq!(2,c2.count());
        v2.set(43);
        v2.set(43);
        assert_eq!(2,c1.count());
        assert_eq!(2,c2.count());
        reactive.run_observers();
        assert_eq!(3,c1.count());
        assert_eq!(2,c2.count());
        reactive.run_observers();
        assert_eq!(3,c1.count());
        assert_eq!(2,c2.count());
        drop(v1);
        drop(v2);
        drop(c1);
        drop(c2);
        assert!(c1_dropped.assert());
    }

    #[test]
    fn reactive_observer_drop() {
        let reactive = Reactive::new();
        let v1 = reactive.variable(0);
        let mut c1 = Counter::new(&v1);
        c1.observe(&v1);
        let dropped = c1.dropped();
        drop(c1);
        reactive.run_observers();
        assert!(dropped.assert());
    }
}