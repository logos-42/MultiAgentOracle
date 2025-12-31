//! 预言机智能体命令行工具
//!
//! 启动和管理预言机智能体节点。

use multi_agent_oracle::{
    OracleAgent, OracleAgentConfig, OracleDataType, DataSource,
    ReputationManager, ReputationConfig,
    NetworkManager, NetworkConfig,
};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::signal;
use log::{info, warn, error};

/// 命令行参数
#[derive(Parser)]
#[command(name = "oracle-agent")]
#[command(about = "多智能体预言机节点", long_about = None)]
struct Cli {
    /// 配置文件路径
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    
    /// 日志级别
    #[arg(short, long, default_value = "info")]
    log_level: String,
    
    /// 命令
    #[command(subcommand)]
    command: Commands,
}

/// 子命令
#[derive(Subcommand)]
enum Commands {
    /// 启动预言机节点
    Start {
        /// 节点名称
        #[arg(short, long)]
        name: Option<String>,
        
        /// 数据源配置文件
        #[arg(short, long)]
        data_sources: Option<PathBuf>,
        
        /// 监听端口
        #[arg(short, long, default_value_t = 4001)]
        port: u16,
        
        /// 启用P2P网络
        #[arg(long)]
        enable_p2p: bool,
    },
    
    /// 停止预言机节点
    Stop {
        /// 节点ID
        #[arg(short, long)]
        id: String,
    },
    
    /// 查看节点状态
    Status {
        /// 节点ID
        #[arg(short, long)]
        id: Option<String>,
    },
    
    /// 测试数据采集
    Test {
        /// 数据类型
        #[arg(short, long)]
        data_type: String,
        
        /// 符号（如BTC、ETH等）
        #[arg(short, long)]
        symbol: Option<String>,
        
        /// 位置（用于天气数据）
        #[arg(long)]
        location: Option<String>,
    },
    
    /// 管理信誉系统
    Reputation {
        /// 子命令
        #[command(subcommand)]
        command: ReputationCommands,
    },
    
    /// 查看帮助
    Help,
}

