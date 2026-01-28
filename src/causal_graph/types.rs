//! Causal Graph Types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Lightweight causal graph with 3-5 core variables and 2-3 main paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalGraph {
    /// Graph identifier
    pub id: String,
    
    /// Causal nodes (variables)
    pub nodes: Vec<CausalNode>,
    
    /// Causal edges (relationships)
    pub edges: Vec<CausalEdge>,
    
    /// Main causal paths (top 2-3 most influential paths)
    pub main_paths: Vec<CausalPath>,
    
    /// Graph metadata
    pub metadata: GraphMetadata,
}

/// Causal node representing a variable in the graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalNode {
    /// Node ID
    pub id: String,
    
    /// Node name
    pub name: String,
    
    /// Node type (treatment, outcome, confounder, mediator)
    pub node_type: NodeType,
    
    /// Node value (if assigned)
    pub value: Option<f64>,
    
    /// Intervention target (if this node can be intervened upon)
    pub intervention_target: bool,
    
    /// Node importance score (0.0-1.0)
    pub importance: f64,
}

/// Node type in causal graph
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// Treatment variable (X in do(X))
    Treatment,
    /// Outcome variable (Y)
    Outcome,
    /// Confounder variable (Z that affects both X and Y)
    Confounder,
    /// Mediator variable (M on path from X to Y)
    Mediator,
    /// Control variable
    Control,
}

/// Causal edge representing a directed relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEdge {
    /// Edge ID
    pub id: String,
    
    /// Source node ID
    pub source: String,
    
    /// Target node ID
    pub target: String,
    
    /// Edge weight (causal strength)
    pub weight: f64,
    
    /// Edge type
    pub edge_type: EdgeType,
}

/// Edge type in causal graph
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum EdgeType {
    /// Direct causal effect
    Direct,
    /// Indirect causal effect (through mediator)
    Indirect,
    /// Confounding relationship
    Confounding,
}

/// Causal path representing a sequence of edges from treatment to outcome
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalPath {
    /// Path ID
    pub id: String,
    
    /// Sequence of node IDs along the path
    pub nodes: Vec<String>,
    
    /// Path strength (product of edge weights)
    pub strength: f64,
    
    /// Path type
    pub path_type: PathType,
}

/// Path type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum PathType {
    /// Front-door path (through mediators, no backdoor)
    FrontDoor,
    /// Backdoor path (has confounding)
    BackDoor,
    /// Confounded path (contains cycles or colliders)
    Confounded,
}

/// Graph metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    /// Creation timestamp
    pub created_at: u64,
    
    /// Last updated timestamp
    pub updated_at: u64,
    
    /// Number of core variables
    pub num_core_variables: usize,
    
    /// Number of main paths
    pub num_main_paths: usize,
    
    /// Graph version
    pub version: String,
}

/// Causal effect computation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CausalEffect {
    /// Average treatment effect (ATE)
    pub ate: f64,
    
    /// Conditional average treatment effect (CATE)
    pub cate: Option<f64>,
    
    /// Confidence interval
    pub confidence_interval: Option<(f64, f64)>,
    
    /// Computation method used
    pub method: EffectMethod,
}

/// Method used to compute causal effect
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EffectMethod {
    /// Backdoor adjustment formula
    Backdoor,
    /// Front-door criterion
    FrontDoor,
    /// Instrumental variable
    InstrumentalVariable,
    /// Direct computation from graph structure
    Direct,
}

/// Intervention operation (do-operator)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Intervention {
    /// Target node ID
    pub target_node: String,
    
    /// Intervention value
    pub value: f64,
    
    /// Intervention type
    pub intervention_type: InterventionType,
}

/// Intervention type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InterventionType {
    /// Hard intervention (set value directly)
    Hard,
    /// Soft intervention (modify distribution)
    Soft,
    /// Perfect intervention (no noise)
    Perfect,
}

/// Result of applying do-operator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoOperatorResult {
    /// Intervention performed
    pub intervention: Intervention,
    
    /// Modified causal graph
    pub modified_graph: CausalGraph,
    
    /// Causal effect on outcome
    pub causal_effect: Option<CausalEffect>,
    
    /// Probability distribution after intervention
    pub post_intervention_dist: HashMap<String, f64>,
}

impl CausalGraph {
    /// Create a new empty causal graph
    pub fn new(id: String) -> Self {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        Self {
            id,
            nodes: Vec::new(),
            edges: Vec::new(),
            main_paths: Vec::new(),
            metadata: GraphMetadata {
                created_at: now,
                updated_at: now,
                num_core_variables: 0,
                num_main_paths: 0,
                version: "1.0.0".to_string(),
            },
        }
    }
    
    /// Add a node to the graph
    pub fn add_node(&mut self, node: CausalNode) -> Result<(), String> {
        if self.nodes.iter().any(|n| n.id == node.id) {
            return Err(format!("Node with ID {} already exists", node.id));
        }
        self.nodes.push(node);
        self.metadata.num_core_variables = self.nodes.len();
        self.metadata.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(())
    }
    
    /// Add an edge to the graph
    pub fn add_edge(&mut self, edge: CausalEdge) -> Result<(), String> {
        // Validate source and target nodes exist
        if !self.nodes.iter().any(|n| n.id == edge.source) {
            return Err(format!("Source node {} does not exist", edge.source));
        }
        if !self.nodes.iter().any(|n| n.id == edge.target) {
            return Err(format!("Target node {} does not exist", edge.target));
        }
        
        self.edges.push(edge);
        self.metadata.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Ok(())
    }
    
    /// Find children of a node (outgoing edges)
    pub fn get_children(&self, node_id: &str) -> Vec<String> {
        self.edges
            .iter()
            .filter(|e| e.source == node_id)
            .map(|e| e.target.clone())
            .collect()
    }
    
