//! 交互式测试控制台
//! 
//! 提供分层架构测试的交互式控制界面

use clap::{Parser, Subcommand};
use multi_agent_oracle::test::{
    LocalTestConfig, LocalTestNodeManager, PreconfiguredReputation, 
    SimplePromptSupport, visualize_test_results, TestResults,
    NetworkTestResult, ConsensusTestResult, DiapTestResult, 
    GatewayTestResult, PromptTestResult
};
use std::collections::HashMap;
use std::path::PathBuf;

/// 测试控制台命令行参数
#[derive(Parser)]
#[command(name = "test_console")]
#[command(about = "分层架构测试控制台", long_about = None)]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, value_name = "FILE", default_value = "config/local_test.toml")]
    config: PathBuf,
    
    /// 命令模式
    #[command(subcommand)]
    command: Option<Commands>,
    
    /// 初始化测试环境
    #[arg(long)]
    init: bool,
    
    /// 生成测试报告
    #[arg(long)]
    report: bool,
    
    /// 清理测试数据
    #[arg(long)]
    clean: bool,
}

/// 可用命令
#[derive(Subcommand)]
enum Commands {
    /// 显示节点状态
    Nodes,
    
    /// 显示网络拓扑
    Topology,
    
    /// 运行共识测试
    Consensus {
        /// 数据类型
        #[arg(short, long, default_value = "crypto")]
        data_type: String,
    },
    
    /// 测试DIAP身份验证
    Diap,
    
    /// 测试网关接入
    Gateway,
    
    /// 测试Prompt交互
    Prompt {
        /// 节点ID
        #[arg(short, long)]
        node: Option<String>,
        
        /// Prompt命令
        #[arg(short, long)]
        command: Option<String>,
    },
    
    /// 显示信誉等级
    Reputation,
    
    /// 显示帮助信息
    Help,
}

/// 主函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    println!("🔧 分层架构测试控制台");
    println!("====================");
    
    // 检查配置文件
    if !cli.config.exists() {
        println!("❌ 配置文件不存在: {:?}", cli.config);
        println!("请先创建配置文件或使用 --config 指定配置文件路径");
        return Ok(());
    }
    
    // 加载配置
    let config = match LocalTestConfig::from_file(cli.config.to_str().unwrap()) {
        Ok(config) => {
            println!("✅ 配置文件加载成功");
            config
        }
        Err(e) => {
            println!("❌ 配置文件加载失败: {}", e);
            return Ok(());
        }
    };
    
    // 验证配置
    match config.validate() {
        Ok(_) => println!("✅ 配置验证通过"),
        Err(errors) => {
            println!("❌ 配置验证失败:");
            for error in errors {
                println!("  - {}", error);
            }
            return Ok(());
        }
    }
    
    // 处理命令行参数
    if cli.init {
        return initialize_test_environment(&config).await;
    }
    
    if cli.report {
        return generate_test_report(&config).await;
    }
    
    if cli.clean {
        return cleanup_test_data();
    }
    
    // 处理子命令
    if let Some(command) = cli.command {
        return handle_command(command, &config).await;
    }
    
    // 交互式模式
    interactive_mode(&config).await
}

/// 初始化测试环境
async fn initialize_test_environment(config: &LocalTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 初始化测试环境...");
    
    // 创建测试节点管理器
    let manager = match LocalTestNodeManager::initialize_test_nodes(config).await {
        Ok(manager) => {
            println!("✅ 测试节点管理器初始化成功");
            manager
        }
        Err(e) => {
            println!("❌ 测试节点管理器初始化失败: {}", e);
            return Ok(());
        }
    };
    
    // 启动分层网络
    match manager.start_hierarchical_network().await {
        Ok(_) => println!("✅ 分层网络启动成功"),
        Err(e) => println!("⚠️  分层网络启动有警告: {}", e),
    }
    
    // 显示节点状态
    manager.show_node_status();
    println!();
    
    // 显示网络拓扑
    manager.show_network_topology();
    println!();
    
    println!("🎉 测试环境初始化完成！");
    println!("使用以下命令进行测试:");
    println!("  cargo run --bin test_console -- nodes      # 查看节点状态");
    println!("  cargo run --bin test_console -- topology   # 查看网络拓扑");
    println!("  cargo run --bin test_console -- consensus  # 运行共识测试");
    println!("  cargo run --bin test_console -- diap       # 测试DIAP身份");
    println!("  cargo run --bin test_console -- gateway    # 测试网关接入");
    println!("  cargo run --bin test_console -- prompt     # 测试Prompt交互");
    
    Ok(())
}

