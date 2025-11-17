use crate::graph::Graph;
use crate::sbvr::SbvrModel;

pub fn import_sbvr_xmi(xmi: &str) -> Result<Graph, String> {
    match SbvrModel::from_xmi(xmi) {
        Ok(model) => match model.to_graph() {
            Ok(graph) => Ok(graph),
            Err(e) => Err(format!("Failed to convert SBVR to Graph: {}", e)),
        },
        Err(e) => Err(format!("Failed to parse SBVR XMI: {}", e)),
    }
}
