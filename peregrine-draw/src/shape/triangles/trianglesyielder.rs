use crate::{Message, shape::layers::geometry::{GeometryProcessName, GeometryProgram, GeometryYielder}, webgl::ProgramBuilder};
use super::trianglesprogram::TrackTrianglesProgram;

pub(crate) struct TrackTrianglesYielder {
    geometry_process_name: GeometryProcessName,
    track_triangles: Option<TrackTrianglesProgram>
}

impl<'a> GeometryYielder for TrackTrianglesYielder {
    fn name(&self) -> &GeometryProcessName { &self.geometry_process_name }

    fn set(&mut self, program: &GeometryProgram) -> Result<(),Message> {
        self.track_triangles = Some(match program {
            GeometryProgram::Triangles(_,prog) => prog,
            _ => { Err(Message::CodeInvariantFailed(format!("mismatched program: tracktriangles")))? }
        }.clone());
        Ok(())
    }
}

impl TrackTrianglesYielder {
    pub(crate) fn new(geometry_process_name: &GeometryProcessName) -> TrackTrianglesYielder {
        TrackTrianglesYielder {
            geometry_process_name: geometry_process_name.clone(),
            track_triangles: None
        }
    }

    pub(super) fn program(&self) -> Result<&TrackTrianglesProgram,Message> {
        self.track_triangles.as_ref().ok_or_else(|| Message::CodeInvariantFailed(format!("using accessor without setting")))
    }
}
