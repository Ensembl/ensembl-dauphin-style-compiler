/* A GlobalValue is a value which depends on the entire train's state, not just that of 
 * a single carriage. Examples are track height, screen height, etc. These use the
 * puzzle system to allow different values for different combinations to co-exist.
 * 
 * Each "value" has two parts, a *local* value, the value for that carriage, and a
 * *global* value, the value for this particular train state. Local values are set
 * to puzzle answers (closures) during creation of the CarriageProcess. Note that it
 * is only the *value* of a global value which is global, for lifetime management 
 * reasons, there is a global variable for each CarriageProcess.
 * 
 * Globals are potentially called upon at the time of creation of a DrawingCarriage. 
 * Therefore the resolution process must "fit" between these two endpoints.
 * 
 * Once build & locate, as called by Root, are complete, we know the local value for
 * each variable and so this can be set, along with the global value for the independent
 * answer for this carriage. At this point the (concreate) local value amd the global
 * value (closure) can go into the generated CarriageTrainStateSpec.
 * 
 * Creating a DrawingCarriage requires a TrainState. Therefore, we know we are early
 * enough if we set the global value in the constructor of the TrainState. In the
 * constructor all CarriageTrainStateSpec's are available, so the concrete value can
 * be deretminedand the answer to use is directly in the object.
 * 
 * As TrainState depends on GlobalValue values (and, actually, little else), we need to
 * ensure that the concrete value is hashable or easily converted to a hashable form and
 * that this hash can be recalled quickly as, in practice, equality of TrainState is
 * called upon in many places.
 * 
 * When building a carriage a LocalValueBuilder can be created. This hasan entry() method
 * which allows access to a LocalEntry which allows the local value to be set and the
 * global to be got. Once done, this LocalValueBuilder is conveted into a LocalValueSpec,
 * with the aid of the independent answer. This LocalValueSpec can then be stored with the
 * carriage.
 * 
 * To build for a given set of carriages a GlobalValueBuilder is created and all the
 * relevant LocalValueSpecs added. Once done, the GlobalValueBuilder can be converted into
 * a GlobalValueSpec to add to train state. This constructor also sets the global answers
 * from the start of the process.
 */

use std::{collections::{HashMap, hash_map::DefaultHasher}, fmt::Debug, hash::{Hash, Hasher}, sync::Arc};

use peregrine_toolkit::{puzzle::{UnknownSetter, StaticValue, StaticAnswer, short_unknown_function_promise, Value, constant}, log};

pub(crate) struct LocalEntry<T: 'static+Clone> {
    global_setter: UnknownSetter<'static,StaticValue<T>>,
    global: StaticValue<T>,
    local: Vec<StaticValue<T>>
}

impl<T: 'static+Clone> LocalEntry<T> {
    fn new() -> LocalEntry<T> {
        let (global_setter,global) = short_unknown_function_promise();
        LocalEntry {
            global_setter, global, local: vec![]
        }
    }

    pub(crate) fn add_local(&mut self, local: StaticValue<T>) {
        self.local.push(local);
    }

    pub(crate) fn get_global(&self) -> &StaticValue<T> { &self.global }

    pub(crate) fn set_global(&mut self, answer: &mut StaticAnswer, value: StaticValue<T>) {
        self.global_setter.set(answer,value);
    }
}

pub(crate) struct LocalValueBuilder<X: Hash+Eq+Clone, T:'static+Clone> {
    entries: HashMap<X,LocalEntry<T>>
}

impl<X: Hash+Eq+Clone, T:'static+Clone> LocalValueBuilder<X,T> {
    pub(crate) fn new() -> LocalValueBuilder<X,T> {
        LocalValueBuilder {
            entries: HashMap::new()
        }
    }

    pub(crate) fn entry(&mut self, key: X) -> &mut LocalEntry<T> {
        if !self.entries.contains_key(&key) {
            self.entries.insert(key.clone(),LocalEntry::new());
        }
        self.entries.get_mut(&key).unwrap()
    }
}

#[derive(Clone)]
struct BuiltLocalEntry<T: 'static+Clone> {
    global_setter: UnknownSetter<'static,StaticValue<T>>,
    local_value: T
}

