//! Causal Graph Construction Module
//!
//! This module provides lightweight causal graph construction for the CFO system.
//! Key features:
//! - Variable selection (3-5 core variables)
//! - Path constraints (2-3 main causal paths)
//! - Causal effect computation using do-calculus

pub mod builder;
pub mod types;
pub mod selection;
pub mod utils;
pub mod visualization;
pub mod judgment;
pub mod ai_reasoning;

// Re-exports
pub use types::{
    CausalGraph, CausalNode, CausalEdge, CausalPath,
    CausalEffect, Intervention, DoOperatorResult
};
pub use builder::{CausalGraphBuilder, GraphBuilderConfig};
pub use selection::{VariableSelector, SelectionMethod, SelectionScore};
pub use utils::{compute_causal_effect, apply_do_operator, calculate_path_influence};
pub use visualization::{print_causal_graph, generate_dot_format, print_graph_statistics};
pub use judgment::{
    CausalJudgment, JudgmentCriteria, Recommendation,
    compare_causal_graphs, GraphComparison, detect_collusion, CollusionDetection
};
pub use ai_reasoning::{
    AIReasoningEngine, AIReasoningConfig, PromptTemplate
};
