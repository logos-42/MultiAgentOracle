//! AI-powered Causal Reasoning Engine
//!
//! This module provides AI-driven causal graph generation using LLMs
//! to replace or augment statistical approaches.

use crate::oracle_agent::LlmClient;
use crate::oracle_agent::LlmProvider;
use crate::causal_graph::types::{
    CausalGraph, CausalNode, CausalEdge, CausalPath, NodeType, EdgeType, PathType
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use log::{info, debug, warn};

/// Configuration for AI causal reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIReasoningConfig {
    /// LLM provider to use
    pub llm_provider: LlmProvider,
    
    /// Model name
    pub model: String,
    
    /// Temperature for LLM generation (0.0-2.0)
    pub temperature: f32,
    
    /// Maximum tokens for response
    pub max_tokens: u32,
    
    /// Enable JSON mode for structured output
    pub enable_json_mode: bool,
    
    /// Minimum number of nodes to generate
    pub min_nodes: usize,
    
    /// Maximum number of nodes to generate
    pub max_nodes: usize,
    
    /// Minimum number of causal paths
    pub min_paths: usize,
    
    /// Maximum number of causal paths
    pub max_paths: usize,
}

impl Default for AIReasoningConfig {
    fn default() -> Self {
        Self {
            llm_provider: LlmProvider::DeepSeek,
            model: "deepseek-chat".to_string(),
            temperature: 0.7,
            max_tokens: 2000,
            enable_json_mode: true,
            min_nodes: 3,
            max_nodes: 5,
            min_paths: 2,
            max_paths: 3,
        }
    }
}

/// AI-generated causal node response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AINode {
    /// Node ID
    pub id: String,
    /// Node name
    pub name: String,
    /// Node type
    pub node_type: String,
    /// Importance score (0.0-1.0)
    pub importance: f64,
    /// Whether this can be intervened upon
    #[serde(default)]
    pub intervention_target: bool,
}

/// AI-generated causal edge response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AIEdge {
    /// Edge ID
    pub id: String,
    /// Source node ID
    pub source: String,
    /// Target node ID
    pub target: String,
    /// Causal strength (0.0-1.0)
    pub weight: f64,
    /// Edge type
    pub edge_type: String,
}

/// AI-generated causal path response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AIPath {
    /// Path ID
    pub id: String,
    /// Sequence of node IDs
    pub nodes: Vec<String>,
    /// Path strength
    pub strength: f64,
    /// Path type
    pub path_type: String,
}

/// AI-generated complete causal graph response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AICausalResponse {
    /// List of causal nodes
    pub nodes: Vec<AINode>,
    /// List of causal edges
    pub edges: Vec<AIEdge>,
    /// List of main causal paths
    pub paths: Vec<AIPath>,
    /// Explanation/reasoning from AI
    #[serde(default)]
    pub reasoning: String,
    /// Confidence score (0.0-1.0)
    #[serde(default)]
    pub confidence: f64,
}

/// Prompt template with variable placeholders
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    /// Template content
    pub content: String,
}

impl PromptTemplate {
    /// Create a new prompt template
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }
    
    /// Replace variables in the template
    pub fn render(&self, variables: &HashMap<String, String>) -> String {
        let mut content = self.content.clone();
        for (key, value) in variables {
            content = content.replace(&format!("{{{{{}}}}}", key), value);
        }
        content
    }
}

/// AI Causal Reasoning Engine
pub struct AIReasoningEngine {
    /// LLM client
    llm_client: LlmClient,
    /// Configuration
    config: AIReasoningConfig,
}

