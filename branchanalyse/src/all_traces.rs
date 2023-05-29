use serde::Serialize;

#[derive(Serialize)]
pub struct AllTracesResult {
    pub table_size: usize,
    pub trace: String,
    pub accuracy: f64
}