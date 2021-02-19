use anyhow::bail;
use std::collections::BTreeMap;
use crate::webgl::GPUSpec;

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
            y_offset: self.height_watermark-width,
            height
        });
        let out = self.height_watermark;
        self.height_watermark += height;
        out
    }

    fn allocate(&mut self, width: u32, height: u32) -> (u32,u32) {
        let out = if let Some(offset) = self.allocate_on_existing_shelf(width,height) {
            offset
        } else {
            (0,self.create_new_shelf(width,height))
        };
        self.add_subshelves(height);
        out
    }

    fn height(&self) -> u32 { self.height_watermark }
}

// TODO test this algorithm

fn filter_areas(sizes: &[(u32,u32)], max_area: u64) -> Vec<(u32,u32)> {
    let mut out = vec![];
    let mut sorted = sizes.to_vec();
    sorted.sort_by_key(|(w,h)| ((*w as u64)*(*h as u64)));
    let mut area = 0;
    for (w,h) in sorted {
        let this_area = (w as u64)*(h as u64);
        if area+this_area < max_area {
            out.push((w,h));
            area += this_area;
        } else {
            out.push((0,0));
        }
    }
    out
}

fn try_allocate_areas(sizes: &[(u32,u32)], max_size: u64, max_area: Option<u64>) -> Option<Vec<(u32,u32)>> {
    let mut sorted = if let Some(max_area) = max_area {
        filter_areas(sizes,max_area)
    } else {
        sizes.to_vec()
    };
    sorted.sort_by_key(|(w,h)| (*h,*w));
    sorted.reverse();
    let max_width = sorted.iter().map(|(w,_)| *w as u64).max();
    let square_dim : u64 = sorted.iter().map(|(w,h)| (w*h) as f64).sum::<f64>().sqrt() as u64;
    if let Some(max_width) = max_width {
        let mut texture_width = max_width.max(square_dim).next_power_of_two();
        if texture_width > max_size {
            return None;
        }
        loop {
            let mut out = vec![];
            let mut bin = Bin::new(texture_width as u32);
            for (width,height) in &sorted {
                out.push(bin.allocate(*width,*height));
            }
            let texture_height = bin.height().next_power_of_two() as u64;
            if texture_height <= max_size {
                return Some(out);
            }
            texture_width *= 2;
        }
    } else {
        Some(vec![])
    }
}

pub(crate) fn allocate_areas(sizes: &[(u32,u32)], gpu_spec: &GPUSpec) -> anyhow::Result<Vec<(u32,u32)>> {
    let max_size = gpu_spec.max_texture_size() as u64;
    if let Some(result) = try_allocate_areas(sizes,max_size,None) {
        return Ok(result);
    }
    let mut max_area : u64 = max_size*max_size / 16;
    while max_area > 1 {
        if let Some(result) = try_allocate_areas(sizes,max_size,None) {
            return Ok(result);
        }    
        max_area /= 2;
    }
    bail!("could not generate areas")
}
