use std::{collections::{HashMap, BTreeMap, hash_map::DefaultHasher}, hash::{Hash, Hasher}, rc::Rc, sync::{Arc, Mutex}};
use peregrine_toolkit::{puzzle::{UnknownSetter, StaticValue, short_unknown_function_promise, commute, StaticAnswer, constant, derived}, lock};
use crate::{allotment::{core::allotmentname::AllotmentName, collision::collisionalgorithm::{BumpRequestSet, BumpResponses}}, request};
use super::{playingfield::PlayingFieldEdge, trainpersistent::TrainPersistent, allotmentmetadata::LocalAllotmentMetadataBuilder};

struct CorrelateBuildEntry<T:'static, V:'static> {
    global_setter: UnknownSetter<'static,StaticValue<V>>,
    global: StaticValue<V>,
    local: Vec<StaticValue<T>>
}

impl<T: 'static+Clone,V> CorrelateBuildEntry<T,V> {
    fn new() -> CorrelateBuildEntry<T,V> {
        let (global_setter,global) = short_unknown_function_promise();
        CorrelateBuildEntry {
            global_setter, global, local: vec![]
        }
    }

    fn add_local(&mut self, local: StaticValue<T>) {
        self.local.push(local);
    }

    fn get_global(&self) -> &StaticValue<V> { &self.global }
}

#[derive(Clone)]
struct CorrelateEntry<U:'static,V:'static> {
    global_setter: UnknownSetter<'static,StaticValue<V>>,
    local_value: StaticValue<U>
}

trait SpecificCorrelator {
    type Item;
    type Carriage;
    type Train;

    fn items_to_carriage(&self, values_in: &[StaticValue<Self::Item>]) -> StaticValue<Self::Carriage>;
    fn carriage_to_train(&self, values_in: &[StaticValue<Self::Carriage>], answer: &mut StaticAnswer, hasher: &mut DefaultHasher, persistent: &Arc<Mutex<TrainPersistent>>) -> Self::Train;
}

struct MaxFloatCorrelator;

impl SpecificCorrelator for MaxFloatCorrelator {
    type Item = f64;
    type Carriage = f64;
    type Train = f64;

    fn items_to_carriage(&self, v: &[StaticValue<Self::Item>]) -> StaticValue<Self::Carriage> {
        commute(v,0.,|x,y| x.max(*y)).derc()
    }

    fn carriage_to_train(&self, values_in: &[StaticValue<Self::Carriage>], answer: &mut StaticAnswer, hasher: &mut DefaultHasher, persistent: &Arc<Mutex<TrainPersistent>>) -> Self::Train {
        let value = values_in.iter().map(|x| x.call(answer)).fold(f64::NEG_INFINITY,f64::max);
        ((value*10000.).round() as i64).hash(hasher);
        value
    }
}

struct BumpingCorrelator;

impl SpecificCorrelator for BumpingCorrelator {
    type Item = (AllotmentName,Rc<BumpRequestSet>);
    type Carriage = (AllotmentName,Rc<BumpRequestSet>);
    type Train = BumpResponses;

    fn items_to_carriage(&self, v: &[StaticValue<Self::Item>]) -> StaticValue<Self::Carriage> {
        v[0].clone() // multiple should be impossible
    }

    fn carriage_to_train(&self, values_in: &[StaticValue<Self::Carriage>], answer: &mut StaticAnswer, hasher: &mut DefaultHasher, persistent: &Arc<Mutex<TrainPersistent>>) -> Self::Train {
        let requests = values_in.iter().map(|x| x.call(answer)).collect::<Vec<_>>();
        let name = requests.iter().next().cloned().unwrap().0;
        let requests = requests.iter().map(|(_,x)| x.clone()).collect::<Vec<_>>();
        let mut persistent = lock!(persistent);
        let (out,hash) = persistent.bump_mut(&name).make(&requests);
        hash.hash(hasher);
        out
    }
}

