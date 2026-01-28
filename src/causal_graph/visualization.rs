//! Causal Graph Visualization Module
//!
//! Provides ASCII-based visualization for causal graphs

use crate::causal_graph::types::{CausalGraph, NodeType, EdgeType, PathType};
use std::collections::HashMap;

/// Print causal graph as ASCII art
pub fn print_causal_graph(graph: &CausalGraph) {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║           CAUSAL GRAPH VISUALIZATION                        ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║ Graph ID: {:<50} ║", graph.id);
    println!("║ Nodes: {:<3} | Paths: {:<3} | Edges: {:<3} {:<21} ║",
             graph.nodes.len(), graph.main_paths.len(), graph.edges.len(), "");
    println!("╚════════════════════════════════════════════════════════════╝");

    // Create node positions
    let positions = create_node_positions(graph);

    // Print nodes in layers
    print_nodes_by_layer(graph, &positions);

    // Print edges
    println!("\n┌─ Causal Edges ──────────────────────────────────────────────┐");
    for edge in &graph.edges {
        let source_node = graph.get_node(&edge.source);
        let target_node = graph.get_node(&edge.target);
        
        let type_str = match edge.edge_type {
            EdgeType::Direct => "→",
            EdgeType::Indirect => "⇢",
            EdgeType::Confounding => "↔",
        };
        
        let source_type = source_node.map(|n| format!("{:?}", n.node_type)).unwrap_or("???".to_string());
        let target_type = target_node.map(|n| format!("{:?}", n.node_type)).unwrap_or("???".to_string());
        
        println!("│ {} {} {} [weight: {:.3}] {}→{} │",
            edge.source, type_str, edge.target, edge.weight,
            source_type, target_type);
    }
    println!("└──────────────────────────────────────────────────────────────┘");

    // Print main paths
    if !graph.main_paths.is_empty() {
        println!("\n┌─ Main Causal Paths ───────────────────────────────────────┐");
        for (i, path) in graph.main_paths.iter().enumerate() {
            let path_str = path.nodes.join(" → ");
            let type_str = match path.path_type {
                PathType::FrontDoor => "🟢 Front-Door",
                PathType::BackDoor => "🟡 Back-Door",
                PathType::Confounded => "🔴 Confounded",
            };
            println!("│ Path {} [strength: {:.3}] {}", i + 1, path.strength, type_str);
            println!("│     {}", path_str);
        }
        println!("└──────────────────────────────────────────────────────────────┘");
    }

    // Print ASCII diagram
    println!("\n┌─ Graph Structure ────────────────────────────────────────────┐");
    print_ascii_graph(graph, &positions);
    println!("└──────────────────────────────────────────────────────────────┘");
}

/// Create positions for nodes (topological layout)
fn create_node_positions(graph: &CausalGraph) -> HashMap<String, (usize, usize)> {
    let mut positions = HashMap::new();
    let mut layer_map: HashMap<String, usize> = HashMap::new();

    // Assign layers based on node type
    for node in &graph.nodes {
        let layer = match node.node_type {
            NodeType::Treatment => 0,
            NodeType::Confounder => 1,
            NodeType::Mediator => 2,
            NodeType::Outcome => 3,
            NodeType::Control => 4,
        };
        layer_map.insert(node.id.clone(), layer);
    }

    // Count nodes in each layer
    let mut layer_counts: Vec<usize> = vec![0; 5];
    for (_, layer) in &layer_map {
        layer_counts[*layer] += 1;
    }

    // Assign positions
    let mut used_in_layer: Vec<usize> = vec![0; 5];
    for node in &graph.nodes {
        if let Some(&layer) = layer_map.get(&node.id) {
            let count = layer_counts[layer];
            let offset = if count == 1 { 6 } else { 12 };
            let pos = offset + used_in_layer[layer] * 12;
            positions.insert(node.id.clone(), (layer, pos));
            used_in_layer[layer] += 1;
        }
    }

    positions
}

