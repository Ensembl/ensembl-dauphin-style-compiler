use std::sync::Arc;

use peregrine_toolkit::puzzle::{PuzzlePiece, Puzzle, PuzzleValueHolder, DerivedPuzzlePiece, PuzzleValue, PuzzleSolution, ClonablePuzzleValue, ConstantPuzzlePiece};

use crate::{AllotmentMetadata, allotment::core::arbitrator::Arbitrator, AllotmentMetadataRequest, MetadataMergeStrategy};

pub struct AllotmentBoxBuilder {
    puzzle: Puzzle,
    padding_top: f64,
    padding_bottom: f64,
    min_height: Option<f64>,
    children: Vec<AllotmentBox>,
    self_indent: Option<PuzzleValueHolder<f64>>,
    current_bottom: PuzzleValueHolder<f64>,
    final_bottom: PuzzlePiece<f64>
}

impl AllotmentBoxBuilder {
    pub fn new(arbitrator: &Arbitrator, metadata: &AllotmentMetadata, natural_height: f64, self_indent: Option<&PuzzleValueHolder<f64>>) -> AllotmentBoxBuilder {
        AllotmentBoxBuilder {
            puzzle: arbitrator.puzzle().clone(),
            padding_top:  metadata.get_f64("padding-top").unwrap_or(0.),
            padding_bottom: metadata.get_f64("padding-bottom").unwrap_or(0.),
            min_height: metadata.get_f64("min-height"),
            children: vec![],
            self_indent: self_indent.cloned(),
            current_bottom: PuzzleValueHolder::new(ConstantPuzzlePiece::new(natural_height)),
            final_bottom: arbitrator.puzzle().new_piece(Some(0.))
        }
    }

    pub fn empty(arbitrator: &Arbitrator, natural_height: f64, self_indent: &Option<PuzzleValueHolder<f64>>) -> AllotmentBoxBuilder {
        AllotmentBoxBuilder {
            puzzle: arbitrator.puzzle().clone(),
            padding_top: 0.,
            padding_bottom: 0.,
            min_height: None,
            children: vec![],
            self_indent: self_indent.clone(),
            current_bottom: PuzzleValueHolder::new(ConstantPuzzlePiece::new(natural_height)),
            final_bottom: arbitrator.puzzle().new_piece(Some(0.))
        }
    }

    fn unpadded_height_piece(&self) -> PuzzleValueHolder<f64> {
        let min_height = self.min_height.unwrap_or(0.);
        PuzzleValueHolder::new(DerivedPuzzlePiece::new(self.current_bottom.clone(),move |height| height.max(min_height)))
    }

    fn unpadded_height(&self, solution: &PuzzleSolution) -> f64 {
        self.current_bottom.get_clone(solution).max(self.min_height.unwrap_or(0.))
    }

    fn padded_height_piece(&self) -> PuzzleValueHolder<f64> {
        let unpadded_height = self.unpadded_height_piece();
        let padding_top = self.padding_top;
        let padding_bottom = self.padding_bottom;
        PuzzleValueHolder::new(DerivedPuzzlePiece::new(unpadded_height,move |unpadded_height| {
            unpadded_height + padding_top + padding_bottom
        }))
    }

    /* don't make visible except to AllotmentBox */
    fn padded_height(&self, solution: &PuzzleSolution) -> f64 {
        self.unpadded_height(solution) + self.padding_top + self.padding_bottom
    }

    /* don't make visible except to AllotmentBox */
    fn apply_root(&self, solution: &mut PuzzleSolution, container_offset: f64) {
        for child in &self.children {
            child.apply_root(solution,container_offset);
        }
    }

    /* don't make visible except to AllotmentBox */
    fn apply_indent(&self, solution: &mut PuzzleSolution, indent: f64) {
        for child in &self.children {
            child.apply_indent(solution,indent);
        }
    }

    pub fn append(&mut self, child: AllotmentBox) {
        let old_bottom = self.current_bottom.clone();
        let padded_height = self.padded_height_piece().clone();
        let bottom = self.puzzle.new_piece(None);
        bottom.add_solver(&[old_bottom.dependency(),padded_height.dependency()], move |solution| {
            Some(old_bottom.get_clone(solution) + padded_height.get_clone(solution))
        });
        self.current_bottom = PuzzleValueHolder::new(bottom.clone());
        child.container_offset_piece().add_solver(&vec![bottom.dependency()], move |solution| {
            Some(bottom.get_clone(solution))
        });
        self.children.push(child);
    }

    pub fn overlay(&mut self, child: AllotmentBox) {
        let top = self.padding_top.clone();
        child.container_offset_piece().add_solver(&vec![], move |solution| {
            Some(top)
        });
    }

    pub fn append_all(&mut self, mut children: Vec<AllotmentBox>) {
        for b in children.drain(..) {
            self.append(b);
        }
    }

    pub fn overlay_all(&mut self, mut children: Vec<AllotmentBox>) {
        for b in children.drain(..) {
            self.overlay(b);
        }
    }

    fn finalise(&mut self) {
        let bottom = self.current_bottom.clone();
        self.final_bottom.add_solver(&[bottom.dependency()], move |solution| {
            Some(bottom.get_clone(solution))
        });
    }
}

#[derive(Clone)]
pub struct AllotmentBox {
    indent: PuzzlePiece<f64>,
    offset_from_container: PuzzlePiece<f64>,
    offset_from_root: PuzzlePiece<f64>,
    allot_box: Arc<AllotmentBoxBuilder>,
}