impl<T: 'static+Clone+Debug> Debug for BuiltLocalEntry<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuiltLocalEntry").field("local_value", &self.local_value).finish()
    }
}

impl<T: 'static+Clone> BuiltLocalEntry<T> {
    fn new<F>(entry: &LocalEntry<T>, merger: F, independent_answer: &mut StaticAnswer) -> BuiltLocalEntry<T>
            where F: Fn(&[StaticValue<T>]) -> StaticValue<T> {
        let local_value = merger(&entry.local).call(&independent_answer);
        BuiltLocalEntry {
            global_setter: entry.global_setter.clone(),
            local_value
        }
    }
}

pub(crate) struct LocalValueSpec<X: Hash+Eq+Clone, T:'static+Clone> {
    entries: HashMap<X,BuiltLocalEntry<T>>
}

impl<X: Hash+Eq+Clone, T:'static+Clone> LocalValueSpec<X,T> {
    pub(crate) fn new<F>(builder: &LocalValueBuilder<X,T>, merger: F, independent_answer: &mut StaticAnswer) -> LocalValueSpec<X,T>
            where F: Fn(&[StaticValue<T>]) -> StaticValue<T> {
        let out = LocalValueSpec {
            entries: builder.entries.iter().map(move |(k,v)| {
                let out = (k,BuiltLocalEntry::new(&v,&merger,independent_answer));
                out
            }).map(|(k,v)| (k.clone(),v.clone())).collect()
        };
        out
    }
}

pub(crate) struct GlobalValueBuilder<X:Hash+Eq+Clone, T:'static+Clone> {
    entries: HashMap<X,Vec<BuiltLocalEntry<T>>>
}

impl<X:Hash+Eq+Clone, T:'static+Clone> GlobalValueBuilder<X,T> {
    pub(crate) fn new() -> GlobalValueBuilder<X,T> {
        GlobalValueBuilder {
            entries: HashMap::new()
        }
    }

    pub(crate) fn add(&mut self, spec: &LocalValueSpec<X,T>) {
        for (key,value) in spec.entries.iter() {
            if !self.entries.contains_key(key) {
                self.entries.insert(key.clone(),vec![]);
            }
            self.entries.get_mut(key).unwrap().push(value.clone());
        }
    }
}

#[derive(Clone)]
pub(crate) struct GlobalValueSpec<X:Hash+Eq+Clone, T:'static+Clone> {
    hash: u64,
    entries: Arc<HashMap<X,T>>
}

impl<X:Hash+Eq+Clone, T:'static+Clone> PartialEq for GlobalValueSpec<X,T> {
    fn eq(&self, other: &Self) -> bool { self.hash == other.hash }
}

impl<X:Hash+Eq+Clone, T:'static+Clone> Eq for GlobalValueSpec<X,T> {}

impl<X:Hash+Eq+Clone, T:'static+Clone> Hash for GlobalValueSpec<X,T> {
    fn hash<H: Hasher>(&self, state: &mut H) { self.hash.hash(state); }
}

impl<X:Hash+Eq+Clone+Debug, T:'static+Clone+Debug> std::fmt::Debug for GlobalValueSpec<X,T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalValueSpec").field("entries", &self.entries).finish()
    }
}

impl<X:Hash+Eq+Clone, T:'static+Clone> GlobalValueSpec<X,T> {
    pub(crate) fn new<F,H: Hash>(builder: GlobalValueBuilder<X,T>, merger: F, answer: &mut StaticAnswer) -> GlobalValueSpec<X,T>
            where F: Fn(&[&T]) -> (T,H) {
        let mut hasher = DefaultHasher::new();
        let mut out = HashMap::new();
        for (key,entries) in builder.entries {
            let local_values = entries.iter().map(|x| &x.local_value).collect::<Vec<_>>();
            let (global_value,hash_value) = merger(&local_values[..]);
            key.hash(&mut hasher);
            hash_value.hash(&mut hasher);
            for entry in &entries {
                entry.global_setter.set(answer,constant(global_value.clone()));
            }
            out.insert(key,global_value);
        }
        GlobalValueSpec { 
            hash: hasher.finish(),
            entries: Arc::new(out)
        }
    }

    pub(crate) fn get(&self, key: &X) -> Option<&T> {
        self.entries.get(key)
    }
}