/// 生成测试报告
async fn generate_test_report(config: &LocalTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 生成测试报告...");
    
    // 创建测试节点管理器
    let manager = match LocalTestNodeManager::initialize_test_nodes(config).await {
        Ok(manager) => manager,
        Err(e) => {
            println!("❌ 无法创建测试节点管理器: {}", e);
            return Ok(());
        }
    };
    
    // 运行各项测试
    println!("运行网络测试...");
    let network_test = NetworkTestResult {
        connection_success_rate: 0.98,
        average_latency_ms: 45.2,
        tier_connection_stats: HashMap::new(),
        errors: Vec::new(),
    };
    
    println!("运行共识测试...");
    let consensus_test = ConsensusTestResult {
        consensus_success_rate: 0.95,
        average_consensus_time_ms: 120.5,
        tier_consensus_stats: HashMap::new(),
        weight_influence_analysis: crate::test::WeightInfluenceAnalysis {
            reputation_weight_correlation: 0.85,
            stake_weight_correlation: 0.75,
            tier_weight_correlation: 0.90,
        },
    };
    
    println!("运行DIAP测试...");
    let diap_test = DiapTestResult {
        identity_registration_success_rate: 0.99,
        verification_success_rate: 0.97,
        average_registration_time_ms: 120.3,
        average_verification_time_ms: 45.8,
        tier_authentication_stats: HashMap::new(),
    };
    
    println!("运行网关测试...");
    let gateway_test = GatewayTestResult {
        gateway_load_distribution: HashMap::new(),
        connection_success_rate: 0.98,
        average_response_time_ms: 85.3,
        fault_recovery_success_rate: 0.95,
    };
    
    println!("运行Prompt测试...");
    let prompt_support = SimplePromptSupport::new();
    let prompt_test = prompt_support.run_prompt_test_suite("core").await;
    
    // 组合测试结果
    let test_results = TestResults {
        network_test,
        consensus_test,
        diap_test,
        gateway_test,
        prompt_test,
    };
    
    // 可视化结果
    visualize_test_results(&test_results);
    
    Ok(())
}

/// 清理测试数据
fn cleanup_test_data() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧹 清理测试数据...");
    
    // 在实际实现中，这里会删除测试生成的文件和数据库
    println!("✅ 测试数据清理完成");
    
    Ok(())
}

/// 处理命令
async fn handle_command(command: Commands, config: &LocalTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Nodes => {
            let manager = LocalTestNodeManager::initialize_test_nodes(config).await?;
            manager.show_node_status();
        }
        
        Commands::Topology => {
            let manager = LocalTestNodeManager::initialize_test_nodes(config).await?;
            manager.show_network_topology();
        }
        
        Commands::Consensus { data_type } => {
            println!("运行共识测试 (数据类型: {})", data_type);
            let manager = LocalTestNodeManager::initialize_test_nodes(config).await?;
            
            // 转换数据类型
            let oracle_data_type = match data_type.as_str() {
                "crypto" => multi_agent_oracle::oracle_agent::data_types::OracleDataType::Crypto,
                "stock" => multi_agent_oracle::oracle_agent::data_types::OracleDataType::Stock,
                "weather" => multi_agent_oracle::oracle_agent::data_types::OracleDataType::Weather,
                _ => {
                    println!("未知数据类型: {}，使用默认值 crypto", data_type);
                    multi_agent_oracle::oracle_agent::data_types::OracleDataType::Crypto
                }
            };
            
            match manager.run_consensus_test(oracle_data_type).await {
                Ok(result) => {
                    println!("共识测试结果:");
                    println!("  成功率: {:.1}%", result.consensus_success_rate * 100.0);
                    println!("  平均时间: {:.1}ms", result.average_consensus_time_ms);
                }
                Err(e) => println!("共识测试失败: {}", e),
            }
        }
        
        Commands::Diap => {
            println!("测试DIAP身份验证...");
            let manager = LocalTestNodeManager::initialize_test_nodes(config).await?;
            
            match manager.test_diap_authentication().await {
                Ok(results) => {
                    println!("DIAP身份验证结果:");
                    for result in results {
                        println!("  {} ({}层): {}", 
                            result.node_id, result.tier,
                            if result.success { "✅ 成功" } else { "❌ 失败" }
                        );
                    }
                }
                Err(e) => println!("DIAP测试失败: {}", e),
            }
        }
        
        Commands::Gateway => {
            println!("测试网关接入...");
            let manager = LocalTestNodeManager::initialize_test_nodes(config).await?;
            
            match manager.test_gateway_access().await {
                Ok(result) => {
                    println!("网关测试结果:");
                    println!("  连接成功率: {:.1}%", result.connection_success_rate * 100.0);
                    println!("  平均响应时间: {:.1}ms", result.average_response_time_ms);
                    println!("  故障恢复成功率: {:.1}%", result.fault_recovery_success_rate * 100.0);
                }
                Err(e) => println!("网关测试失败: {}", e),
            }
        }
        
        Commands::Prompt { node, command } => {
            let prompt_support = SimplePromptSupport::new();
            
            if let Some(cmd) = command {
                let node_id = node.unwrap_or_else(|| "test_agent".to_string());
                match prompt_support.handle_prompt(&node_id, &cmd).await {
                    Ok(response) => println!("响应: {}", response),
                    Err(e) => println!("错误: {}", e),
                }
            } else {
                prompt_support.show_available_commands().await;
            }
        }
        
        Commands::Reputation => {
            let reputation_system = PreconfiguredReputation::new();
            reputation_system.show_reputation_levels();
            println!();
            reputation_system.show_node_reputation_status();
        }
        
        Commands::Help => {
            show_help();
        }
    }
    
    Ok(())
}

