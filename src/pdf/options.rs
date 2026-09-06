use super::Faces;
use super::Paper;
use std::path::Path;

#[derive(Debug)]
pub struct Options<'a> {
    pub output: &'a Path,
    pub cache: &'a Path,
    pub paper: Paper,
    pub faces: Faces,
    pub gap: f32,
    pub offline: bool,
    pub force: bool,
}