fn items_to_carriage<K,I,C,T>(method: &dyn SpecificCorrelator<Item=I,Carriage=C,Train=T>, mut input: HashMap<K,CorrelateBuildEntry<I,T>>)
            -> HashMap<K,CorrelateEntry<C,T>>
            where K: PartialEq+Eq+Hash {
    input.drain().map(|(k,v)| {
        (k,CorrelateEntry {
            global_setter: v.global_setter.clone(),
            local_value: method.items_to_carriage(&v.local)
        })
    }).collect()
}

fn carriage_to_train<K,I,C,T: Clone>(method: &dyn SpecificCorrelator<Item=I,Carriage=C,Train=T>, 
                                input: BTreeMap<K,Vec<CorrelateEntry<C,T>>>,
                                answer: &mut StaticAnswer,
                                hasher: &mut DefaultHasher,
                                persistent: &Arc<Mutex<TrainPersistent>>) -> BTreeMap<K,T>
                                where K: PartialOrd + Ord {
    let mut output = BTreeMap::new();
    for (key,values_in) in input {
        let local = values_in.iter().map(|x| x.local_value.clone()).collect::<Vec<_>>();
        let value = method.carriage_to_train(&local,answer,hasher,persistent);
        for entry in &values_in {
            entry.global_setter.set(answer,constant(value.clone()));
        }
        output.insert(key,value);
    }
    output
}

struct CarriageCorrelateBuilder {
    align: HashMap<String,CorrelateBuildEntry<f64,f64>>,
    playing_field: HashMap<PlayingFieldEdge,CorrelateBuildEntry<f64,f64>>,
    height_tracker: HashMap<AllotmentName,CorrelateBuildEntry<f64,f64>>,
    bumper: HashMap<AllotmentName,CorrelateBuildEntry<(AllotmentName,Rc<BumpRequestSet>),BumpResponses>>,
    //reporting: LocalAllotmentMetadataBuilder
}

impl CarriageCorrelateBuilder {
    fn new() -> CarriageCorrelateBuilder {
        CarriageCorrelateBuilder {
            align: HashMap::new(),
            playing_field: HashMap::new(),
            height_tracker: HashMap::new(),
            bumper: HashMap::new()
        }
    }

    fn add_align(&mut self, name: &str, value: StaticValue<f64>) {
        self.align.entry(name.to_string()).or_insert_with(|| CorrelateBuildEntry::new()).add_local(value);
    }

    fn add_playing_field(&mut self, name: &PlayingFieldEdge, value: StaticValue<f64>) {
        self.playing_field.entry(name.clone()).or_insert_with(|| CorrelateBuildEntry::new()).add_local(value);
    }

    fn add_height_tracker(&mut self, name: &AllotmentName, value: StaticValue<f64>) {
        self.height_tracker.entry(name.clone()).or_insert_with(|| CorrelateBuildEntry::new()).add_local(value);
    }

    fn add_bumper(&mut self, name: &AllotmentName, value: StaticValue<Rc<BumpRequestSet>>) {
        let name2 = name.clone();
        let value = derived(value, move |v| (name2.clone(),v));
        self.bumper.entry(name.clone()).or_insert_with(|| CorrelateBuildEntry::new()).add_local(value);
    }
}

struct CarriageCorrelate {
    align: HashMap<String,CorrelateEntry<f64,f64>>,
    playing_field: HashMap<PlayingFieldEdge,CorrelateEntry<f64,f64>>,
    height_tracker: HashMap<AllotmentName,CorrelateEntry<f64,f64>>,
    bumper: HashMap<AllotmentName,CorrelateEntry<(AllotmentName,Rc<BumpRequestSet>),BumpResponses>>
}

