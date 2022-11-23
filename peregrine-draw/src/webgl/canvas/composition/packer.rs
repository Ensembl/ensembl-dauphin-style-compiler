use std::collections::BTreeMap;
use peregrine_toolkit::{error::Error};
use crate::webgl::GPUSpec;

use super::areabuilder::CanvasItemAreaBuilder;

/* see alloc.md in guide for details */

const SPLIT_FACTOR_NUM : u32 = 1;
const SPLIT_FACTOR_DENOM : u32 = 4;

struct Shelf {
    y_offset: u32,
    height: u32
}

/* All shelves of a given width (not specified) */
struct Shelves(BTreeMap<u32,Vec<Shelf>>);

impl Shelves {
    fn new() -> Shelves {
        Shelves(BTreeMap::new())
    }

    fn add(&mut self, width: u32, shelf: Shelf) {
        self.0.entry(width).or_insert_with(|| vec![]).push(shelf);
    }

    fn take_min_width(&mut self, min_width: u32) -> Option<(u32,Shelf)> {
        let mut out = None;
        let mut empty = None;
        if let Some((found_width,shelves)) = self.0.range_mut(min_width..).next() {
            let shelf = shelves.pop().unwrap();
            out = Some((*found_width,shelf));
            if shelves.len() == 0 {
                empty = Some(*found_width);
            }
        }
        if let Some(width) = empty {
            self.0.remove(&width);
        }
        out
    }
}

struct Bin {
    shelves: Shelves,
    pending_shelves: BTreeMap<u32,Vec<(u32,Shelf)>>,
    height_watermark: u32,
    width: u32
}

impl Bin {
    fn new(width: u32) -> Bin {
        Bin {
            shelves: Shelves::new(),
            pending_shelves: BTreeMap::new(),
            height_watermark: 0,
            width
        }
    }

    fn add_subshelves(&mut self, height: u32) {
        let mut empty = None; // will only catch one, but next will catch it.
        for (empty_height,shelves) in self.pending_shelves.range_mut(height..) {
            for (width,shelf) in shelves.drain(..) {
                self.shelves.add(width,shelf);
            }
            empty = Some(*empty_height);
        }
        if let Some(height) = empty {
            self.pending_shelves.remove(&height);
        }
    }

    fn maybe_split(&mut self, shelf: &mut Shelf, width: u32, height: u32)  {
        let spare_height = shelf.height-height;
        if spare_height > 0 && spare_height*SPLIT_FACTOR_DENOM > SPLIT_FACTOR_NUM*shelf.height {
            self.pending_shelves.entry(spare_height).or_insert_with(|| vec![]).push((width,Shelf {
                y_offset: shelf.y_offset+height,
                height: spare_height
            }));
            shelf.height = height;
        }
    }

    /* returns (x_offset, y_offset) */
    fn allocate_on_existing_shelf(&mut self, width: u32, height: u32) -> Option<(u32,u32)> {
        if let Some((found_width,mut shelf)) = self.shelves.take_min_width(width) {
            let new_width = found_width - width;
            let y_offset = shelf.y_offset;
            if new_width > 0 {
                self.maybe_split(&mut shelf,new_width,height);
                self.shelves.add(new_width,shelf);
            }
            Some((self.width-found_width,y_offset))
        } else {
            None
        }
    }

    /* returns y_offset */
    fn create_new_shelf(&mut self, width: u32, height: u32) -> u32 {
        let new_width = self.width - width;
        self.shelves.add(new_width,Shelf {
            y_offset: self.height_watermark,
            height
        });
        let out = self.height_watermark;
        self.height_watermark += height;
        out
    }

    fn allocate(&mut self, width: u32, height: u32) -> (u32,u32) {
        let out = if let Some(offset) = self.allocate_on_existing_shelf(width+1,height+1) {
            offset
        } else {
            (0,self.create_new_shelf(width+1,height+1))
        };
        self.add_subshelves(height+1);
        out
    }

    fn height(&self) -> u32 { self.height_watermark }
}

// TODO test this algorithm

fn tallest_first(sizes: &[(u32,u32)]) -> Vec<usize> {
    let mut sorted = sizes.iter().enumerate().collect::<Vec<_>>();
    sorted.sort_by_key(|(_,(w,h))| (*h,*w));
    sorted.reverse();
    sorted.iter().map(|x| x.0).collect()
}

fn initial_width(sizes: &[(u32,u32)]) -> u64 {
    let max_width = sizes.iter().map(|(w,_)| *w as u64).max().unwrap()+1;
    let square_dim : u64 = sizes.iter().map(|(w,h)| (w*h) as f64).sum::<f64>().sqrt() as u64;
    max_width.max(square_dim).next_power_of_two()
}

fn attempt_at_width(order: &[usize], sizes: &[(u32,u32)], texture_width: u64) -> (Vec<(u32,u32)>,u32) {
    let mut out = vec![(0,0);order.len()];
    let mut bin = Bin::new(texture_width as u32);
    for index in order {
        let area = &sizes[*index];
        out[*index] = bin.allocate(area.0,area.1);
    }
    (out,bin.height())
}

pub(crate) fn allocate_areas(items: &mut [&mut CanvasItemAreaBuilder], gpu_spec: &GPUSpec) -> Result<(u32,u32),Error> {
    let sizes = items.iter().map(|x| x.size()).collect::<Vec<_>>();
    if sizes.len() == 0 { return Ok((1,1)); }
    let order = tallest_first(&sizes);
    let max_size = gpu_spec.max_texture_size() as u64;
    let mut texture_width = initial_width(&sizes);
    loop {
        if texture_width > max_size {
            return Err(Error::fatal("cannot pack rectangles: all attempts failed"));
        }
        let (out,texture_height) = attempt_at_width(&order,&sizes,texture_width);
        let texture_height = texture_height.next_power_of_two() as u64;
        if texture_height <= max_size {
            for (i,origin) in out.iter().enumerate() {
                items[i].set_origin(*origin);
            }
            return Ok((texture_width as u32,texture_height as u32));
        }
        texture_width *= 2;
    }
}

pub(crate) fn allocate_linear(items: &mut [&mut CanvasItemAreaBuilder], gpu_spec: &GPUSpec, horizontal: bool) -> Result<(u32,u32),Error> {
    if items.len() == 0 { return Ok((1,1)) }
    let (stack,other) = if horizontal { (0,1) } else { (1,0) };
    let mut cur  = vec![0,0];
    let mut max = vec![0,0];
    for item in items.iter_mut() {
        item.set_origin((cur[0],cur[1]));
        let size = item.size();
        let size = vec![size.0,size.1];
        cur[stack] += size[stack];
        max[other] = max[other].max(size[other]);
    }
    let mut size = (max[0].max(cur[0]),max[1].max(cur[1]));
    let max_size = gpu_spec.max_texture_size();
    if size.0 > max_size || size.1 > max_size {
        return Err(Error::fatal("cannot pack rectangles: all attempts failed"));
    }
    size.0 = size.0.next_power_of_two();
    size.1 = size.1.next_power_of_two();
    Ok(size)
}
