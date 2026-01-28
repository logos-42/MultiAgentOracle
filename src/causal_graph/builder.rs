//! Causal Graph Builder

use crate::causal_graph::{
    types::{CausalGraph, CausalNode, CausalEdge, CausalPath, NodeType, EdgeType, PathType},
    selection::VariableSelector,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for causal graph builder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphBuilderConfig {
    /// Maximum number of core variables (default: 5)
    pub max_core_variables: usize,
    
    /// Minimum number of core variables (default: 3)
    pub min_core_variables: usize,
    
    /// Maximum number of main paths (default: 3)
    pub max_main_paths: usize,
    
    /// Minimum number of main paths (default: 2)
    pub min_main_paths: usize,
    
    /// Minimum edge weight threshold
    pub min_edge_weight: f64,
    
    /// Method for variable selection
    pub selection_method: crate::causal_graph::selection::SelectionMethod,
    
    /// Alpha for importance weighting (0.0-1.0)
    pub importance_alpha: f64,
}

impl Default for GraphBuilderConfig {
    fn default() -> Self {
        Self {
            max_core_variables: 5,
            min_core_variables: 3,
            max_main_paths: 3,
            min_main_paths: 2,
            min_edge_weight: 0.1,
            selection_method: crate::causal_graph::selection::SelectionMethod::MutualInformation,
            importance_alpha: 0.8,
        }
    }
}

/// Causal graph builder
pub struct CausalGraphBuilder {
    config: GraphBuilderConfig,
}

impl CausalGraphBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: GraphBuilderConfig::default(),
        }
    }
    
    /// Create a builder with custom configuration
    pub fn with_config(config: GraphBuilderConfig) -> Self {
        Self { config }
    }
    
    /// Build a causal graph from intervention and response data
    pub fn build(
        &self,
        intervention_data: &[f64],
        response_data: &[f64],
        context: Option<&HashMap<String, f64>>,
    ) -> Result<CausalGraph, String> {
        // Step 1: Select core variables
        let variable_selector = VariableSelector::new(self.config.selection_method);
        let selected_variables = variable_selector.select_variables(
            intervention_data,
            response_data,
            context,
            self.config.max_core_variables,
            self.config.min_core_variables,
        )?;
        
        // Step 2: Create nodes
        let mut graph = self.create_graph(selected_variables.scores.len());
        
        self.create_nodes(&mut graph, &selected_variables, intervention_data, response_data)?;
        
        // Step 3: Create edges
        self.create_edges(&mut graph, intervention_data, response_data, context)?;
        
        // Step 4: Identify main paths
        self.identify_main_paths(&mut graph)?;
        
        // Step 5: Validate graph
        if !graph.is_valid() {
            return Err("Generated causal graph is invalid".to_string());
        }
        
        Ok(graph)
    }
    
    /// Build a causal graph from historical response matrix
    pub fn build_from_history(
        &self,
        history: &[Vec<f64>],
        current_intervention: &[f64],
    ) -> Result<CausalGraph, String> {
        if history.is_empty() {
            return Err("History matrix is empty".to_string());
        }
        
        let num_agents = history.len();
        let num_dimensions = history[0].len();
        
        // Extract intervention and response patterns
        let mut intervention_data = Vec::new();
        let mut response_data = Vec::new();
        
        for i in 0..num_dimensions {
            intervention_data.push(current_intervention[i]);
            response_data.push(
                history.iter().map(|agent| agent[i]).sum::<f64>() / num_agents as f64
            );
        }
        
        self.build(&intervention_data, &response_data, None)
    }
    
    /// Create an empty graph with proper metadata
    fn create_graph(&self, num_variables: usize) -> CausalGraph {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        CausalGraph {
            id: format!("graph_{}", now),
            nodes: Vec::with_capacity(num_variables),
            edges: Vec::new(),
            main_paths: Vec::new(),
            metadata: crate::causal_graph::types::GraphMetadata {
                created_at: now,
                updated_at: now,
                num_core_variables: 0,
                num_main_paths: 0,
                version: "1.0.0".to_string(),
            },
        }
    }
    
    /// Create nodes from selected variables
    fn create_nodes(
        &self,
        graph: &mut CausalGraph,
        variables: &crate::causal_graph::selection::SelectionScore,
        intervention_data: &[f64],
        response_data: &[f64],
    ) -> Result<(), String> {
        // First node: treatment (most influential intervention dimension)
        let treatment_idx = variables
            .scores
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .map(|(i, _)| i)
            .ok_or("No variables found")?;
        
        graph.add_node(CausalNode {
            id: "X".to_string(),
            name: format!("Treatment_{}", treatment_idx),
            node_type: NodeType::Treatment,
            value: Some(intervention_data[treatment_idx]),
            intervention_target: true,
            importance: variables.scores[treatment_idx],
        })?;
        
        // Last node: outcome (most influential response dimension)
        let outcome_idx = if response_data.len() > treatment_idx {
            // Use a different dimension for outcome
            (treatment_idx + 1) % variables.scores.len()
        } else {
            0
        };
        
        graph.add_node(CausalNode {
            id: "Y".to_string(),
            name: format!("Outcome_{}", outcome_idx),
            node_type: NodeType::Outcome,
            value: Some(response_data[outcome_idx]),
            intervention_target: false,
            importance: variables.scores[outcome_idx],
        })?;
        
        // Middle nodes: confounders/mediators
        for i in 0..variables.scores.len() {
            if i != treatment_idx && i != outcome_idx {
                let node_type = if i % 2 == 0 {
                    NodeType::Confounder
                } else {
                    NodeType::Mediator
                };
                
                graph.add_node(CausalNode {
                    id: format!("N{}", i),
                    name: format!("Variable_{}", i),
                    node_type,
                    value: Some(response_data.get(i).copied().unwrap_or(0.0)),
                    intervention_target: false,
                    importance: variables.scores[i],
                })?;
            }
        }
        
        Ok(())
    }
    
    /// Create edges based on causal relationships
    fn create_edges(
        &self,
        graph: &mut CausalGraph,
        intervention_data: &[f64],
        response_data: &[f64],
        _context: Option<&HashMap<String, f64>>,
    ) -> Result<(), String> {
        // Calculate correlation-based edge weights
        let mut weights = Vec::new();
        
        for i in 0..intervention_data.len() {
            for j in 0..response_data.len() {
                if i != j {
                    let weight = self.calculate_edge_weight(intervention_data[i], response_data[j]);
                    if weight.abs() >= self.config.min_edge_weight {
                        weights.push((i, j, weight));
                    }
                }
            }
        }
        
        // Sort by absolute weight and create edges
        weights.sort_by(|a, b| b.2.abs().partial_cmp(&a.2.abs()).unwrap());
        
        let mut edge_id = 0;
        for (i, j, weight) in weights {
            if edge_id >= self.config.max_main_paths * 2 {
                break;
            }
            
            let source_node = graph.nodes.get(i).ok_or("Invalid node index")?;
            let target_node = graph.nodes.get(j).ok_or("Invalid node index")?;
            
            let edge_type = match (source_node.node_type, target_node.node_type) {
                (NodeType::Treatment, NodeType::Outcome) => EdgeType::Direct,
                (NodeType::Treatment, NodeType::Confounder) => EdgeType::Direct,
                (NodeType::Confounder, NodeType::Outcome) => EdgeType::Indirect,
                (NodeType::Mediator, NodeType::Outcome) => EdgeType::Indirect,
                _ => EdgeType::Indirect,
            };
            
            graph.add_edge(CausalEdge {
                id: format!("edge_{}", edge_id),
                source: source_node.id.clone(),
                target: target_node.id.clone(),
                weight,
                edge_type,
            })?;
            
            edge_id += 1;
        }
        
        Ok(())
    }
    
    /// Calculate edge weight based on intervention-response correlation
    fn calculate_edge_weight(&self, intervention: f64, response: f64) -> f64 {
        // Simple correlation-based weight
        let correlation = (intervention * response).signum();
        let magnitude = (response.abs() / (intervention.abs() + 1e-6)).min(1.0);
        correlation * magnitude
    }
    
    /// Identify main causal paths in the graph
    fn identify_main_paths(&self, graph: &mut CausalGraph) -> Result<(), String> {
        if graph.edges.is_empty() {
            return Err("No edges found in graph".to_string());
        }
        
        // Find treatment and outcome nodes
        let treatment_id = graph.nodes
            .iter()
            .find(|n| n.node_type == NodeType::Treatment)
            .map(|n| n.id.clone())
            .ok_or("No treatment node found")?;
        
        let outcome_id = graph.nodes
            .iter()
            .find(|n| n.node_type == NodeType::Outcome)
            .map(|n| n.id.clone())
            .ok_or("No outcome node found")?;
        
        // Find all paths from treatment to outcome
        let mut all_paths = self.find_all_paths(graph, &treatment_id, &outcome_id)?;
        
        // Sort paths by strength
        all_paths.sort_by(|a, b| b.strength.partial_cmp(&a.strength).unwrap());
        
        // Keep only top 2-3 paths
        let num_paths = all_paths.len()
            .clamp(self.config.min_main_paths, self.config.max_main_paths);
        
        graph.main_paths = all_paths.into_iter().take(num_paths).collect();
        graph.metadata.num_main_paths = graph.main_paths.len();
        
        Ok(())
    }
    
    /// Find all paths from source to target
    fn find_all_paths(
        &self,
        graph: &CausalGraph,
        source: &str,
        target: &str,
    ) -> Result<Vec<CausalPath>, String> {
        let mut paths = Vec::new();
        let mut visited = std::collections::HashSet::new();
        
        self.find_paths_recursive(
            graph,
            source,
            target,
            &mut vec![source.to_string()],
            &mut visited,
            1.0,
            &mut paths,
        );
        
        Ok(paths)
    }
    
    /// Recursively find paths
    fn find_paths_recursive(
        &self,
        graph: &CausalGraph,
        current: &str,
        target: &str,
        path: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
        path_strength: f64,
        paths: &mut Vec<CausalPath>,
    ) {
        if current == target {
            let path_type = self.classify_path(graph, path);
            paths.push(CausalPath {
                id: format!("path_{}", paths.len()),
                nodes: path.clone(),
                strength: path_strength,
                path_type,
            });
            return;
        }
        
        visited.insert(current.to_string());
        
        // Get all outgoing edges from current node
        let children: Vec<_> = graph.edges
            .iter()
            .filter(|e| e.source == current && !visited.contains(&e.target))
            .collect();
        
        for edge in children {
            path.push(edge.target.clone());
            self.find_paths_recursive(
                graph,
                &edge.target,
                target,
                path,
                visited,
                path_strength * edge.weight.abs(),
                paths,
            );
            path.pop();
        }
        
        visited.remove(current);
    }
    
    /// Classify path type
    fn classify_path(&self, graph: &CausalGraph, path: &[String]) -> PathType {
        // Check if path contains confounders (nodes with confounding edges)
        let has_confounder = path.iter().any(|node_id| {
            graph.get_node(node_id)
                .map(|node| node.node_type == NodeType::Confounder)
                .unwrap_or(false)
        });
        
        if has_confounder {
            PathType::BackDoor
        } else {
            PathType::FrontDoor
        }
    }
}

impl Default for CausalGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    
    #[test]
    fn test_builder_creation() {
        let builder = CausalGraphBuilder::new();
        assert_eq!(builder.config.max_core_variables, 5);
        assert_eq!(builder.config.min_core_variables, 3);
    }
    
    #[test]
    fn test_build_simple_graph() {
        let builder = CausalGraphBuilder::new();
        
        let intervention = vec![1.0, -1.0, 0.5];
        let response = vec![0.8, -0.7, 0.4];
        
        let graph = builder.build(&intervention, &response, None);
        
        assert!(graph.is_ok());
        let graph = graph.unwrap();
        assert!(graph.is_valid());
        assert!(!graph.nodes.is_empty());
    }
    
    #[test]
    fn test_build_from_history() {
        let builder = CausalGraphBuilder::new();
        
        let history = vec![
            vec![1.0, 0.8, 0.5],
            vec![0.9, 0.7, 0.6],
            vec![1.1, 0.9, 0.4],
        ];
        
        let current_intervention = vec![1.0, -1.0, 0.5];
        
        let graph = builder.build_from_history(&history, &current_intervention);
        
        assert!(graph.is_ok());
        let graph = graph.unwrap();
        assert!(graph.is_valid());
    }
}