    /// Find parents of a node (incoming edges)
    pub fn get_parents(&self, node_id: &str) -> Vec<String> {
        self.edges
            .iter()
            .filter(|e| e.target == node_id)
            .map(|e| e.source.clone())
            .collect()
    }
    
    /// Get node by ID
    pub fn get_node(&self, node_id: &str) -> Option<&CausalNode> {
        self.nodes.iter().find(|n| n.id == node_id)
    }
    
    /// Compute graph hash for verification
    pub fn compute_hash(&self) -> [u8; 32] {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;
        
        let mut hasher = DefaultHasher::new();
        self.id.hash(&mut hasher);
        
        // Hash nodes
        for node in &self.nodes {
            node.id.hash(&mut hasher);
            node.node_type.hash(&mut hasher);
            node.importance.to_bits().hash(&mut hasher);
        }
        
        // Hash edges
        for edge in &self.edges {
            edge.id.hash(&mut hasher);
            edge.source.hash(&mut hasher);
            edge.target.hash(&mut hasher);
            edge.weight.to_bits().hash(&mut hasher);
            edge.edge_type.hash(&mut hasher);
        }
        
        let hash = hasher.finish();
        let mut bytes = [0u8; 32];
        let hash_bytes = hash.to_be_bytes();
        bytes[..hash_bytes.len()].copy_from_slice(&hash_bytes);
        bytes
    }
    
    /// Check if graph is valid
    pub fn is_valid(&self) -> bool {
        // Check core variables constraint (3-5 nodes)
        if self.nodes.len() < 3 || self.nodes.len() > 5 {
            return false;
        }
        
        // Check main paths constraint (2-3 paths)
        if self.main_paths.len() < 2 || self.main_paths.len() > 3 {
            return false;
        }
        
        // Check for at least one treatment and one outcome
        let has_treatment = self.nodes.iter().any(|n| n.node_type == NodeType::Treatment);
        let has_outcome = self.nodes.iter().any(|n| n.node_type == NodeType::Outcome);
        
        has_treatment && has_outcome
    }
    
    /// Generate causal graph from AI prompt
    pub async fn from_ai_prompt(
        prompt: &str,
        context: &str,
        llm_client: &crate::oracle_agent::LlmClient,
        config: Option<crate::causal_graph::ai_reasoning::AIReasoningConfig>,
    ) -> Result<Self, anyhow::Error> {
        use crate::causal_graph::ai_reasoning::AIReasoningEngine;
        
        let engine_config = config.unwrap_or_default();
        let engine = AIReasoningEngine::from_client(llm_client.clone(), engine_config);
        
        engine.generate_causal_graph(prompt, context).await
    }
}

impl Default for CausalGraph {
    fn default() -> Self {
        Self::new("default_graph".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_graph() {
        let graph = CausalGraph::new("test_graph".to_string());
        assert_eq!(graph.id, "test_graph");
        assert!(graph.nodes.is_empty());
        assert!(graph.edges.is_empty());
    }
    
    #[test]
    fn test_add_node() {
        let mut graph = CausalGraph::new("test_graph".to_string());
        
        let node = CausalNode {
            id: "X".to_string(),
            name: "Treatment".to_string(),
            node_type: NodeType::Treatment,
            value: Some(1.0),
            intervention_target: true,
            importance: 0.8,
        };
        
        assert!(graph.add_node(node).is_ok());
        assert_eq!(graph.nodes.len(), 1);
        assert_eq!(graph.metadata.num_core_variables, 1);
    }
    
    #[test]
    fn test_duplicate_node() {
        let mut graph = CausalGraph::new("test_graph".to_string());
        
        let node = CausalNode {
            id: "X".to_string(),
            name: "Treatment".to_string(),
            node_type: NodeType::Treatment,
            value: None,
            intervention_target: true,
            importance: 0.8,
        };
        
        assert!(graph.add_node(node.clone()).is_ok());
        assert!(graph.add_node(node).is_err());
    }
    
    #[test]
    fn test_graph_validation() {
        let mut graph = CausalGraph::new("test_graph".to_string());
        
        // Invalid: too few nodes
        assert!(!graph.is_valid());
        
        // Add nodes
        let treatment = CausalNode {
            id: "X".to_string(),
            name: "Treatment".to_string(),
            node_type: NodeType::Treatment,
            value: None,
            intervention_target: true,
            importance: 0.8,
        };
        
        let outcome = CausalNode {
            id: "Y".to_string(),
            name: "Outcome".to_string(),
            node_type: NodeType::Outcome,
            value: None,
            intervention_target: false,
            importance: 0.9,
        };
        
        let confounder = CausalNode {
            id: "Z".to_string(),
            name: "Confounder".to_string(),
            node_type: NodeType::Confounder,
            value: None,
            intervention_target: false,
            importance: 0.7,
        };
        
        graph.add_node(treatment).unwrap();
        graph.add_node(outcome).unwrap();
        graph.add_node(confounder).unwrap();
        
        // Invalid: no main paths
        assert!(!graph.is_valid());
        
        // Add main paths
        graph.main_paths.push(CausalPath {
            id: "path1".to_string(),
            nodes: vec!["X".to_string(), "Y".to_string()],
            strength: 0.8,
            path_type: PathType::FrontDoor,
        });
        
        graph.main_paths.push(CausalPath {
            id: "path2".to_string(),
            nodes: vec!["X".to_string(), "Z".to_string(), "Y".to_string()],
            strength: 0.6,
            path_type: PathType::BackDoor,
        });
        
        // Now valid
        assert!(graph.is_valid());
    }
}