impl AllotmentBox {
    pub fn new(mut builder: AllotmentBoxBuilder) -> AllotmentBox {
        builder.finalise();
        AllotmentBox {
            offset_from_container: builder.puzzle.new_piece(Some(0.)),
            offset_from_root: builder.puzzle.new_piece(Some(0.)),
            indent: builder.puzzle.new_piece(Some(0.)),
            allot_box: Arc::new(builder)
        }
    }

    fn apply_root(&self, solution: &mut PuzzleSolution, container_offset: f64) {
        let offset_from_root = self.offset_from_container.get_clone(solution) + (container_offset as f64);
        self.offset_from_root.set_answer(solution,offset_from_root);
        self.allot_box.apply_root(solution,offset_from_root);
    }

    fn apply_indent(&self, solution: &mut PuzzleSolution, container_indent: f64) {
        let indent = self.allot_box.self_indent.as_ref().map(|x| x.get_clone(solution)).unwrap_or(container_indent as f64);
        self.indent.set_answer(solution,indent);
        self.allot_box.apply_indent(solution,indent);
    }

    pub fn set_root(&self, solution: &mut PuzzleSolution, container_offset: f64, indent: f64) {
        self.apply_root(solution,container_offset);
        self.apply_indent(solution,indent);
    }

    pub fn container_offset_piece(&self) -> &PuzzlePiece<f64> { &self.offset_from_container }
    pub fn total_height(&self, solution: &PuzzleSolution) -> f64 { self.allot_box.padded_height(solution) }

    pub fn top_delayed(&self) -> PuzzleValueHolder<f64> { PuzzleValueHolder::new(self.offset_from_root.clone()) }

    pub fn bottom_delayed(&self) -> PuzzleValueHolder<f64> {
        let offset_from_root = self.offset_from_root.clone();
        let padded_height = self.padded_height_piece();
        let piece = self.allot_box.puzzle.new_piece(None);
        piece.add_solver(&[padded_height.dependency(),self.offset_from_root.dependency()], move |solution| {
            Some(offset_from_root.get_clone(solution) + padded_height.get_clone(solution))
        });
        PuzzleValueHolder::new(piece)
    }

    fn padded_height_piece(&self) -> PuzzleValueHolder<f64> {
        let piece = self.allot_box.puzzle.new_piece(Some(0.));
        let bottom = self.allot_box.current_bottom.clone();
        let padding_top = self.allot_box.padding_top.clone();
        let padding_bottom = self.allot_box.padding_bottom.clone();
        let min_height = self.allot_box.min_height;
        piece.add_solver(&[bottom.dependency()], move |solution| {
            let unpadded_height = bottom.get_clone(solution).max(min_height.unwrap_or(0.));
            let padded_height = unpadded_height + padding_top + padding_bottom;
            Some(padded_height)
        });
        PuzzleValueHolder::new(piece)
    }

    fn padded_height(&self, solution: &PuzzleSolution) -> f64 {
        let bottom = self.allot_box.current_bottom.get_clone(solution);
        let unpadded_height = bottom.max(self.allot_box.min_height.unwrap_or(0.));
        unpadded_height + self.allot_box.padding_top + self.allot_box.padding_bottom
    }

    // XXX these accessors to post-solution object
    pub fn padding_top(&self) -> f64 { self.allot_box.padding_top }
    pub fn top(&self, solution: &PuzzleSolution) -> f64 { self.offset_from_root.get_clone(solution) }
    pub fn bottom(&self, solution: &PuzzleSolution) -> f64 { self.top(solution) + self.total_height(solution) }
    pub fn draw_top(&self, solution: &PuzzleSolution) -> f64 { self.top(solution) + self.padding_top() }
    pub fn draw_bottom(&self, solution: &PuzzleSolution) -> f64 { self.draw_top(solution) + self.allot_box.unpadded_height(solution) }
    pub fn indent(&self, solution: &PuzzleSolution) -> f64 { self.indent.get_clone(solution) }

    pub fn indent_delayed(&self) -> PuzzleValueHolder<f64> { PuzzleValueHolder::new(self.indent.clone()) }

    pub fn add_transform_metadata(&self, solution: &PuzzleSolution, out: &mut AllotmentMetadataRequest) {
        out.add_pair("type","track",&MetadataMergeStrategy::Replace);
        out.add_pair("offset",&self.top(solution).to_string(),&MetadataMergeStrategy::Minimum);
        out.add_pair("height",&(self.bottom(solution)-self.top(solution)).to_string(),&MetadataMergeStrategy::Maximum);
    }
}

pub struct SolvedAllotmentBox<'a> {
    solution: &'a PuzzleSolution,
    allot_box: Arc<AllotmentBox>
}

impl<'a> SolvedAllotmentBox<'a> {
    pub fn new(allot_box: &AllotmentBox, solution: &'a PuzzleSolution) -> SolvedAllotmentBox<'a> {
        SolvedAllotmentBox { solution, allot_box: Arc::new(allot_box.clone()) }
    }

    pub fn total_height(&self) -> f64 { self.allot_box.allot_box.padded_height(&self.solution) }
    pub fn padding_top(&self) -> f64 { self.allot_box.allot_box.padding_top }
    pub fn top(&self) -> f64 { self.allot_box.offset_from_root.get_clone(&self.solution) }
    pub fn bottom(&self) -> f64 { self.top() + self.total_height() }
    pub fn draw_top(&self) -> f64 { self.top() + self.padding_top() }
    pub fn draw_bottom(&self) -> f64 { self.draw_top() /*+ self.allot_box.unpadded_height(&self.solution)*/ }
    pub fn indent(&self) -> f64 { self.allot_box.indent.get_clone(&self.solution) }
}
