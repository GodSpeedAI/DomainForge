//! Example demonstrating the SEA DSL Parser
//! 
//! This example shows how to use the parser to create a supply chain model
//! from DSL source code.

use sea_core::parse_to_graph;

fn main() {
    // Define a camera supply chain using SEA DSL
    let supply_chain_dsl = r#"
        // === Entities in the Supply Chain ===
        Entity "Component Supplier" in sourcing
        Entity "Camera Assembly Plant" in manufacturing
        Entity "Quality Control Lab" in qa
        Entity "Distribution Center" in logistics
        Entity "Retail Partner A" in sales
        Entity "Retail Partner B" in sales
        
        // === Resources ===
        Resource "Camera Lens" units in components
        Resource "Camera Body" units in components
        Resource "Camera Sensor" units in components
        Resource "Assembled Camera" units in products
        Resource "QA Approved Camera" units in products
        
        // === Flows ===
        // Component sourcing
        Flow "Camera Lens" from "Component Supplier" to "Camera Assembly Plant" quantity 1000
        Flow "Camera Body" from "Component Supplier" to "Camera Assembly Plant" quantity 1000
        Flow "Camera Sensor" from "Component Supplier" to "Camera Assembly Plant" quantity 1000
        
        // Assembly and QA
        Flow "Assembled Camera" from "Camera Assembly Plant" to "Quality Control Lab" quantity 950
        Flow "QA Approved Camera" from "Quality Control Lab" to "Distribution Center" quantity 900
        
        // Distribution to retail
        Flow "QA Approved Camera" from "Distribution Center" to "Retail Partner A" quantity 450
        Flow "QA Approved Camera" from "Distribution Center" to "Retail Partner B" quantity 450
        
        // === Business Policies ===
        Policy min_flow_quantity as forall f in flows: (f.quantity > 0)
        Policy qa_exists as exists e in entities: (e.name = "Quality Control Lab")
        Policy unique_distribution as exists_unique e in entities: (e.name = "Distribution Center")
    "#;
    
    // Parse the DSL into a Graph
    match parse_to_graph(supply_chain_dsl) {
        Ok(graph) => {
            println!("✅ Successfully parsed supply chain model!\n");
            
            // Display summary
            println!("📊 Model Summary:");
            println!("  • Entities:  {}", graph.all_entities().len());
            println!("  • Resources: {}", graph.all_resources().len());
            println!("  • Flows:     {}", graph.all_flows().len());
            println!();
            
            // Display entities by domain
            println!("🏢 Entities by Domain:");
            let mut domains = std::collections::HashMap::new();
            for entity in graph.all_entities() {
                let domain = entity.namespace().unwrap_or("(no domain)");
                domains.entry(domain).or_insert_with(Vec::new).push(entity.name());
            }
            for (domain, entities) in domains.iter() {
                println!("  {}: {}", domain, entities.join(", "));
            }
            println!();
            
            // Display resource types
            println!("📦 Resources:");
            for resource in graph.all_resources() {
                println!("  • {} [{}] in {}", 
                    resource.name(), 
                    resource.unit(),
                    resource.namespace().unwrap_or("(no domain)"));
            }
            println!();
            
            // Analyze a specific entity
            if let Some(plant) = graph.all_entities().iter()
                .find(|e| e.name() == "Camera Assembly Plant") 
            {
                println!("🏭 Camera Assembly Plant Analysis:");
                
                let inflows = graph.flows_to(plant.id());
                println!("  Incoming flows: {}", inflows.len());
                for flow in inflows {
                    let from = graph.get_entity(flow.from_id()).unwrap();
                    let resource = graph.get_resource(flow.resource_id()).unwrap();
                    println!("    ← {} of {} from {}", 
                        flow.quantity(), 
                        resource.name(),
                        from.name());
                }
                
                let outflows = graph.flows_from(plant.id());
                println!("  Outgoing flows: {}", outflows.len());
                for flow in outflows {
                    let to = graph.get_entity(flow.to_id()).unwrap();
                    let resource = graph.get_resource(flow.resource_id()).unwrap();
                    println!("    → {} of {} to {}", 
                        flow.quantity(),
                        resource.name(),
                        to.name());
                }
                
                println!("  Upstream entities: {}", 
                    graph.upstream_entities(plant.id()).len());
                println!("  Downstream entities: {}", 
                    graph.downstream_entities(plant.id()).len());
            }
            println!();
            
            // Display flow summary
            println!("🔄 Total Flow Quantities:");
            let mut resource_totals = std::collections::HashMap::new();
            for flow in graph.all_flows() {
                let resource = graph.get_resource(flow.resource_id()).unwrap();
                *resource_totals.entry(resource.name()).or_insert(0.0) += 
                    flow.quantity().to_string().parse::<f64>().unwrap_or(0.0);
            }
            for (resource, total) in resource_totals.iter() {
                println!("  • {}: {} units", resource, total);
            }
            
        }
        Err(e) => {
            eprintln!("❌ Parse error: {}", e);
            std::process::exit(1);
        }
    }
}