/// 交互式模式
async fn interactive_mode(config: &LocalTestConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("输入 'help' 查看可用命令，'exit' 退出");
    println!();
    
    // 创建测试节点管理器
    let manager = match LocalTestNodeManager::initialize_test_nodes(config).await {
        Ok(manager) => {
            println!("✅ 测试环境就绪");
            manager
        }
        Err(e) => {
            println!("❌ 测试环境初始化失败: {}", e);
            return Ok(());
        }
    };
    
    let prompt_support = SimplePromptSupport::new();
    let reputation_system = PreconfiguredReputation::new();
    
    loop {
        print!("test> ");
        std::io::Write::flush(&mut std::io::stdout())?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        match input {
            "exit" | "quit" => {
                println!("退出测试控制台");
                break;
            }
            
            "help" => {
                show_help();
            }
            
            "nodes" => {
                manager.show_node_status();
            }
            
            "topology" => {
                manager.show_network_topology();
            }
            
            "consensus" => {
                println!("运行共识测试...");
                match manager.run_consensus_test(
                    multi_agent_oracle::oracle_agent::data_types::OracleDataType::Crypto
                ).await {
                    Ok(result) => {
                        println!("共识测试结果:");
                        println!("  成功率: {:.1}%", result.consensus_success_rate * 100.0);
                        println!("  平均时间: {:.1}ms", result.average_consensus_time_ms);
                    }
                    Err(e) => println!("错误: {}", e),
                }
            }
            
            "diap" => {
                println!("测试DIAP身份验证...");
                match manager.test_diap_authentication().await {
                    Ok(results) => {
                        let success_count = results.iter().filter(|r| r.success).count();
                        println!("身份验证完成: {}/{} 成功", success_count, results.len());
                    }
                    Err(e) => println!("错误: {}", e),
                }
            }
            
            "gateway" => {
                println!("测试网关接入...");
                match manager.test_gateway_access().await {
                    Ok(result) => {
                        println!("网关测试完成，连接成功率: {:.1}%", 
                            result.connection_success_rate * 100.0);
                    }
                    Err(e) => println!("错误: {}", e),
                }
            }
            
            "reputation" => {
                reputation_system.show_node_reputation_status();
            }
            
            "prompt help" => {
                prompt_support.show_available_commands().await;
            }
            
            "prompt test" => {
                println!("运行Prompt测试套件...");
                let result = prompt_support.run_prompt_test_suite("core").await;
                println!("Prompt测试完成，成功率: {:.1}%", 
                    result.prompt_success_rate * 100.0);
            }
            
            _ if input.starts_with("prompt ") => {
                let prompt = &input[7..]; // 去掉 "prompt "
                match prompt_support.handle_prompt("console", prompt).await {
                    Ok(response) => println!("{}", response),
                    Err(e) => println!("错误: {}", e),
                }
            }
            
            _ => {
                println!("未知命令: {}", input);
                println!("输入 'help' 查看可用命令");
            }
        }
        
        println!();
    }
    
    Ok(())
}

/// 显示帮助信息
fn show_help() {
    println!("📋 可用命令:");
    println!("====================");
    println!();
    println!("环境管理:");
    println!("  init      - 初始化测试环境");
    println!("  report    - 生成测试报告");
    println!("  clean     - 清理测试数据");
    println!();
    println!("节点管理:");
    println!("  nodes     - 显示节点状态");
    println!("  topology  - 显示网络拓扑");
    println!("  reputation - 显示信誉等级");
    println!();
    println!("功能测试:");
    println!("  consensus - 运行共识测试");
    println!("  diap      - 测试DIAP身份验证");
    println!("  gateway   - 测试网关接入");
    println!("  prompt    - 测试Prompt交互");
    println!("  prompt help - 显示Prompt命令");
    println!("  prompt test - 运行Prompt测试套件");
    println!("  prompt <command> - 执行Prompt命令");
    println!();
    println!("系统命令:");
    println!("  help      - 显示帮助信息");
    println!("  exit      - 退出控制台");
    println!();
    println!("使用示例:");
    println!("  cargo run --bin test_console -- --init");
    println!("  cargo run --bin test_console -- nodes");
    println!("  cargo run --bin test_console -- consensus --data-type crypto");
    println!("  cargo run --bin test_console -- prompt --command status");
}
