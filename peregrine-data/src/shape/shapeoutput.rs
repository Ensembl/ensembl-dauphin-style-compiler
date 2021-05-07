use crate::lock;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::sync::{ Arc, Mutex };
use super::shapelist::ShapeList;
use owning_ref::MutexGuardRefMut;
use super::core::{ Patina, AnchorPair, SingleAnchor, track_split, bulk, Pen, Plotter };
use crate::Track;

fn track_sort(tracks: &[Track]) -> (Vec<usize>, Vec<Track>) {
    let mut next_idx = 0;
    let mut fwd = vec![];
    let mut out = vec![];
    let mut rev = HashMap::new();
    for track in tracks {
        let idx = match rev.entry(track) {
            Entry::Occupied(e) => { *e.get() },
            Entry::Vacant(e) => {
                let idx = next_idx;
                next_idx += 1;
                e.insert(idx);
                fwd.push(track.clone());
                idx
            }
        };
        out.push(idx);
    }
    (out,fwd)
}


#[derive(Debug)]
pub struct ShapeOutputData {
    shapes: HashMap<Track,ShapeList>
}

impl ShapeOutputData {
    fn new() -> ShapeOutputData {
        ShapeOutputData {
            shapes: HashMap::new()
        }
    }

    fn track(&mut self, track: &Track) -> &mut ShapeList {
        self.shapes.entry(track.clone()).or_insert_with(|| ShapeList::new())
    }

    fn filter(&self, min_value: f64, max_value: f64) -> ShapeOutputData {
        let mut new_shapes = HashMap::new();
        for (track,shapes) in self.shapes.iter() {
            new_shapes.insert(track.clone(),shapes.filter(min_value,max_value));
        }
        ShapeOutputData {
            shapes: new_shapes
        }
    }
}

#[derive(Clone,Debug)]
pub struct ShapeOutput {
    data: Arc<Mutex<ShapeOutputData>>
}

impl ShapeOutput {
    pub fn new() -> ShapeOutput {
        ShapeOutput {
            data: Arc::new(Mutex::new(ShapeOutputData::new()))
        }
    }

    pub fn track_shapes(&self, track: &Track) -> MutexGuardRefMut<ShapeOutputData,ShapeList> {
        MutexGuardRefMut::new(self.data.lock().unwrap()).map_mut(|x| x.track(track))
    }

    pub fn filter(&self, min_value: f64, max_value: f64) -> ShapeOutput {
        let data = lock!(self.data).filter(min_value,max_value);
        ShapeOutput {
            data: Arc::new(Mutex::new(data))
        }
    }

    pub fn add_rectangle_1(&self, anchors: SingleAnchor, x_size: Vec<f64>, y_size: Vec<f64>, patina: Patina, allotments: Vec<String>, tracks: Vec<Track>) {
        let (track_map,track_names) = track_sort(&tracks);
        let track_map = bulk(track_map,anchors.len(),true);
        let count = anchors.len();
        let anchors = anchors.bulk(count,true);
        let mut anchors = anchors.split(&track_map,true);
        let mut allotments = track_split(allotments,&track_map,false);
        let mut patinas = patina.split(&track_map,false);
        let mut x_size = track_split(x_size,&track_map,false);
        let mut y_size = track_split(y_size,&track_map,false);
        let sz_it = x_size.drain(..).zip(y_size.drain(..));
        let it =
            anchors.drain(..)
            .zip(patinas.drain(..))
            .zip(allotments.drain(..))
            .zip(sz_it);
        for (i,(((anchors,patinas),allotments),(x_size,y_size))) in it.enumerate() {
            self.track_shapes(&track_names[i]).add_rectangle_1(anchors,patinas,allotments,x_size,y_size);
        }
    }

    pub fn add_rectangle_2(&self, anchors: AnchorPair, patina: Patina, allotments: Vec<String>, tracks: Vec<Track>) {
        use web_sys::console;
        console::log_1(&format!("allotments: {:?}",allotments).into());
        let (track_map,track_names) = track_sort(&tracks);
        let track_map = bulk(track_map,anchors.len(),true);
        let count = anchors.len();
        let anchors = anchors.bulk(count,true);
        let mut anchors = anchors.split(&track_map,true);
        let mut allotments = track_split(allotments,&track_map,false);
        let mut patinas = patina.split(&track_map,false);
        let it =
            anchors.drain(..)
            .zip(patinas.drain(..))
            .zip(allotments.drain(..));
        for (i,((anchors,patinas),allotments)) in it.enumerate() {
            self.track_shapes(&track_names[i]).add_rectangle_2(anchors,patinas,allotments);
        }
    }

    pub fn add_text(&self, anchors: SingleAnchor, pen: Pen, text: Vec<String>, allotments: Vec<String>, tracks: Vec<Track>) {
        let (track_map,track_names) = track_sort(&tracks);
        let track_map = bulk(track_map,anchors.len(),true);
        let count = anchors.len();
        let anchors = anchors.bulk(count,true);
        let mut anchors = anchors.split(&track_map,true);
        let mut allotments = track_split(allotments,&track_map,false);
        let mut pen = pen.split(&track_map,false);
        let mut text = track_split(text,&track_map,false);
        let it =
            anchors.drain(..)
            .zip(pen.drain(..))
            .zip(text.drain(..))
            .zip(allotments.drain(..));
        for (i,(((anchors,pen),text),allotments)) in it.enumerate() {
            self.track_shapes(&track_names[i]).add_text(anchors,pen,text,allotments);
        }
    }

    pub fn add_wiggle(&self, min: f64, max: f64, plotter: Plotter, values: Vec<Option<f64>>, allotment: String, track: Track) {
        self.track_shapes(&track).add_wiggle(min,max,plotter,values,allotment);
    }
}
