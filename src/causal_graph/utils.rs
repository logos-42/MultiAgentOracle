//! Causal Graph Utility Functions

use crate::causal_graph::types::{
    CausalGraph, CausalEffect,
    Intervention, NodeType, EffectMethod
};
use std::collections::HashMap;

/// Compute causal effect using do-calculus
pub fn compute_causal_effect(
    graph: &CausalGraph,
    intervention: &Intervention,
    outcome_node: &str,
) -> Result<CausalEffect, String> {
    // Check if outcome node exists
    graph.get_node(outcome_node)
        .ok_or(format!("Outcome node {} not found", outcome_node))?;
    
    // Check if intervention is valid
    if !graph.get_node(&intervention.target_node).is_some() {
        return Err(format!("Intervention target {} not found", intervention.target_node));
    }
    
    // Apply do-operator to get modified graph
    let modified_graph = apply_do_operator(graph, intervention)?;
    
    // Determine computation method
    let method = determine_effect_method(graph, &intervention.target_node, outcome_node)?;
    
    // Compute effect based on method
    let ate = match method {
        EffectMethod::Backdoor => {
            compute_backdoor_adjustment(graph, &modified_graph, intervention, outcome_node)?
        }
        EffectMethod::FrontDoor => {
            compute_frontdoor_adjustment(graph, &modified_graph, intervention, outcome_node)?
        }
        EffectMethod::Direct => {
            compute_direct_effect(graph, &modified_graph, intervention, outcome_node)?
        }
        _ => {
            // Fallback to direct computation
            compute_direct_effect(graph, &modified_graph, intervention, outcome_node)?
        }
    };
    
    Ok(CausalEffect {
        ate,
        cate: None,
        confidence_interval: None,
        method,
    })
}

/// Apply do-operator to a causal graph
pub fn apply_do_operator(
    graph: &CausalGraph,
    intervention: &Intervention,
) -> Result<CausalGraph, String> {
    let mut modified_graph = graph.clone();
    
    // Find and update the intervention target node
    let target_node = modified_graph.nodes
        .iter_mut()
        .find(|n| n.id == intervention.target_node)
        .ok_or(format!("Target node {} not found", intervention.target_node))?;
    
    // Apply intervention: set node value and remove incoming edges
    target_node.value = Some(intervention.value);
    
    // Remove all incoming edges to the target node (do-operator cuts them)
    modified_graph.edges.retain(|e| e.target != intervention.target_node);
    
    // Update metadata
    modified_graph.metadata.updated_at = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    // Recompute post-intervention distribution
    let _post_intervention_dist = compute_post_intervention_distribution(&modified_graph);
    
    Ok(modified_graph)
}

/// Compute path influence (total effect along a path)
pub fn calculate_path_influence(
    graph: &CausalGraph,
    path: &[String],
) -> Result<f64, String> {
    if path.len() < 2 {
        return Ok(0.0);
    }
    
    let mut total_influence = 1.0;
    
    for i in 0..path.len() - 1 {
        let edge = graph.edges
            .iter()
            .find(|e| e.source == path[i] && e.target == path[i + 1])
            .ok_or(format!("No edge found from {} to {}", path[i], path[i + 1]))?;
        
        total_influence *= edge.weight.abs();
    }
    
    Ok(total_influence)
}

/// Compute backdoor adjustment formula
fn compute_backdoor_adjustment(
    original_graph: &CausalGraph,
    _modified_graph: &CausalGraph,
    intervention: &Intervention,
    outcome_node: &str,
) -> Result<f64, String> {
    // Find confounders (nodes that affect both treatment and outcome)
    let confounders = original_graph.nodes
        .iter()
        .filter(|n| n.node_type == NodeType::Confounder)
        .collect::<Vec<_>>();
    
    if confounders.is_empty() {
        // No confounders, use direct effect
        return compute_direct_effect(original_graph, _modified_graph, intervention, outcome_node);
    }
    
    // Backdoor adjustment: Σ_z P(Y | X=x, Z=z) P(Z=z)
    // Simplified: direct effect weighted by confounder importance
    let direct_effect = compute_direct_effect(original_graph, _modified_graph, intervention, outcome_node)?;
    
    let confounder_weight: f64 = confounders.iter().map(|c| c.importance).sum::<f64>() / confounders.len() as f64;
    
    Ok(direct_effect * (1.0 + confounder_weight * 0.5))
}

