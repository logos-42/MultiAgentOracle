//! 简单Prompt支持系统
//! 
//! 支持智能体响应简单prompt，用于测试分层架构的交互能力

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Prompt处理器类型
pub type PromptHandler = Box<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

/// 简单Prompt支持系统
pub struct SimplePromptSupport {
    prompt_handlers: Arc<RwLock<HashMap<String, PromptHandler>>>,
    test_prompts: Vec<TestPrompt>,
}

/// 测试prompt定义
#[derive(Debug, Clone)]
pub struct TestPrompt {
    pub command: String,
    pub description: String,
    pub expected_response_pattern: String,
    pub tier_restriction: Option<String>, // 可选的层级限制
}

impl SimplePromptSupport {
    /// 创建新的Prompt支持系统
    pub fn new() -> Self {
        let mut support = Self {
            prompt_handlers: Arc::new(RwLock::new(HashMap::new())),
            test_prompts: Vec::new(),
        };
        
        // 注册默认的prompt处理器
        support.register_default_handlers();
        
        // 创建测试prompt
        support.create_test_prompts();
        
        support
    }
    
    /// 注册默认的prompt处理器
    fn register_default_handlers(&mut self) {
        // 状态查询命令
        self.register_handler("status", Box::new(|_| {
            Ok("系统状态正常，所有节点在线".to_string())
        }));
        
        // 网络拓扑查询
        self.register_handler("topology", Box::new(|_| {
            Ok("分层网络拓扑：核心层(2节点) -> 验证层(3节点) -> 数据层(5节点)".to_string())
        }));
        
        // 共识状态查询
        self.register_handler("consensus", Box::new(|_| {
            Ok("共识引擎运行正常，最近一次共识成功率：95%".to_string())
        }));
        
        // 信誉查询
        self.register_handler("reputation", Box::new(|args| {
            if args.is_empty() {
                Ok("请输入节点ID，例如：reputation node1".to_string())
            } else {
                Ok(format!("节点 {} 的信誉分：850.0，层级：core", args))
            }
        }));
        
        // 数据查询
        self.register_handler("data", Box::new(|args| {
            if args.is_empty() {
                Ok("请输入数据类型，例如：data crypto".to_string())
            } else {
                match args {
                    "crypto" => Ok("BTC: $45,200, ETH: $3,150, SOL: $120".to_string()),
                    "stock" => Ok("AAPL: $185, TSLA: $240, NVDA: $950".to_string()),
                    "weather" => Ok("北京: 25°C 晴, 上海: 28°C 多云, 深圳: 30°C 阵雨".to_string()),
                    _ => Ok(format!("未知数据类型: {}", args)),
                }
            }
        }));
        
        // 层级信息查询
        self.register_handler("tier", Box::new(|args| {
            if args.is_empty() {
                Ok("核心层：高信誉节点，负责最终共识\n验证层：中等信誉节点，负责数据验证\n数据层：基础节点，负责数据采集".to_string())
            } else {
                match args {
                    "core" => Ok("核心层：需要信誉分≥800，质押≥0.5，负责最终共识决策".to_string()),
                    "validator" => Ok("验证层：需要信誉分500-799，质押≥0.3，负责数据验证".to_string()),
                    "data" => Ok("数据层：信誉分<500，质押≥0.1，负责数据采集和提交".to_string()),
                    _ => Ok(format!("未知层级: {}", args)),
                }
            }
        }));
        
        // 帮助命令
        self.register_handler("help", Box::new(|_| {
            let commands = vec![
                "status - 查看系统状态",
                "topology - 查看网络拓扑",
                "consensus - 查看共识状态",
                "reputation <node> - 查询节点信誉",
                "data <type> - 查询数据（crypto/stock/weather）",
                "tier [level] - 查看层级信息",
                "help - 显示帮助信息",
            ];
            Ok(commands.join("\n"))
        }));
    }
    
    /// 创建测试prompt
    fn create_test_prompts(&mut self) {
        self.test_prompts = vec![
            TestPrompt {
                command: "status".to_string(),
                description: "查询系统状态".to_string(),
                expected_response_pattern: "系统状态正常".to_string(),
                tier_restriction: None,
            },
            TestPrompt {
                command: "topology".to_string(),
                description: "查询网络拓扑".to_string(),
                expected_response_pattern: "分层网络拓扑".to_string(),
                tier_restriction: None,
            },
            TestPrompt {
                command: "consensus".to_string(),
                description: "查询共识状态".to_string(),
                expected_response_pattern: "共识引擎运行正常".to_string(),
                tier_restriction: Some("core".to_string()),
            },
            TestPrompt {
                command: "reputation node1".to_string(),
                description: "查询节点信誉".to_string(),
                expected_response_pattern: "信誉分".to_string(),
                tier_restriction: None,
            },
            TestPrompt {
                command: "data crypto".to_string(),
                description: "查询加密货币数据".to_string(),
                expected_response_pattern: "BTC".to_string(),
                tier_restriction: Some("data".to_string()),
            },
            TestPrompt {
                command: "tier core".to_string(),
                description: "查询核心层信息".to_string(),
                expected_response_pattern: "核心层".to_string(),
                tier_restriction: None,
            },
            TestPrompt {
                command: "help".to_string(),
                description: "显示帮助信息".to_string(),
                expected_response_pattern: "status".to_string(),
                tier_restriction: None,
            },
        ];
    }
    