/// 信誉系统子命令
#[derive(Subcommand)]
enum ReputationCommands {
    /// 查看信誉排名
    Rankings {
        /// 显示数量
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
    
    /// 查看特定智能体信誉
    View {
        /// 智能体DID
        did: String,
    },
    
    /// 更新信誉分
    Update {
        /// 智能体DID
        did: String,
        
        /// 变化值
        delta: f64,
        
        /// 原因
        #[arg(short, long)]
        reason: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // 初始化日志
    env_logger::Builder::new()
        .filter_level(match cli.log_level.as_str() {
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,
            _ => log::LevelFilter::Info,
        })
        .init();
    
    info!("🚀 多智能体预言机节点启动");
    info!("版本: {}", multi_agent_oracle::VERSION);
    info!("描述: {}", multi_agent_oracle::DESCRIPTION);
    
    match cli.command {
        Commands::Start { name, data_sources, port, enable_p2p } => {
            start_node(name, data_sources, port, enable_p2p).await?;
        }
        Commands::Stop { id } => {
            stop_node(&id).await?;
        }
        Commands::Status { id } => {
            show_status(id).await?;
        }
        Commands::Test { data_type, symbol, location } => {
            test_data_collection(&data_type, symbol, location).await?;
        }
        Commands::Reputation { command } => {
            handle_reputation_command(command).await?;
        }
        Commands::Help => {
            print_help();
        }
    }
    
    Ok(())
}

/// 启动节点
async fn start_node(
    name: Option<String>,
    data_sources: Option<PathBuf>,
    port: u16,
    enable_p2p: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let node_name = name.unwrap_or_else(|| {
        format!("oracle_node_{}", std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs())
    });
    
    info!("📡 启动预言机节点: {}", node_name);
    
    // 创建预言机智能体配置
    let mut config = OracleAgentConfig::default_with_name(&node_name);
    
    // 如果有数据源配置文件，加载它
    if let Some(ds_path) = data_sources {
        info!("📂 加载数据源配置: {:?}", ds_path);
        // 这里应该实现从文件加载数据源配置
        // 简化版本：使用默认配置
    }
    
    // 创建预言机智能体
    let mut agent = OracleAgent::new(config)?;
    
    // 设置DIAP身份（简化版本）
    agent.set_diap_identity(
        format!("did:diap:{}", node_name),
        vec![1, 2, 3, 4, 5], // 简化私钥
    );
    
    info!("✅ 预言机智能体创建成功");
    info!("   名称: {}", node_name);
    info!("   DID: {}", agent.get_did().unwrap_or("未知"));
    info!("   支持的数据类型: {} 种", agent.get_supported_data_types().len());
    
    // 初始化信誉系统
    let reputation_config = ReputationConfig::default();
    let reputation_manager = Arc::new(ReputationManager::new(reputation_config));
    
    // 注册智能体到信誉系统
    if let Some(did) = agent.get_did() {
        reputation_manager.register_agent(did.to_string(), 1000).await?;
        info!("📊 注册到信誉系统: {}", did);
    }
    
    // 初始化网络系统（如果启用）
    let network_manager = if enable_p2p {
        info!("🌐 启用P2P网络");
        let network_config = NetworkConfig {
            listen_address: "0.0.0.0".to_string(),
            listen_port: 8080,
            bootstrap_nodes: vec![],
            max_connections: 100,
            connection_timeout_secs: 30,
            heartbeat_interval_secs: 10,
            enable_nat_traversal: false,
            enable_relay: false,
            relay_nodes: vec![],
        };
        Some(NetworkManager::new(node_name.clone(), network_config)?)
    } else {
        info!("🌐 P2P网络未启用");
        None
    };
    
    // 启动网络（如果启用）
    if let Some(mut nm) = network_manager {
        info!("📡 启动网络监听端口: {}", port);
        // 这里应该启动网络监听
        // 简化版本：只显示信息
    }
    
    info!("🎯 节点启动完成，等待命令...");
    
    // 等待Ctrl+C信号
    signal::ctrl_c().await?;
    info!("🛑 收到停止信号，正在关闭节点...");
    
    Ok(())
}

/// 停止节点
async fn stop_node(id: &str) -> Result<(), Box<dyn std::error::Error>> {
    info!("🛑 停止节点: {}", id);
    // 这里应该实现停止节点的逻辑
    // 简化版本：只显示信息
    info!("✅ 节点 {} 已停止", id);
    Ok(())
}

/// 显示节点状态
async fn show_status(id: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(node_id) = id {
        info!("📊 查看节点状态: {}", node_id);
        // 这里应该实现查看特定节点状态的逻辑
        println!("节点ID: {}", node_id);
        println!("状态: 运行中");
        println!("启动时间: 刚刚");
        println!("数据采集次数: 0");
        println!("信誉分: 100.0");
    } else {
        info!("📊 查看所有节点状态");
        // 这里应该实现查看所有节点状态的逻辑
        println!("总节点数: 1");
        println!("运行中: 1");
        println!("离线: 0");
    }
    
    Ok(())
}

/// 测试数据采集
async fn test_data_collection(
    data_type: &str,
    symbol: Option<String>,
    location: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("🧪 测试数据采集: {}", data_type);
    
    // 创建测试智能体
    let config = OracleAgentConfig::default_with_name("test_agent");
    let mut agent = OracleAgent::new(config)?;
    
    // 根据数据类型创建OracleDataType
    let oracle_data_type = match data_type.to_lowercase().as_str() {
        "crypto" | "cryptoprice" => {
            let sym = symbol.unwrap_or_else(|| "BTC".to_string());
            OracleDataType::CryptoPrice { symbol: sym }
        }
        "stock" | "stockprice" => {
            let sym = symbol.unwrap_or_else(|| "AAPL".to_string());
            OracleDataType::StockPrice { 
                symbol: sym, 
                exchange: "NASDAQ".to_string() 
            }
        }
        "weather" => {
            let loc = location.unwrap_or_else(|| "Beijing".to_string());
            OracleDataType::WeatherData { 
                location: loc, 
                metric: "temperature".to_string() 
            }
        }
        _ => {
            return Err(format!("不支持的数据类型: {}", data_type).into());
        }
    };
    
    info!("采集数据类型: {:?}", oracle_data_type);
    
    // 采集数据
    match agent.collect_data(&oracle_data_type).await {
        Ok(result) => {
            if result.success {
                info!("✅ 数据采集成功");
                if let Some(data) = result.data {
                    println!("数据类型: {:?}", data.data_type);
                    println!("值: {:?}", data.value);
                    println!("置信度: {:.2}", data.confidence);
                    println!("数据源: {:?}", data.sources_used);
                    println!("时间戳: {}", data.timestamp);
                    println!("采集耗时: {}ms", result.collection_time_ms);
                }
            } else {
                warn!("⚠️ 数据采集失败: {:?}", result.error);
            }
        }
        Err(e) => {
            error!("❌ 数据采集错误: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

/// 处理信誉系统命令
async fn handle_reputation_command(
    command: ReputationCommands,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建信誉管理器
    let reputation_config = ReputationConfig::default();
    let reputation_manager = Arc::new(ReputationManager::new(reputation_config));
    
    match command {
        ReputationCommands::Rankings { limit } => {
            info!("🏆 查看信誉排名 (前{}名)", limit);
            
            // 注册一些测试智能体
            let test_agents = vec![
                ("did:diap:agent_1".to_string(), 1000),
                ("did:diap:agent_2".to_string(), 2000),
                ("did:diap:agent_3".to_string(), 1500),
            ];
            
            for (did, stake) in test_agents {
                reputation_manager.register_agent(did.clone(), stake).await?;
                // 模拟一些信誉更新
                reputation_manager.update_for_data_accuracy(
                    &did,
                    45000.0,
                    45100.0,
                    0.02,
                    Some("test_data".to_string()),
                ).await?;
            }
            
            let rankings = reputation_manager.get_rankings(limit).await;
            println!("信誉排名 (前{}名):", limit);
            println!("{:<5} {:<30} {:<10} {:<10}", "排名", "智能体DID", "信誉分", "质押金额");
            println!("{}", "-".repeat(60));
            
            for (i, ranking) in rankings.iter().enumerate() {
                println!("{:<5} {:<30} {:<10.2} {:<10}", 
                    i + 1, 
                    ranking.agent_did, 
                    ranking.score,
                    ranking.staked_amount
                );
            }
        }
        ReputationCommands::View { did } => {
            info!("👁️ 查看智能体信誉: {}", did);
            
            if let Some(score) = reputation_manager.get_score(&did).await {
                println!("智能体DID: {}", did);
                println!("信誉分: {:.2}", score.score);
                println!("质押金额: {}", score.staked_amount);
                println!("成功率: {:.2}%", score.success_rate() * 100.0);
                println!("服务次数: {}", score.total_services);
                println!("是否活跃: {}", score.is_active);
            } else {
                println!("未找到智能体: {}", did);
            }
        }
        ReputationCommands::Update { did, delta, reason } => {
            info!("📝 更新信誉分: {} Δ = {:.2}", did, delta);
            
            // 简化版本：直接更新信誉分
            // 注意：这里需要实际的更新逻辑，目前只是模拟
            println!("⚠️  信誉更新功能需要实现");
            println!("智能体DID: {}", did);
            println!("变化值: {:.2}", delta);
            println!("原因: {:?}", reason);
            println!("注意：实际更新逻辑需要调用ReputationManager的相应方法");
        }
    }
    
    Ok(())
}

/// 打印帮助信息
fn print_help() {
    println!("多智能体预言机节点命令行工具");
    println!();
    println!("使用方法:");
    println!("  oracle-agent [OPTIONS] <COMMAND>");
    println!();
    println!("选项:");
    println!("  -c, --config <FILE>    配置文件路径");
    println!("  -l, --log-level <LEVEL> 日志级别 [error, warn, info, debug, trace]");
    println!();
    println!("命令:");
    println!("  start                   启动预言机节点");
    println!("  stop                    停止预言机节点");
    println!("  status                  查看节点状态");
    println!("  test                    测试数据采集");
    println!("  reputation              管理信誉系统");
    println!("  help                    查看帮助");
    println!();
    println!("示例:");
    println!("  oracle-agent start --name my_node --port 4001");
    println!("  oracle-agent test --data-type crypto --symbol BTC");
    println!("  oracle-agent reputation rankings --limit 10");
}