/// Compute front-door adjustment formula
fn compute_frontdoor_adjustment(
    original_graph: &CausalGraph,
    _modified_graph: &CausalGraph,
    intervention: &Intervention,
    outcome_node: &str,
) -> Result<f64, String> {
    // Find mediators (nodes on causal path from treatment to outcome)
    let mediators = original_graph.nodes
        .iter()
        .filter(|n| n.node_type == NodeType::Mediator)
        .collect::<Vec<_>>();
    
    if mediators.is_empty() {
        return compute_direct_effect(original_graph, _modified_graph, intervention, outcome_node);
    }
    
    // Front-door: Σ_m P(M | X=x) Σ_x' P(Y | M=m, X=x') P(X=x')
    // Simplified: effect through mediators
    let direct_effect = compute_direct_effect(original_graph, _modified_graph, intervention, outcome_node)?;
    
    let mediator_influence: f64 = mediators.iter()
        .filter_map(|m| calculate_path_influence(original_graph, &[intervention.target_node.clone(), m.id.clone(), outcome_node.to_string()]).ok())
        .sum::<f64>();
    
    Ok(direct_effect + mediator_influence)
}

/// Compute direct causal effect
fn compute_direct_effect(
    original_graph: &CausalGraph,
    _modified_graph: &CausalGraph,
    intervention: &Intervention,
    outcome_node: &str,
) -> Result<f64, String> {
    // Find edge from treatment to outcome (if exists)
    let direct_edge = original_graph.edges
        .iter()
        .find(|e| e.source == intervention.target_node && e.target == outcome_node);
    
    if let Some(edge) = direct_edge {
        // Direct effect: edge weight * intervention value
        Ok(edge.weight * intervention.value)
    } else {
        // No direct edge, compute via main paths
        let path_effect: f64 = original_graph.main_paths
            .iter()
            .filter(|p| p.nodes.first() == Some(&intervention.target_node) && p.nodes.last() == Some(&outcome_node.to_string()))
            .map(|p| p.strength * intervention.value)
            .sum();
        
        Ok(path_effect)
    }
}

/// Determine effect computation method
fn determine_effect_method(
    graph: &CausalGraph,
    treatment_node: &str,
    outcome_node: &str,
) -> Result<EffectMethod, String> {
    // Check for confounders
    let has_confounders = graph.nodes
        .iter()
        .any(|n| n.node_type == NodeType::Confounder);
    
    // Check for mediators
    let has_mediators = graph.nodes
        .iter()
        .any(|n| n.node_type == NodeType::Mediator);
    
    // Check for direct edge
    let has_direct_edge = graph.edges
        .iter()
        .any(|e| e.source == treatment_node && e.target == outcome_node);
    
    if has_direct_edge && !has_confounders {
        Ok(EffectMethod::Direct)
    } else if has_confounders {
        Ok(EffectMethod::Backdoor)
    } else if has_mediators {
        Ok(EffectMethod::FrontDoor)
    } else {
        Ok(EffectMethod::Direct)
    }
}

/// Compute post-intervention distribution
fn compute_post_intervention_distribution(graph: &CausalGraph) -> HashMap<String, f64> {
    let mut distribution = HashMap::new();
    
    // For each node, compute its value based on causal structure
    for node in &graph.nodes {
        if let Some(value) = node.value {
            distribution.insert(node.id.clone(), value);
        } else {
            // Node without explicit value: compute from parents
            let parents: Vec<_> = graph.get_parents(&node.id);
            if parents.is_empty() {
                // No parents, use importance as default
                distribution.insert(node.id.clone(), node.importance);
            } else {
                // Compute from parents (simplified)
                let parent_sum: f64 = parents.iter()
                    .filter_map(|pid| graph.get_node(pid))
                    .filter_map(|parent| distribution.get(&parent.id))
                    .map(|&v| v)
                    .sum();
                let avg_parent_value = parent_sum / parents.len() as f64;
                distribution.insert(node.id.clone(), avg_parent_value * node.importance);
            }
        }
    }
    
    distribution
}

/// Compare two causal graphs for similarity
pub fn compare_graphs(
    graph1: &CausalGraph,
    graph2: &CausalGraph,
) -> f64 {
    // Compare structure
    let node_similarity = compare_nodes(graph1, graph2);
    let edge_similarity = compare_edges(graph1, graph2);
    let path_similarity = compare_paths(graph1, graph2);
    
    // Weighted average
    0.4 * node_similarity + 0.4 * edge_similarity + 0.2 * path_similarity
}