    /// 注册prompt处理器
    pub fn register_handler(&mut self, command: &str, handler: PromptHandler) {
        let mut handlers = self.prompt_handlers.blocking_write();
        handlers.insert(command.to_string(), handler);
    }
    
    /// 处理智能体prompt
    pub async fn handle_prompt(&self, agent_id: &str, prompt: &str) -> Result<String, String> {
        println!("🤖 节点 {} 处理prompt: {}", agent_id, prompt);
        
        let handlers = self.prompt_handlers.read().await;
        
        // 分割命令和参数
        let parts: Vec<&str> = prompt.split_whitespace().collect();
        if parts.is_empty() {
            return Err("请输入有效的命令".to_string());
        }
        
        let command = parts[0];
        let args = if parts.len() > 1 {
            parts[1..].join(" ")
        } else {
            String::new()
        };
        
        // 查找处理器
        if let Some(handler) = handlers.get(command) {
            match handler(&args) {
                Ok(response) => {
                    println!("  响应: {}", response);
                    Ok(response)
                }
                Err(e) => {
                    println!("  错误: {}", e);
                    Err(e)
                }
            }
        } else {
            let error_msg = format!("未知命令: {}，输入 help 查看可用命令", command);
            println!("  {}", error_msg);
            Err(error_msg)
        }
    }
    
    /// 获取预定义测试prompt
    pub fn get_test_prompts(&self) -> &Vec<TestPrompt> {
        &self.test_prompts
    }
    
    /// 运行prompt测试套件
    pub async fn run_prompt_test_suite(&self, agent_tier: &str) -> PromptTestResult {
        println!("🧪 运行Prompt测试套件 (层级: {})", agent_tier);
        
        let mut results = HashMap::new();
        let mut total_success = 0;
        let mut total_failures = 0;
        let mut total_response_time = 0.0;
        
        for test_prompt in &self.test_prompts {
            // 检查层级限制
            if let Some(required_tier) = &test_prompt.tier_restriction {
                if agent_tier != required_tier {
                    println!("  ⚠️  跳过 {} (需要 {} 层，当前为 {} 层)", 
                        test_prompt.command, required_tier, agent_tier);
                    continue;
                }
            }
            
            println!("  测试: {} - {}", test_prompt.command, test_prompt.description);
            
            let start_time = std::time::Instant::now();
            let result = self.handle_prompt("test_agent", &test_prompt.command).await;
            let response_time = start_time.elapsed().as_millis() as f64;
            total_response_time += response_time;
            
            let success = match &result {
                Ok(response) => response.contains(&test_prompt.expected_response_pattern),
                Err(_) => false,
            };
            
            if success {
                total_success += 1;
                println!("    ✅ 成功 (响应时间: {:.1}ms)", response_time);
            } else {
                total_failures += 1;
                println!("    ❌ 失败 (响应时间: {:.1}ms)", response_time);
            }
            
            results.insert(
                test_prompt.command.clone(),
                CommandStats {
                    command: test_prompt.command.clone(),
                    success_count: if success { 1 } else { 0 },
                    failure_count: if success { 0 } else { 1 },
                    average_response_time_ms: response_time,
                },
            );
        }
        
        let total_tests = total_success + total_failures;
        let success_rate = if total_tests > 0 {
            total_success as f64 / total_tests as f64
        } else {
            0.0
        };
        
        let average_response_time = if total_tests > 0 {
            total_response_time / total_tests as f64
        } else {
            0.0
        };
        
        println!("📊 Prompt测试结果:");
        println!("  成功率: {:.1}% ({}/{})", 
            success_rate * 100.0, total_success, total_tests);
        println!("  平均响应时间: {:.1}ms", average_response_time);
        
        PromptTestResult {
            prompt_success_rate: success_rate,
            average_response_time_ms: average_response_time,
            command_coverage: results,
            tier_response_stats: HashMap::new(),
        }
    }
    
    /// 显示可用命令
    pub async fn show_available_commands(&self) {
        let handlers = self.prompt_handlers.read().await;
        
        println!("📋 可用命令:");
        println!("====================");
        
        for (command, _) in handlers.iter() {
            println!("  - {}", command);
        }
        
        println!("\n使用示例:");
        println!("  status - 查询系统状态");
        println!("  data crypto - 查询加密货币价格");
        println!("  reputation node1 - 查询节点信誉");
        println!("  help - 显示详细帮助");
    }
}

/// Prompt测试结果
#[derive(Debug, Clone)]
pub struct PromptTestResult {
    pub prompt_success_rate: f64,
    pub average_response_time_ms: f64,
    pub command_coverage: HashMap<String, CommandStats>,
    pub tier_response_stats: HashMap<String, TierResponseStats>,
}

/// 命令统计
#[derive(Debug, Clone)]
pub struct CommandStats {
    pub command: String,
    pub success_count: u32,
    pub failure_count: u32,
    pub average_response_time_ms: f64,
}

/// 层级响应统计
#[derive(Debug, Clone)]
pub struct TierResponseStats {
    pub tier: String,
    pub response_success_rate: f64,
    pub average_response_quality: f64,
}

impl Default for SimplePromptSupport {
    fn default() -> Self {
        Self::new()
    }
}