impl AIReasoningEngine {
    /// Create a new AI reasoning engine
    pub fn new(config: AIReasoningConfig) -> Result<Self> {
        let mut llm_config = match config.llm_provider {
            LlmProvider::OpenAI => {
                crate::oracle_agent::LlmClientConfig::openai(&config.model)
                    .with_temperature(config.temperature)
                    .with_max_tokens(config.max_tokens)
            },
            LlmProvider::Anthropic => {
                crate::oracle_agent::LlmClientConfig::anthropic(&config.model)
                    .with_temperature(config.temperature)
                    .with_max_tokens(config.max_tokens)
            },
            LlmProvider::DeepSeek => {
                crate::oracle_agent::LlmClientConfig::deepseek(&config.model)
                    .with_temperature(config.temperature)
                    .with_max_tokens(config.max_tokens)
            },
            LlmProvider::Minimax => {
                crate::oracle_agent::LlmClientConfig::minimax(&config.model)
                    .with_temperature(config.temperature)
                    .with_max_tokens(config.max_tokens)
            },
            LlmProvider::Local => {
                crate::oracle_agent::LlmClientConfig::local(
                    "http://localhost:11434/api/generate",
                    &config.model
                )
                .with_temperature(config.temperature)
                .with_max_tokens(config.max_tokens)
            },
        };
        
        // 如果启用 JSON 模式，配置客户端强制返回 JSON
        if config.enable_json_mode {
            llm_config = llm_config.with_json_mode();
        }
        
        let llm_client = LlmClient::new(llm_config)?;
        
        info!("✅ AI推理引擎初始化成功: {:?} ({})", 
              config.llm_provider, config.model);
        
        Ok(Self {
            llm_client,
            config,
        })
    }
    
    /// Create engine from existing LLM client
    pub fn from_client(llm_client: LlmClient, config: AIReasoningConfig) -> Self {
        Self {
            llm_client,
            config,
        }
    }
    
    /// Generate a causal graph from a prompt
    pub async fn generate_causal_graph(&self, prompt: &str, context: &str) -> Result<CausalGraph> {
        info!("🤖 开始AI因果图生成...");
        
        // Build the full prompt
        let full_prompt = self.build_causal_analysis_prompt(prompt, context)?;
        
        // Call LLM
        let response = self.llm_client.generate_response(&full_prompt).await?;
        
        debug!("LLM响应: {}", response.text);
        
        // Parse response
        let ai_response = self.parse_ai_response(&response.text)?;
        
        // Validate response
        self.validate_ai_response(&ai_response)?;
        
        // Convert to CausalGraph
        let mut graph = self.convert_to_causal_graph(&ai_response)?;
        
        // Final validation
        if !graph.is_valid() {
            warn!("⚠️ AI生成的因果图未通过验证，尝试修复...");
            self.attempt_graph_fix(&mut graph)?;
        }
        
        info!("✅ AI因果图生成完成，置信度: {:.2}", ai_response.confidence);
        
        Ok(graph)
    }
    
    /// Build the causal analysis prompt
    fn build_causal_analysis_prompt(&self, user_prompt: &str, context: &str) -> Result<String> {
        let template = PromptTemplate::new(include_str!("prompts/causal_analysis.txt"));
        
        let mut variables = HashMap::new();
        variables.insert("SCENARIO".to_string(), user_prompt.to_string());
        variables.insert("CONTEXT".to_string(), context.to_string());
        variables.insert("MIN_NODES".to_string(), self.config.min_nodes.to_string());
        variables.insert("MAX_NODES".to_string(), self.config.max_nodes.to_string());
        variables.insert("MIN_PATHS".to_string(), self.config.min_paths.to_string());
        variables.insert("MAX_PATHS".to_string(), self.config.max_paths.to_string());
        
        Ok(template.render(&variables))
    }
    