/// Compare node sets
fn compare_nodes(graph1: &CausalGraph, graph2: &CausalGraph) -> f64 {
    if graph1.nodes.is_empty() || graph2.nodes.is_empty() {
        return 0.0;
    }
    
    let mut _similar_count = 0;
    let mut total_importance = 0.0;
    
    for node1 in &graph1.nodes {
        if let Some(node2) = graph2.get_node(&node1.id) {
            // Compare node type and importance
            if node1.node_type == node2.node_type {
                _similar_count += 1;
                total_importance += (1.0 - (node1.importance - node2.importance).abs()).max(0.0);
            }
        }
    }
    
    if graph1.nodes.is_empty() {
        return 0.0;
    }
    
    total_importance / graph1.nodes.len() as f64
}

/// Compare edge sets
fn compare_edges(graph1: &CausalGraph, graph2: &CausalGraph) -> f64 {
    if graph1.edges.is_empty() && graph2.edges.is_empty() {
        return 1.0;
    }
    
    if graph1.edges.is_empty() || graph2.edges.is_empty() {
        return 0.0;
    }
    
    let mut matched_edges = 0;
    
    for edge1 in &graph1.edges {
        let matched = graph2.edges.iter().any(|edge2| {
            edge1.source == edge2.source
                && edge1.target == edge2.target
                && (edge1.weight - edge2.weight).abs() < 0.1
        });
        
        if matched {
            matched_edges += 1;
        }
    }
    
    matched_edges as f64 / graph1.edges.len() as f64
}

/// Compare main paths
fn compare_paths(graph1: &CausalGraph, graph2: &CausalGraph) -> f64 {
    if graph1.main_paths.is_empty() && graph2.main_paths.is_empty() {
        return 1.0;
    }
    
    if graph1.main_paths.is_empty() || graph2.main_paths.is_empty() {
        return 0.0;
    }
    
    let mut matched_paths = 0;
    
    for path1 in &graph1.main_paths {
        let matched = graph2.main_paths.iter().any(|path2| {
            path1.nodes == path2.nodes
                && (path1.strength - path2.strength).abs() < 0.1
        });
        
        if matched {
            matched_paths += 1;
        }
    }
    
    matched_paths as f64 / graph1.main_paths.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causal_graph::builder::{CausalGraphBuilder, GraphBuilderConfig};
    
    #[test]
    fn test_apply_do_operator() {
        let builder = CausalGraphBuilder::new();
        
        let intervention = vec![1.0, 0.5, 2.0];
        let response = vec![0.8, 0.4, 1.6];
        
        let graph = builder.build(&intervention, &response, None).unwrap();
        
        let intervention_op = Intervention {
            target_node: "X".to_string(),
            value: 2.0,
            intervention_type: crate::causal_graph::types::InterventionType::Hard,
        };
        
        let modified = apply_do_operator(&graph, &intervention_op);
        
        assert!(modified.is_ok());
        let modified = modified.unwrap();
        
        // Check that incoming edges to X are removed
        let incoming_edges = modified.edges.iter().filter(|e| e.target == "X").count();
        assert_eq!(incoming_edges, 0);
    }
    
    #[test]
    fn test_compute_causal_effect() {
        let builder = CausalGraphBuilder::new();
        
        let intervention = vec![1.0, 0.5, 2.0];
        let response = vec![0.8, 0.4, 1.6];
        
        let graph = builder.build(&intervention, &response, None).unwrap();
        
        let intervention_op = Intervention {
            target_node: "X".to_string(),
            value: 1.5,
            intervention_type: crate::causal_graph::types::InterventionType::Hard,
        };
        
        let effect = compute_causal_effect(&graph, &intervention_op, "Y");
        
        assert!(effect.is_ok());
        let effect = effect.unwrap();
        assert!(effect.ate.abs() > 0.0);
    }
    
    #[test]
    fn test_calculate_path_influence() {
        let builder = CausalGraphBuilder::new();
        
        let intervention = vec![1.0, 0.5, 2.0];
        let response = vec![0.8, 0.4, 1.6];
        
        let graph = builder.build(&intervention, &response, None).unwrap();
        
        if !graph.main_paths.is_empty() {
            let influence = calculate_path_influence(&graph, &graph.main_paths[0].nodes);
            assert!(influence.is_ok());
            assert!(influence.unwrap() >= 0.0);
        }
    }
}
