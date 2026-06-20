use crate::graph::Graph;
use crate::sbvr::SbvrModel;

pub fn import_sbvr_xmi(xmi: &str) -> Result<Graph, String> {
    let model = SbvrModel::from_xmi(xmi).map_err(|e| format!("Failed to parse SBVR XMI: {}", e))?;
    model
        .to_graph()
        .map_err(|e| format!("Failed to convert SBVR to Graph: {}", e))
}
