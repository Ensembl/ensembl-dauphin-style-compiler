use peregrine_toolkit::{boom::Boom, log};
use super::bumprequest::BumpRequest;

struct WallRow(Boom<i64>);

impl WallRow {
    fn new() -> WallRow { WallRow(Boom::new()) }

    fn add_interval(&mut self, start: i64, end: i64) -> bool {
        let mut iter = self.0.seek_mut(&(start+1));
        iter.rewind();
        iter.rewind();
        while let Some((match_start,match_end)) = iter.next() {
            if match_start >= end { break; }
            if *match_end > start { return false; }
        }
        self.0.insert(start,end);
        true
    }
}

pub(super) struct Wall {
    row_height: Option<i64>,
    rows: Vec<WallRow>
}

impl Wall {
    pub(super) fn new() -> Wall {
        Wall {
            row_height: None,
            rows: vec![]
        }
    }

    pub(super) fn verify(&mut self, requests: &[BumpRequest]) -> bool {
        let height_here = requests.iter().map(|h| h.height as i64).max();
        match (self.row_height,height_here) {
            (Some(old),Some(new)) => {  if old < new { return false; } }
            (None,new) => { self.row_height = new; }
            _ => {}
        }
        true
    }

    pub(super) fn total_height(&self) -> f64 { (self.rows.len() as i64 * self.row_height.unwrap_or(0)) as f64 }

    pub(super) fn renew(&mut self, start: i64, end: i64, offset: f64) {
        let row_height = self.row_height.expect("no row height. should never happen");
        let row = (offset as i64/row_height) as usize;
        if row > self.rows.len() {
            self.rows.resize_with(row+1,|| WallRow::new());
        }
        self.rows[row].add_interval(start,end);
    }

    pub(super) fn allocate(&mut self, start: i64, end: i64) -> f64 {
        let row_height = self.row_height.expect("no row height. should never happen") as usize;
        for (i,row) in self.rows.iter_mut().enumerate() {
            if row.add_interval(start,end) {
                return (i * row_height) as f64;
            }
        }
        let row = self.rows.len();
        self.rows.push(WallRow::new());
        self.rows[row].add_interval(start,end);
        (row * row_height) as f64
    }
}