/// Print nodes grouped by layer
fn print_nodes_by_layer(graph: &CausalGraph, positions: &HashMap<String, (usize, usize)>) {
    let layers = vec![
        ("Treatment Nodes", NodeType::Treatment),
        ("Confounder Nodes", NodeType::Confounder),
        ("Mediator Nodes", NodeType::Mediator),
        ("Outcome Node", NodeType::Outcome),
    ];

    for (layer_name, node_type) in &layers {
        let nodes: Vec<_> = graph.nodes.iter()
            .filter(|n| n.node_type == *node_type)
            .collect();

        if !nodes.is_empty() {
            println!("\n[{}] ({} nodes)", layer_name, nodes.len());
            for node in nodes {
                let value_str = node.value.map(|v| format!("{:.3}", v)).unwrap_or("N/A".to_string());
                println!("  {:<20} = {} [importance: {:.3}]",
                    node.name, value_str, node.importance);
            }
        }
    }
}

/// Print ASCII graph structure
fn print_ascii_graph(graph: &CausalGraph, positions: &HashMap<String, (usize, usize)>) {
    // Find maximum position for width
    let max_x = positions.values().map(|(_, x)| *x).max().unwrap_or(60);
    let max_layer = positions.values().map(|(layer, _)| *layer).max().unwrap_or(3);

    // Create grid
    let mut grid = vec![vec![' '; max_x + 10]; (max_layer + 1) * 4];

    // Place nodes
    for node in &graph.nodes {
        if let Some(&(layer, x)) = positions.get(&node.id) {
            let y = layer * 4;
            let label = format!("{:2}", node.id.replace("X", "X").replace("Y", "Y").replace("N", ""));
            
            // Place node box
            grid[y][x] = '┌';
            grid[y][x + 1] = '─';
            grid[y][x + 2] = '┐';
            grid[y + 1][x] = '│';
            grid[y + 1][x + 1] = label.chars().next().unwrap_or('?');
            grid[y + 1][x + 2] = '│';
            grid[y + 2][x] = '└';
            grid[y + 2][x + 1] = '─';
            grid[y + 2][x + 2] = '┘';
        }
    }

    // Draw edges
    for edge in &graph.edges {
        if let (Some(source_pos), Some(target_pos)) = (
            positions.get(&edge.source),
            positions.get(&edge.target)
        ) {
            let start_y = source_pos.0 * 4 + 1;
            let end_y = target_pos.0 * 4 + 1;
            let start_x = source_pos.1 + 1;
            let end_x = target_pos.1 + 1;

            // Draw horizontal line
            if start_x < end_x {
                for x in start_x + 3..=end_x - 1 {
                    if grid[start_y][x] == ' ' {
                        grid[start_y][x] = if edge.edge_type == EdgeType::Confounding { '═' } else { '─' };
                    }
                }
            } else if start_x > end_x {
                for x in end_x + 3..=start_x - 1 {
                    if grid[start_y][x] == ' ' {
                        grid[start_y][x] = if edge.edge_type == EdgeType::Confounding { '═' } else { '─' };
                    }
                }
            }

            // Draw vertical line
            for y in start_y + 1..end_y {
                if grid[y][start_x] == ' ' {
                    grid[y][start_x] = '│';
                }
            }
        }
    }

    // Print grid
    for row in grid {
        let line: String = row.iter().collect();
        if line.trim().is_empty() {
            println!("│{}│", " ".repeat(max_x + 6));
        } else {
            println!("│{}│", line);
        }
    }
}

/// Generate GraphViz DOT format for causal graph
pub fn generate_dot_format(graph: &CausalGraph) -> String {
    let mut dot = String::from("digraph CausalGraph {\n");
    dot.push_str("    rankdir=TB;\n");
    dot.push_str("    node [shape=box, style=\"rounded,filled\"];\n\n");

    // Define node styles
    dot.push_str("    // Node definitions\n");
    for node in &graph.nodes {
        let color = match node.node_type {
            NodeType::Treatment => "lightblue",
            NodeType::Outcome => "lightgreen",
            NodeType::Confounder => "lightyellow",
            NodeType::Mediator => "lightpink",
            NodeType::Control => "lightgray",
        };
        
        dot.push_str(&format!("    {} [label=\"{}\\n(val: {:.2})\", fillcolor={}];\n",
            node.id,
            node.name,
            node.value.unwrap_or(0.0),
            color));
    }

    dot.push_str("\n    // Edge definitions\n");
    for edge in &graph.edges {
        let style = match edge.edge_type {
            EdgeType::Direct => "solid",
            EdgeType::Indirect => "dashed",
            EdgeType::Confounding => "dotted",
        };
        
        dot.push_str(&format!("    {} -> {} [label=\"{:.2}\", style={}];\n",
            edge.source, edge.target, edge.weight, style));
    }

    dot.push_str("}\n");
    dot
}

