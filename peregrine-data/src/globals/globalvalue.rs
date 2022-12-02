/* A GlobalValue is a value which depends on the entire train's state, not just that of 
 * a single carriage. Examples are track height, screen height, etc. These use the
 * puzzle system to allow different values for different combinations to co-exist.
 * 
 * Each "value" has two parts, a *local* value, the value for that carriage, and a
 * *global* value, the value for this particular train state. Local values are set
 * to puzzle answers (closures) during creation of the CarriageBuilder. Note that it
 * is only the *value* of a global value which is global, for lifetime management 
 * reasons, there is a global variable for each CarriageBuilder.
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

/* Throughout this file there is a convention on type placeholders to avoid confusion:
 * T: local type added each time
 * U: consolidated local type
 * V: global type
 */

use std::{collections::{HashMap, hash_map::DefaultHasher}, fmt::Debug, hash::{Hash, Hasher}, sync::Arc};
use peregrine_toolkit::{puzzle::{UnknownSetter, StaticValue, StaticAnswer, short_unknown_function_promise, constant}};

pub(crate) struct LocalEntry<T:'static+Clone, V:'static> {
    global_setter: UnknownSetter<'static,StaticValue<V>>,
    global: StaticValue<V>,
    local: Vec<StaticValue<T>>
}

impl<T: 'static+Clone,V> LocalEntry<T,V> {
    fn new() -> LocalEntry<T,V> {
        let (global_setter,global) = short_unknown_function_promise();
        LocalEntry {
            global_setter, global, local: vec![]
        }
    }

    pub(crate) fn add_local(&mut self, local: StaticValue<T>) {
        self.local.push(local);
    }

    pub(crate) fn get_global(&self) -> &StaticValue<V> { &self.global }
}

pub(crate) struct LocalValueBuilder<X: Hash+Eq+Clone, T:'static+Clone, V:'static> {
    entries: HashMap<X,LocalEntry<T,V>>
}

impl<X: Hash+Eq+Clone, T:'static+Clone,V> LocalValueBuilder<X,T,V> {
    pub(crate) fn new() -> LocalValueBuilder<X,T,V> {
        LocalValueBuilder {
            entries: HashMap::new()
        }
    }

    pub(crate) fn entry(&mut self, key: X) -> &mut LocalEntry<T,V> {
        if !self.entries.contains_key(&key) {
            self.entries.insert(key.clone(),LocalEntry::new());
        }
        self.entries.get_mut(&key).unwrap()
    }
}

#[derive(Clone)]
struct BuiltLocalEntry<U:'static,V:'static> {
    global_setter: UnknownSetter<'static,StaticValue<V>>,
    local_value: StaticValue<U>
}

impl<U:'static, V> BuiltLocalEntry<U,V> {
    fn new<F, T:'static+Clone>(entry: &LocalEntry<T,V>, merger: F) -> BuiltLocalEntry<U,V>
            where F: Fn(&[StaticValue<T>]) -> StaticValue<U> {
        let local_value = merger(&entry.local);
        BuiltLocalEntry {
            global_setter: entry.global_setter.clone(),
            local_value
        }
    }
}

pub(crate) struct LocalValueSpec<X: Hash+Eq+Clone, U:'static+Clone, V:'static+Clone> {
    entries: HashMap<X,BuiltLocalEntry<U,V>>
}

impl<X: Hash+Eq+Clone, U:Clone, V:Clone> LocalValueSpec<X,U,V> {
    pub(crate) fn new<F, T:'static+Clone>(builder: &LocalValueBuilder<X,T,V>, merger: F) -> LocalValueSpec<X,U,V>
            where F: Fn(&[StaticValue<T>]) -> StaticValue<U> {
        let out = LocalValueSpec {
            entries: builder.entries.iter().map(move |(k,v)| {
                let out = (k,BuiltLocalEntry::new(&v,&merger));
                out
            }).map(|(k,v)| (k.clone(),v.clone())).collect()
        };
        out
    }
}

pub(crate) struct GlobalValueBuilder<X:Hash+Eq+Clone, U:'static+Clone, V:'static+Clone> {
    entries: HashMap<X,Vec<BuiltLocalEntry<U,V>>>
}

impl<X:Hash+Eq+Clone, U:'static+Clone, V:Clone> GlobalValueBuilder<X,U,V> {
    pub(crate) fn new() -> GlobalValueBuilder<X,U,V> {
        GlobalValueBuilder {
            entries: HashMap::new()
        }
    }

    pub(crate) fn add(&mut self, spec: &LocalValueSpec<X,U,V>) {
        for (key,value) in spec.entries.iter() {
            if !self.entries.contains_key(key) {
                self.entries.insert(key.clone(),vec![]);
            }
            self.entries.get_mut(key).unwrap().push(value.clone());
        }
    }

    fn stable_entries(&self) -> Vec<(&X,&Vec<BuiltLocalEntry<U,V>>)> {
        let mut keys = self.entries.keys().collect::<Vec<_>>();
        keys.sort_by_cached_key(|x| {
            let mut h = DefaultHasher::new();
            x.hash(&mut h);
            h.finish()
        });
        let mut out = vec![];
        for key in keys {
            out.push((key,self.entries.get(key).unwrap()));
        }
        out
    }
}

#[derive(Clone)]
pub(crate) struct GlobalValueSpec<X:Hash+Eq+Clone, V> {
    hash: u64,
    entries: Arc<HashMap<X,V>>
}

impl<X:Hash+Eq+Clone, V> PartialEq for GlobalValueSpec<X,V> {
    fn eq(&self, other: &Self) -> bool { self.hash == other.hash }
}

impl<X:Hash+Eq+Clone, V> Eq for GlobalValueSpec<X,V> {}

impl<X:Hash+Eq+Clone, V> Hash for GlobalValueSpec<X,V> {
    fn hash<H: Hasher>(&self, state: &mut H) { self.hash.hash(state); }
}

impl<X:Hash+Eq+Clone+Debug, V:Debug> std::fmt::Debug for GlobalValueSpec<X,V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GlobalValueSpec").field("entries", &self.entries).finish()
    }
}

impl<X:Hash+Eq+Clone, V:Clone> GlobalValueSpec<X,V> {
    pub(crate) fn new<F, U:'static+Clone, H:Hash>(builder: GlobalValueBuilder<X,U,V>, merger: F, answer: &mut StaticAnswer) -> GlobalValueSpec<X,V>
            where F: Fn(&X,&[&StaticValue<U>],&mut StaticAnswer) -> (V,H) {
        let mut hasher = DefaultHasher::new();
        let mut out = HashMap::new();
        for (key,entries) in builder.stable_entries() {
            let local_values = entries.iter().map(|x| &x.local_value).collect::<Vec<_>>();
            let (global_value,hash_value) = merger(&key,&local_values[..],answer);
            key.hash(&mut hasher);
            hash_value.hash(&mut hasher);
            for entry in entries {
                entry.global_setter.set(answer,constant(global_value.clone()));
            }
            out.insert(key.clone(),global_value);
        }
        GlobalValueSpec { 
            hash: hasher.finish(),
            entries: Arc::new(out)
        }
    }

    pub(crate) fn add<U: Clone>(&self, local: &LocalValueSpec<X,U,V>, answer: &mut StaticAnswer) {
        for (key,entry) in &local.entries {
            if let Some(global_value) = self.entries.get(key) {
                entry.global_setter.set(answer,constant(global_value.clone()));
            }
        }
    }

    pub(crate) fn get(&self, key: &X) -> Option<&V> {
        self.entries.get(key)
    }
}
