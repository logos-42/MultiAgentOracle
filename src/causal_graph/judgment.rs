//! Causal Graph Judgment Module
//!
//! Provides logic for validating causal reasoning based on graph structure

use crate::causal_graph::types::{CausalGraph, CausalEffect, PathType};
use serde::{Deserialize, Serialize};

/// Result of causal judgment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalJudgment {
    /// Whether the causal reasoning is valid
    pub is_valid: bool,
    
    /// Confidence score (0.0-1.0)
    pub confidence: f64,
    
    /// Detailed judgment criteria
    pub criteria: JudgmentCriteria,
    
    /// Recommended action
    pub recommendation: Recommendation,
    
    /// Judgment explanation
    pub explanation: String,
}

/// Judgment criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgmentCriteria {
    /// Graph structure validity
    pub graph_structure_valid: bool,
    
    /// Has sufficient causal paths
    pub has_sufficient_paths: bool,
    
    /// Causal effect is significant
    pub causal_effect_significant: bool,
    
    /// No confounding bias
    pub no_confounding_bias: bool,
    
    /// Path strength is adequate
    pub path_strength_adequate: bool,
    
    /// Graph complexity is appropriate
    pub complexity_appropriate: bool,
}

/// Recommendation based on judgment
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Recommendation {
    /// Accept the causal reasoning
    Accept,
    
    /// Accept with warnings
    AcceptWithWarnings,
    
    /// Request more evidence
    RequestMoreEvidence,
    
    /// Reject - causal reasoning flawed
    Reject,
}

impl CausalGraph {
    /// Validate causal reasoning and produce judgment
    pub fn validate_causal_reasoning(&self, causal_effect: &CausalEffect) -> CausalJudgment {
        let mut criteria = JudgmentCriteria {
            graph_structure_valid: self.is_valid(),
            has_sufficient_paths: !self.main_paths.is_empty(),
            causal_effect_significant: causal_effect.ate.abs() > 0.1,
            no_confounding_bias: self.has_unconfounded_paths(),
            path_strength_adequate: self.has_strong_paths(),
            complexity_appropriate: self.complexity_appropriate(),
        };

        // Calculate confidence based on criteria
        let mut confidence = 0.0;
        let mut passed = 0;
        let total = 6;

        if criteria.graph_structure_valid { passed += 1; }
        if criteria.has_sufficient_paths { passed += 1; }
        if criteria.causal_effect_significant { passed += 1; }
        if criteria.no_confounding_bias { passed += 1; }
        if criteria.path_strength_adequate { passed += 1; }
        if criteria.complexity_appropriate { passed += 1; }

        confidence = passed as f64 / total as f64;

        // Determine recommendation
        let recommendation = match passed {
            6 => Recommendation::Accept,
            4 | 5 => Recommendation::AcceptWithWarnings,
            2 | 3 => Recommendation::RequestMoreEvidence,
            _ => Recommendation::Reject,
        };

        // Generate explanation
        let explanation = self.generate_judgment_explanation(&criteria, causal_effect, confidence);

        CausalJudgment {
            is_valid: recommendation != Recommendation::Reject,
            confidence,
            criteria,
            recommendation,
            explanation,
        }
    }

    /// Check if graph has unconfounded paths (front-door)
    fn has_unconfounded_paths(&self) -> bool {
        self.main_paths
            .iter()
            .any(|p| p.path_type == PathType::FrontDoor)
    }

    /// Check if graph has strong paths
    fn has_strong_paths(&self) -> bool {
        if self.main_paths.is_empty() {
            return false;
        }
        
        let avg_strength: f64 = self.main_paths.iter().map(|p| p.strength).sum::<f64>() 
            / self.main_paths.len() as f64;
        
        avg_strength > 0.3
    }

    /// Check if graph complexity is appropriate (3-5 nodes)
    fn complexity_appropriate(&self) -> bool {
        let node_count = self.nodes.len();
        node_count >= 3 && node_count <= 5
    }