impl CarriageCorrelate {
    fn new(mut builder: CarriageCorrelateBuilder) -> CarriageCorrelate {
        CarriageCorrelate {
            align: items_to_carriage(&MaxFloatCorrelator,builder.align),
            playing_field: items_to_carriage(&MaxFloatCorrelator,builder.playing_field),
            height_tracker: items_to_carriage(&MaxFloatCorrelator,builder.height_tracker),
            bumper: items_to_carriage(&BumpingCorrelator,builder.bumper),
        }
    }
}

struct TrainCorrelateBuilder {
    align: BTreeMap<String,Vec<CorrelateEntry<f64,f64>>>,
    playing_field: BTreeMap<PlayingFieldEdge,Vec<CorrelateEntry<f64,f64>>>,
    height_tracker: BTreeMap<AllotmentName,Vec<CorrelateEntry<f64,f64>>>,
    bumper: BTreeMap<AllotmentName,Vec<CorrelateEntry<(AllotmentName,Rc<BumpRequestSet>),BumpResponses>>>
}

fn add_carriage<X,Y>(target: &mut BTreeMap<X,Vec<Y>>, source: &HashMap<X,Y>)
        where X: PartialEq+Eq+Ord+Clone, Y: Clone {
    for (key,value) in source.iter() {
        target.entry(key.clone()).or_insert_with(|| vec![]).push(value.clone());
    }
}

impl TrainCorrelateBuilder {
    fn new() -> TrainCorrelateBuilder {
        TrainCorrelateBuilder {
            align: BTreeMap::new(),
            playing_field: BTreeMap::new(),
            height_tracker: BTreeMap::new(),
            bumper: BTreeMap::new(),
        }
    }

    fn add(&mut self, carriage: &CarriageCorrelate) {
        add_carriage(&mut self.align,&carriage.align);
        add_carriage(&mut self.playing_field,&carriage.playing_field);
        add_carriage(&mut self.height_tracker,&carriage.height_tracker);
        add_carriage(&mut self.bumper,&carriage.bumper);
    }
}

impl PartialEq for TrainCorrelate {
    fn eq(&self, other: &Self) -> bool { self.hash == other.hash }
}

impl Eq for TrainCorrelate {}

impl Hash for TrainCorrelate {
    fn hash<H: Hasher>(&self, state: &mut H) { self.hash.hash(state); }
}

struct TrainCorrelate {
    align: BTreeMap<String,f64>,
    playing_field: BTreeMap<PlayingFieldEdge,f64>,
    height_tracker: BTreeMap<AllotmentName,f64>,
    bumper: BTreeMap<AllotmentName,BumpResponses>,
    hash: u64
}

impl TrainCorrelate {
    fn new(builder: TrainCorrelateBuilder, answer: &mut StaticAnswer, persistent: &Arc<Mutex<TrainPersistent>>) -> TrainCorrelate {
        let mut hasher = DefaultHasher::new();
        let align = carriage_to_train(&MaxFloatCorrelator,builder.align,answer,&mut hasher,persistent);
        let playing_field = carriage_to_train(&MaxFloatCorrelator,builder.playing_field,answer,&mut hasher,persistent);
        let height_tracker = carriage_to_train(&MaxFloatCorrelator,builder.height_tracker,answer,&mut hasher,persistent);
        let bumper = carriage_to_train(&BumpingCorrelator,builder.bumper,answer,&mut hasher,persistent);
        TrainCorrelate {
            align, playing_field, height_tracker, bumper,
            hash: hasher.finish()
        }
    }

    fn get_align(&mut self, name: &str) -> Option<f64> { self.align.get(name).cloned() }
    fn get_playing_field(&mut self, name: &PlayingFieldEdge) -> Option<f64> { self.playing_field.get(name).cloned() }
    fn get_height_tracker(&mut self, name: &AllotmentName) -> Option<f64> { self.height_tracker.get(name).cloned() }
    fn get_bumper(&mut self, name: &AllotmentName) -> Option<BumpResponses> { self.bumper.get(name).cloned() }
}
