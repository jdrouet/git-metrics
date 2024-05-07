use indexmap::IndexMap;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Metric {
    pub name: String,
    pub context: IndexMap<String, String>,
    pub value: f64,
}