    /// Generate explanation for judgment
    fn generate_judgment_explanation(
        &self,
        criteria: &JudgmentCriteria,
        causal_effect: &CausalEffect,
        confidence: f64,
    ) -> String {
        let mut explanation = String::new();
        
        explanation.push_str("Causal Reasoning Validation:\n");
        
        // Graph structure
        explanation.push_str(&format!(
            "  • Graph Structure: {} ({})\n",
            if criteria.graph_structure_valid { "✓ Valid" } else { "✗ Invalid" },
            format!("{} nodes, {} edges", self.nodes.len(), self.edges.len())
        ));
        
        // Paths
        explanation.push_str(&format!(
            "  • Causal Paths: {} ({})\n",
            if criteria.has_sufficient_paths { "✓ Sufficient" } else { "✗ Insufficient" },
            format!("{} main paths", self.main_paths.len())
        ));
        
        // Path types
        let frontdoor_count = self.main_paths.iter().filter(|p| p.path_type == PathType::FrontDoor).count();
        let backdoor_count = self.main_paths.iter().filter(|p| p.path_type == PathType::BackDoor).count();
        explanation.push_str(&format!(
            "    - Front-door: {}, Back-door: {}\n",
            frontdoor_count, backdoor_count
        ));
        
        // Causal effect
        explanation.push_str(&format!(
            "  • Causal Effect: {} (ATE = {:.4}, method: {:?})\n",
            if criteria.causal_effect_significant { "✓ Significant" } else { "⚠ Weak" },
            causal_effect.ate,
            causal_effect.method
        ));
        
        // Confounding
        explanation.push_str(&format!(
            "  • Confounding Bias: {} ({})\n",
            if criteria.no_confounding_bias { "✓ Controlled" } else { "⚠ Present" },
            if criteria.no_confounding_bias { "Has front-door paths" } else { "All paths are back-door" }
        ));
        
        // Path strength
        let avg_strength: f64 = if !self.main_paths.is_empty() {
            self.main_paths.iter().map(|p| p.strength).sum::<f64>() / self.main_paths.len() as f64
        } else {
            0.0
        };
        explanation.push_str(&format!(
            "  • Path Strength: {} (avg = {:.4})\n",
            if criteria.path_strength_adequate { "✓ Strong" } else { "⚠ Weak" },
            avg_strength
        ));
        
        // Complexity
        explanation.push_str(&format!(
            "  • Complexity: {} ({})\n",
            if criteria.complexity_appropriate { "✓ Appropriate" } else { "⚠ Issue" },
            if self.nodes.len() < 3 { "Too few nodes" } 
            else if self.nodes.len() > 5 { "Too many nodes" }
            else { "Optimal" }
        ));
        
        // Overall confidence
        explanation.push_str(&format!(
            "\nOverall Confidence: {:.1}%\n",
            confidence * 100.0
        ));
        
        explanation
    }
}

/// Compare two causal graphs and determine similarity
pub fn compare_causal_graphs(graph1: &CausalGraph, graph2: &CausalGraph) -> GraphComparison {
    let node_similarity = compute_node_similarity(graph1, graph2);
    let edge_similarity = compute_edge_similarity(graph1, graph2);
    let path_similarity = compute_path_similarity(graph1, graph2);
    
    let overall_similarity = node_similarity * 0.4 + edge_similarity * 0.4 + path_similarity * 0.2;
    
    GraphComparison {
        node_similarity,
        edge_similarity,
        path_similarity,
        overall_similarity,
        is_similar: overall_similarity > 0.7,
        explanation: generate_comparison_explanation(
            node_similarity, edge_similarity, path_similarity, overall_similarity
        ),
    }
}

/// Result of comparing two graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphComparison {
    /// Node similarity (0.0-1.0)
    pub node_similarity: f64,
    
    /// Edge similarity (0.0-1.0)
    pub edge_similarity: f64,
    
    /// Path similarity (0.0-1.0)
    pub path_similarity: f64,
    
    /// Overall similarity (0.0-1.0)
    pub overall_similarity: f64,
    
    /// Whether graphs are considered similar
    pub is_similar: bool,
    
    /// Comparison explanation
    pub explanation: String,
}

/// Compute node similarity
fn compute_node_similarity(graph1: &CausalGraph, graph2: &CausalGraph) -> f64 {
    if graph1.nodes.is_empty() && graph2.nodes.is_empty() {
        return 1.0;
    }
    
    let mut common_nodes = 0;
    for node1 in &graph1.nodes {
        if graph2.nodes.iter().any(|n| n.node_type == node1.node_type) {
            common_nodes += 1;
        }
    }
    
    let max_nodes = graph1.nodes.len().max(graph2.nodes.len());
    if max_nodes == 0 {
        return 1.0;
    }
    
    common_nodes as f64 / max_nodes as f64
}