/// Print causal graph summary statistics
pub fn print_graph_statistics(graph: &CausalGraph) {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║           CAUSAL GRAPH STATISTICS                          ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    
    // Node statistics
    let treatment_count = graph.nodes.iter().filter(|n| n.node_type == NodeType::Treatment).count();
    let outcome_count = graph.nodes.iter().filter(|n| n.node_type == NodeType::Outcome).count();
    let confounder_count = graph.nodes.iter().filter(|n| n.node_type == NodeType::Confounder).count();
    let mediator_count = graph.nodes.iter().filter(|n| n.node_type == NodeType::Mediator).count();
    
    println!("║ Nodes by Type:                                               ║");
    println!("║   • Treatment: {:<2}   • Outcome: {:<2}                            ║", treatment_count, outcome_count);
    println!("║   • Confounder: {:<2}  • Mediator: {:<2}                           ║", confounder_count, mediator_count);
    
    // Edge statistics
    let direct_count = graph.edges.iter().filter(|e| e.edge_type == EdgeType::Direct).count();
    let indirect_count = graph.edges.iter().filter(|e| e.edge_type == EdgeType::Indirect).count();
    let confounding_count = graph.edges.iter().filter(|e| e.edge_type == EdgeType::Confounding).count();
    
    println!("║                                                              ║");
    println!("║ Edges by Type:                                               ║");
    println!("║   • Direct: {:<2}      • Indirect: {:<2}                            ║", direct_count, indirect_count);
    println!("║   • Confounding: {:<2}                                          ║", confounding_count);
    
    // Path statistics
    let frontdoor_count = graph.main_paths.iter().filter(|p| p.path_type == PathType::FrontDoor).count();
    let backdoor_count = graph.main_paths.iter().filter(|p| p.path_type == PathType::BackDoor).count();
    
    if !graph.main_paths.is_empty() {
        println!("║                                                              ║");
        println!("║ Paths by Type:                                               ║");
        println!("║   • Front-Door: {:<2}   • Back-Door: {:<2}                          ║", frontdoor_count, backdoor_count);
        
        let avg_strength: f64 = graph.main_paths.iter().map(|p| p.strength).sum::<f64>() 
            / graph.main_paths.len() as f64;
        println!("║   • Average Path Strength: {:.4}                           ║", avg_strength);
    }
    
    // Graph hash
    println!("║                                                              ║");
    println!("║ Graph Hash: {:<50} ║", format!("{:?}", &graph.compute_hash()[..8]));
    
    println!("╚════════════════════════════════════════════════════════════╝");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::causal_graph::types::{CausalNode, CausalGraph};
    
    #[test]
    fn test_print_causal_graph() {
        let mut graph = CausalGraph::new("test_graph".to_string());
        
        graph.add_node(CausalNode {
            id: "X".to_string(),
            name: "Treatment".to_string(),
            node_type: NodeType::Treatment,
            value: Some(1.0),
            intervention_target: true,
            importance: 0.9,
        }).ok();
        
        print_causal_graph(&graph);
    }
    
    #[test]
    fn test_generate_dot_format() {
        let mut graph = CausalGraph::new("test_graph".to_string());
        
        graph.add_node(CausalNode {
            id: "X".to_string(),
            name: "Treatment".to_string(),
            node_type: NodeType::Treatment,
            value: Some(1.0),
            intervention_target: true,
            importance: 0.9,
        }).ok();
        
        let dot = generate_dot_format(&graph);
        assert!(dot.contains("digraph CausalGraph"));
        assert!(dot.contains("Treatment"));
    }
}