    /// Parse AI response into structured format
    fn parse_ai_response(&self, response: &str) -> Result<AICausalResponse> {
        let response_trimmed = response.trim();
        
        // 尝试1: 直接解析JSON
        match serde_json::from_str::<AICausalResponse>(response_trimmed) {
            Ok(parsed) => {
                eprintln!("✅ 直接解析JSON成功");
                return Ok(parsed);
            }
            Err(_) => {
                // 直接解析失败是正常的，因为响应可能是markdown格式，不需要输出警告
                // eprintln!("⚠️ 直接解析失败: {}", e);
            }
        }
        
        // 尝试2: 从markdown中提取JSON
        if let Some(json_data) = Self::extract_json_from_markdown(response_trimmed) {
            eprintln!("📝 从markdown提取到JSON数据，长度: {}", json_data.len());
            
            match serde_json::from_str::<AICausalResponse>(&json_data) {
                Ok(parsed) => {
                    eprintln!("✅ Markdown提取的JSON解析成功");
                    return Ok(parsed);
                }
                Err(e) => {
                    eprintln!("⚠️ Markdown提取的JSON解析失败: {}", e);
                    
                    // 尝试修复
                    if let Some(fixed) = Self::fix_truncated_json(&json_data) {
                        match serde_json::from_str::<AICausalResponse>(&fixed) {
                            Ok(parsed) => {
                                eprintln!("✅ 修复后的JSON解析成功");
                                return Ok(parsed);
                            }
                            Err(e) => {
                                eprintln!("⚠️ 修复后的JSON解析失败: {}", e);
                            }
                        }
                    }
                }
            }
        } else {
            eprintln!("⚠️ 无法从markdown提取JSON数据");
        }
        
        // 尝试3: 查找第一个{到最后一个}
        if let Some(start) = response_trimmed.find('{') {
            if let Some(end) = response_trimmed.rfind('}') {
                if end > start {
                    let json_str = &response_trimmed[start..=end];
                    eprintln!("📝 尝试解析从 {{ 到 }} 的内容，长度: {}", json_str.len());
                    
                    match serde_json::from_str::<AICausalResponse>(json_str) {
                        Ok(parsed) => {
                            eprintln!("✅ 从{{}}提取的JSON解析成功");
                            return Ok(parsed);
                        }
                        Err(e) => {
                            eprintln!("⚠️ 从{{}}提取的JSON解析失败: {}", e);
                            
                            // 尝试修复
                            if let Some(fixed) = Self::fix_truncated_json(json_str) {
                                match serde_json::from_str::<AICausalResponse>(&fixed) {
                                    Ok(parsed) => {
                                        eprintln!("✅ 修复后的{{}} JSON解析成功");
                                        return Ok(parsed);
                                    }
                                    Err(e) => {
                                        eprintln!("⚠️ 修复后的{{}} JSON解析失败: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // 所有尝试都失败
        Err(anyhow!("无法解析AI响应为JSON格式"))
    }
    
    /// 修复截断或不完整的JSON
    fn fix_truncated_json(json_str: &str) -> Option<String> {
        let mut fixed = json_str.to_string();
        
        // 计算括号平衡
        let open_braces = fixed.matches('{').count();
        let close_braces = fixed.matches('}').count();
        let open_brackets = fixed.matches('[').count();
        let close_brackets = fixed.matches(']').count();
        
        // 补全缺失的闭合括号
        for _ in 0..(open_braces - close_braces) {
            fixed.push('}');
        }
        for _ in 0..(open_brackets - close_brackets) {
            fixed.push(']');
        }
        
        // 修复尾随逗号
        fixed = fixed.replace(",}", "}").replace(",]", "]");
        
        // 如果JSON看起来完整，尝试解析
        if fixed.parse::<serde_json::Value>().is_ok() {
            return Some(fixed);
        }
        
        None
    }
    
    /// 从markdown格式中提取JSON数据
    fn extract_json_from_markdown(response: &str) -> Option<String> {
        // 策略1: 查找 "### 详细数据" 或 "JSON" 标记后的代码块
        let markers = ["### 详细数据", "### JSON", "详细数据", "JSON数据"];
        
        for marker in &markers {
            if let Some(pos) = response.find(marker) {
                let after_marker = &response[pos + marker.len()..];
                // 查找接下来的 ```json 或 ``` 代码块
                if let Some(code_start) = after_marker.find("```json") {
                    let after_code = &after_marker[code_start + 7..];
                    if let Some(code_end) = after_code.find("```") {
                        return Some(after_code[..code_end].trim().to_string());
                    }
                } else if let Some(code_start) = after_marker.find("```") {
                    let after_code = &after_marker[code_start + 3..];
                    if let Some(code_end) = after_code.find("```") {
                        return Some(after_code[..code_end].trim().to_string());
                    }
                }
            }
        }
        
        // 策略2: 查找所有 ```json 代码块（返回最大的一个，通常是完整数据）
        let mut best_json: Option<String> = None;
        let mut search_start = 0;
        
        while let Some(code_start) = response[search_start..].find("```json") {
            let actual_start = search_start + code_start + 7;
            if let Some(code_end) = response[actual_start..].find("```") {
                let json_content = response[actual_start..actual_start + code_end].trim();
                // 选择最长的有效JSON
                if json_content.starts_with('{') && json_content.len() > best_json.as_ref().map_or(0, |s| s.len()) {
                    best_json = Some(json_content.to_string());
                }
                search_start = actual_start + code_end + 3;
            } else {
                // 找到了开始但没有结束，可能是截断的JSON
                let partial_json = response[actual_start..].trim();
                if partial_json.starts_with('{') && partial_json.len() > best_json.as_ref().map_or(0, |s| s.len()) {
                    best_json = Some(partial_json.to_string());
                }
                break;
            }
        }
        
        if best_json.is_some() {
            return best_json;
        }
        
        // 策略3: 查找 ``` 代码块（可能是纯JSON）
        if let Some(start) = response.find("```") {
            let after_start = &response[start + 3..];
            // 跳过可能的语言标识符（如 json）
            let content_start = if after_start.starts_with("json") { 4 } else { 0 };
            let after_lang = &after_start[content_start..];
            
            if let Some(end) = after_lang.find("```") {
                let json_content = after_lang[..end].trim();
                if json_content.starts_with('{') {
                    return Some(json_content.to_string());
                }
            }
        }
        
        None
    }
    
    /// Validate AI-generated response
    fn validate_ai_response(&self, response: &AICausalResponse) -> Result<()> {
        // Check node count
        if response.nodes.len() < self.config.min_nodes || 
           response.nodes.len() > self.config.max_nodes {
            return Err(anyhow!(
                "节点数量不符合要求: {} (要求: {}-{})",
                response.nodes.len(),
                self.config.min_nodes,
                self.config.max_nodes
            ));
        }
        
        // Check path count
        if response.paths.len() < self.config.min_paths || 
           response.paths.len() > self.config.max_paths {
            return Err(anyhow!(
                "因果路径数量不符合要求: {} (要求: {}-{})",
                response.paths.len(),
                self.config.min_paths,
                self.config.max_paths
            ));
        }
        
        // Check for at least one treatment and outcome
        let has_treatment = response.nodes.iter().any(|n| 
            n.node_type.to_lowercase().contains("treatment"));
        let has_outcome = response.nodes.iter().any(|n| 
            n.node_type.to_lowercase().contains("outcome"));
        
        if !has_treatment || !has_outcome {
            return Err(anyhow!("因果图必须包含至少一个treatment和一个outcome节点"));
        }
        
        // Check confidence
        if response.confidence < 0.5 {
            warn!("⚠️ AI置信度较低: {:.2}", response.confidence);
        }
        
        Ok(())
    }
    
    /// Convert AI response to CausalGraph
    fn convert_to_causal_graph(&self, response: &AICausalResponse) -> Result<CausalGraph> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut graph = CausalGraph {
            id: format!("ai_graph_{}", now),
            nodes: Vec::new(),
            edges: Vec::new(),
            main_paths: Vec::new(),
            metadata: crate::causal_graph::types::GraphMetadata {
                created_at: now,
                updated_at: now,
                num_core_variables: response.nodes.len(),
                num_main_paths: response.paths.len(),
                version: "1.0.0".to_string(),
            },
        };
        
        // Convert nodes
        for ai_node in &response.nodes {
            let node_type = self.parse_node_type(&ai_node.node_type)?;
            let node = CausalNode {
                id: ai_node.id.clone(),
                name: ai_node.name.clone(),
                node_type,
                value: None,
                intervention_target: ai_node.intervention_target,
                importance: ai_node.importance.clamp(0.0, 1.0),
            };
            graph.add_node(node).map_err(|e| anyhow::anyhow!(e))?;
        }
        
        // Convert edges
        for ai_edge in &response.edges {
            let edge_type = self.parse_edge_type(&ai_edge.edge_type)?;
            let edge = CausalEdge {
                id: ai_edge.id.clone(),
                source: ai_edge.source.clone(),
                target: ai_edge.target.clone(),
                weight: ai_edge.weight.clamp(-1.0, 1.0),
                edge_type,
            };
            graph.add_edge(edge).map_err(|e| anyhow::anyhow!(e))?;
        }
        
        // Convert paths
        for ai_path in &response.paths {
            let path_type = self.parse_path_type(&ai_path.path_type)?;
            let path = CausalPath {
                id: ai_path.id.clone(),
                nodes: ai_path.nodes.clone(),
                strength: ai_path.strength.clamp(0.0, 1.0),
                path_type,
            };
            graph.main_paths.push(path);
        }
        
        Ok(graph)
    }
    
    /// Parse node type from string
    fn parse_node_type(&self, type_str: &str) -> Result<NodeType> {
        let type_lower = type_str.to_lowercase();
        match type_lower.as_str() {
            "treatment" | "干预" => Ok(NodeType::Treatment),
            "outcome" | "结果" => Ok(NodeType::Outcome),
            "confounder" | "混淆因子" => Ok(NodeType::Confounder),
            "mediator" | "中介因子" => Ok(NodeType::Mediator),
            "control" | "控制变量" => Ok(NodeType::Control),
            _ => {
                warn!("未知节点类型: {}, 默认为Control", type_str);
                Ok(NodeType::Control)
            }
        }
    }
    
    /// Parse edge type from string
    fn parse_edge_type(&self, type_str: &str) -> Result<EdgeType> {
        let type_lower = type_str.to_lowercase();
        match type_lower.as_str() {
            "direct" | "直接" => Ok(EdgeType::Direct),
            "indirect" | "间接" => Ok(EdgeType::Indirect),
            "confounding" | "混淆" => Ok(EdgeType::Confounding),
            _ => {
                warn!("未知边类型: {}, 默认为Indirect", type_str);
                Ok(EdgeType::Indirect)
            }
        }
    }
    
    /// Parse path type from string
    fn parse_path_type(&self, type_str: &str) -> Result<PathType> {
        let type_lower = type_str.to_lowercase();
        match type_lower.as_str() {
            "frontdoor" | "前门" => Ok(PathType::FrontDoor),
            "backdoor" | "后门" => Ok(PathType::BackDoor),
            "confounded" | "混淆" => Ok(PathType::Confounded),
            _ => {
                warn!("未知路径类型: {}, 默认为FrontDoor", type_str);
                Ok(PathType::FrontDoor)
            }
        }
    }
    
    /// Attempt to fix invalid graph
    fn attempt_graph_fix(&self, graph: &mut CausalGraph) -> Result<()> {
        // Fix node count
        if graph.nodes.len() < self.config.min_nodes {
            warn!("节点不足，添加默认节点");
            for i in graph.nodes.len()..self.config.min_nodes {
                let node = CausalNode {
                    id: format!("N{}", i),
                    name: format!("Variable_{}", i),
                    node_type: NodeType::Control,
                    value: None,
                    intervention_target: false,
                    importance: 0.5,
                };
                let _ = graph.add_node(node);
            }
        } else if graph.nodes.len() > self.config.max_nodes {
            warn!("节点过多，移除额外节点");
            graph.nodes.truncate(self.config.max_nodes);
            graph.metadata.num_core_variables = graph.nodes.len();
        }
        
        // Fix path count
        if graph.main_paths.len() < self.config.min_paths {
            warn!("路径不足，添加默认路径");
            while graph.main_paths.len() < self.config.min_paths && graph.nodes.len() >= 2 {
                let path = CausalPath {
                    id: format!("path_{}", graph.main_paths.len()),
                    nodes: vec![
                        graph.nodes[0].id.clone(),
                        graph.nodes[1].id.clone()
                    ],
                    strength: 0.5,
                    path_type: PathType::FrontDoor,
                };
                graph.main_paths.push(path);
            }
            graph.metadata.num_main_paths = graph.main_paths.len();
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prompt_template() {
        let template = PromptTemplate::new("Hello {{NAME}}, today is {{DAY}}");
        let mut vars = HashMap::new();
        vars.insert("NAME".to_string(), "Alice".to_string());
        vars.insert("DAY".to_string(), "Monday".to_string());
        
        let result = template.render(&vars);
        assert_eq!(result, "Hello Alice, today is Monday");
    }
    
    #[test]
    fn test_node_type_parsing() {
        let config = AIReasoningConfig::default();
        let engine = AIReasoningEngine {
            llm_client: unsafe { std::mem::zeroed() },  // For testing only
            config,
        };
        
        assert!(matches!(engine.parse_node_type("treatment").unwrap(), NodeType::Treatment));
        assert!(matches!(engine.parse_node_type("outcome").unwrap(), NodeType::Outcome));
        assert!(matches!(engine.parse_node_type("confounder").unwrap(), NodeType::Confounder));
    }
    
    #[test]
    fn test_edge_type_parsing() {
        let config = AIReasoningConfig::default();
        let engine = AIReasoningEngine {
            llm_client: unsafe { std::mem::zeroed() },  // For testing only
            config,
        };
        
        assert!(matches!(engine.parse_edge_type("direct").unwrap(), EdgeType::Direct));
        assert!(matches!(engine.parse_edge_type("indirect").unwrap(), EdgeType::Indirect));
        assert!(matches!(engine.parse_edge_type("confounding").unwrap(), EdgeType::Confounding));
    }
}