/// Compute edge similarity
fn compute_edge_similarity(graph1: &CausalGraph, graph2: &CausalGraph) -> f64 {
    if graph1.edges.is_empty() && graph2.edges.is_empty() {
        return 1.0;
    }
    
    let mut common_edges = 0;
    for edge1 in &graph1.edges {
        if graph2.edges.iter().any(|e| e.edge_type == edge1.edge_type) {
            common_edges += 1;
        }
    }
    
    let max_edges = graph1.edges.len().max(graph2.edges.len());
    if max_edges == 0 {
        return 1.0;
    }
    
    common_edges as f64 / max_edges as f64
}

/// Compute path similarity
fn compute_path_similarity(graph1: &CausalGraph, graph2: &CausalGraph) -> f64 {
    if graph1.main_paths.is_empty() && graph2.main_paths.is_empty() {
        return 1.0;
    }
    
    let mut common_paths = 0;
    for path1 in &graph1.main_paths {
        if graph2.main_paths.iter().any(|p| p.path_type == path1.path_type) {
            common_paths += 1;
        }
    }
    
    let max_paths = graph1.main_paths.len().max(graph2.main_paths.len());
    if max_paths == 0 {
        return 1.0;
    }
    
    common_paths as f64 / max_paths as f64
}

/// Generate comparison explanation
fn generate_comparison_explanation(
    node_sim: f64,
    edge_sim: f64,
    path_sim: f64,
    overall_sim: f64,
) -> String {
    format!(
        "Graph Comparison:\n\
         • Node Similarity: {:.1}%\n\
         • Edge Similarity: {:.1}%\n\
         • Path Similarity: {:.1}%\n\
         • Overall Similarity: {:.1}%\n\n\
         {}",
        node_sim * 100.0,
        edge_sim * 100.0,
        path_sim * 100.0,
        overall_sim * 100.0,
        if overall_sim > 0.7 {
            "✓ Graphs are structurally similar"
        } else if overall_sim > 0.4 {
            "⚠ Graphs have some similarities but significant differences"
        } else {
            "✗ Graphs are structurally different"
        }
    )
}

/// Detect if graphs are too similar (potential collusion)
pub fn detect_collusion(graphs: &[&CausalGraph], threshold: f64) -> CollusionDetection {
    if graphs.len() < 2 {
        return CollusionDetection {
            collusion_detected: false,
            avg_similarity: 0.0,
            max_similarity: 0.0,
            explanation: "Insufficient graphs for comparison".to_string(),
        };
    }

    let mut similarities = Vec::new();
    for i in 0..graphs.len() {
        for j in (i + 1)..graphs.len() {
            let comparison = compare_causal_graphs(graphs[i], graphs[j]);
            similarities.push(comparison.overall_similarity);
        }
    }

    let avg_similarity = similarities.iter().sum::<f64>() / similarities.len() as f64;
    let max_similarity = *similarities.iter().max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap_or(&0.0);

    let collusion_detected = avg_similarity > threshold;

    CollusionDetection {
        collusion_detected,
        avg_similarity,
        max_similarity,
        explanation: format!(
            "Collusion Detection:\n\
             • Compared {} graph pairs\n\
             • Average Similarity: {:.1}%\n\
             • Maximum Similarity: {:.1}%\n\n\
             {}",
            similarities.len(),
            avg_similarity * 100.0,
            max_similarity * 100.0,
            if collusion_detected {
                format!("⚠ POTENTIAL COLLUSION: Graphs are too similar (threshold: {:.1}%)", 
                        threshold * 100.0)
            } else {
                "✓ No evidence of collusion - graphs show healthy diversity".to_string()
            }
        ),
    }
}

/// Result of collusion detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollusionDetection {
    /// Whether collusion is detected
    pub collusion_detected: bool,
    
    /// Average similarity across all pairs
    pub avg_similarity: f64,
    
    /// Maximum similarity found
    pub max_similarity: f64,
    
    /// Detection explanation
    pub explanation: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causal_graph::types::{CausalNode, NodeType};
    
    #[test]
    fn test_causal_judgment() {
        let mut graph = CausalGraph::new("test".to_string());
        
        graph.add_node(CausalNode {
            id: "X".to_string(),
            name: "Treatment".to_string(),
            node_type: NodeType::Treatment,
            value: Some(1.0),
            intervention_target: true,
            importance: 0.9,
        }).ok();
        
        let causal_effect = CausalEffect {
            ate: 0.5,
            cate: None,
            confidence_interval: None,
            method: crate::causal_graph::types::EffectMethod::Direct,
        };
        
        let judgment = graph.validate_causal_reasoning(&causal_effect);
        println!("Judgment: {:?}", judgment);
    }
}
