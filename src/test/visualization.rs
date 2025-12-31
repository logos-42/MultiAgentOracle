//! 测试结果可视化模块
//! 
//! 提供分层架构测试结果的可视化输出

use crate::test::{TestResults, NetworkTestResult, ConsensusTestResult, DiapTestResult, GatewayTestResult, PromptTestResult};
use std::collections::HashMap;

/// 可视化测试结果
pub fn visualize_test_results(results: &TestResults) {
    println!("📊 分层架构测试结果");
    println!("====================");
    println!();
    
    // 网络拓扑图
    visualize_network_results(&results.network_test);
    println!();
    
    // 共识性能
    visualize_consensus_results(&results.consensus_test);
    println!();
    
    // 身份验证统计
    visualize_diap_results(&results.diap_test);
    println!();
    
    // 网关负载分布
    visualize_gateway_results(&results.gateway_test);
    println!();
    
    // Prompt测试结果
    visualize_prompt_results(&results.prompt_test);
    println!();
    
    // 总体评分
    calculate_overall_score(results);
}

/// 可视化网络测试结果
fn visualize_network_results(results: &NetworkTestResult) {
    println!("🌐 网络测试结果");
    println!("--------------");
    
    println!("连接成功率: {:.1}%", results.connection_success_rate * 100.0);
    println!("平均延迟: {:.1}ms", results.average_latency_ms);
    
    if !results.tier_connection_stats.is_empty() {
        println!("\n层级连接统计:");
        for (tier, stats) in &results.tier_connection_stats {
            let success_rate = if stats.successful_connections + stats.failed_connections > 0 {
                stats.successful_connections as f64 / 
                (stats.successful_connections + stats.failed_connections) as f64 * 100.0
            } else {
                0.0
            };
            
            println!("  {}层:", tier);
            println!("    成功连接: {}，失败: {}", stats.successful_connections, stats.failed_connections);
            println!("    成功率: {:.1}%", success_rate);
            println!("    平均连接时间: {:.1}ms", stats.average_connection_time_ms);
        }
    }
    
    if !results.errors.is_empty() {
        println!("\n错误列表:");
        for error in &results.errors {
            println!("  ❌ {}", error);
        }
    }
}

/// 可视化共识测试结果
fn visualize_consensus_results(results: &ConsensusTestResult) {
    println!("🤝 共识测试结果");
    println!("--------------");
    
    println!("共识成功率: {:.1}%", results.consensus_success_rate * 100.0);
    println!("平均共识时间: {:.1}ms", results.average_consensus_time_ms);
    
    if !results.tier_consensus_stats.is_empty() {
        println!("\n层级共识统计:");
        for (tier, stats) in &results.tier_consensus_stats {
            println!("  {}层:", tier);
            println!("    参与率: {:.1}%", stats.participation_rate * 100.0);
            println!("    平均投票权重: {:.2}", stats.average_voting_weight);
            println!("    共识准确率: {:.1}%", stats.consensus_accuracy * 100.0);
        }
    }
    
    println!("\n权重影响分析:");
    println!("  信誉权重相关性: {:.3}", results.weight_influence_analysis.reputation_weight_correlation);
    println!("  质押权重相关性: {:.3}", results.weight_influence_analysis.stake_weight_correlation);
    println!("  层级权重相关性: {:.3}", results.weight_influence_analysis.tier_weight_correlation);
}

/// 可视化DIAP测试结果
fn visualize_diap_results(results: &DiapTestResult) {
    println!("🔐 DIAP身份测试结果");
    println!("-----------------");
    
    println!("身份注册成功率: {:.1}%", results.identity_registration_success_rate * 100.0);
    println!("验证成功率: {:.1}%", results.verification_success_rate * 100.0);
    println!("平均注册时间: {:.1}ms", results.average_registration_time_ms);
    println!("平均验证时间: {:.1}ms", results.average_verification_time_ms);
    
    if !results.tier_authentication_stats.is_empty() {
        println!("\n层级认证统计:");
        for (tier, stats) in &results.tier_authentication_stats {
            println!("  {}层:", tier);
            println!("    认证成功率: {:.1}%", stats.auth_success_rate * 100.0);
            println!("    平均认证时间: {:.1}ms", stats.average_auth_time_ms);
            println!("    跨层级认证成功率: {:.1}%", stats.cross_tier_auth_success_rate * 100.0);
        }
    }
}

/// 可视化网关测试结果
fn visualize_gateway_results(results: &GatewayTestResult) {
    println!("🚪 网关测试结果");
    println!("-------------");
    
    println!("连接成功率: {:.1}%", results.connection_success_rate * 100.0);
    println!("平均响应时间: {:.1}ms", results.average_response_time_ms);
    println!("故障恢复成功率: {:.1}%", results.fault_recovery_success_rate * 100.0);
    
    if !results.gateway_load_distribution.is_empty() {
        println!("\n网关负载分布:");
        for (gateway_id, stats) in &results.gateway_load_distribution {
            println!("  {} ({}):", gateway_id, stats.gateway_type);
            println!("    活跃连接: {}", stats.active_connections);
            println!("    总请求数: {}", stats.total_requests);
            println!("    平均负载: {:.1}%", stats.average_load_percentage);
            println!("    错误率: {:.1}%", stats.error_rate * 100.0);
        }
    }
}

/// 可视化Prompt测试结果
fn visualize_prompt_results(results: &PromptTestResult) {
    println!("🤖 Prompt测试结果");
    println!("---------------");
    
    println!("Prompt成功率: {:.1}%", results.prompt_success_rate * 100.0);
    println!("平均响应时间: {:.1}ms", results.average_response_time_ms);
    
    if !results.command_coverage.is_empty() {
        println!("\n命令覆盖统计:");
        for (command, stats) in &results.command_coverage {
            let total = stats.success_count + stats.failure_count;
            let success_rate = if total > 0 {
                stats.success_count as f64 / total as f64 * 100.0
            } else {
                0.0
            };
            
            println!("  {}:", command);
            println!("    成功: {}，失败: {}", stats.success_count, stats.failure_count);
            println!("    成功率: {:.1}%", success_rate);
            println!("    平均响应时间: {:.1}ms", stats.average_response_time_ms);
        }
    }
    
    if !results.tier_response_stats.is_empty() {
        println!("\n层级响应统计:");
        for (tier, stats) in &results.tier_response_stats {
            println!("  {}层:", tier);
            println!("    响应成功率: {:.1}%", stats.response_success_rate * 100.0);
            println!("    平均响应质量: {:.1}/10", stats.average_response_quality);
        }
    }
}

/// 计算总体评分
fn calculate_overall_score(results: &TestResults) {
    println!("⭐ 总体评分");
    println!("----------");
    
    let weights = HashMap::from([
        ("network", 0.25),
        ("consensus", 0.30),
        ("diap", 0.20),
        ("gateway", 0.15),
        ("prompt", 0.10),
    ]);
    
    let mut weighted_score = 0.0;
    let mut component_scores = Vec::new();
    
    // 网络组件评分
    let network_score = results.network_test.connection_success_rate * 0.7 + 
                       (1.0 - results.network_test.average_latency_ms / 1000.0).max(0.0) * 0.3;
    weighted_score += network_score * weights["network"];
    component_scores.push(("网络", network_score));
    
    // 共识组件评分
    let consensus_score = results.consensus_test.consensus_success_rate * 0.6 +
                         (1.0 - results.consensus_test.average_consensus_time_ms / 500.0).max(0.0) * 0.4;
    weighted_score += consensus_score * weights["consensus"];
    component_scores.push(("共识", consensus_score));
    
    // DIAP组件评分
    let diap_score = results.diap_test.verification_success_rate * 0.5 +
                    results.diap_test.identity_registration_success_rate * 0.5;
    weighted_score += diap_score * weights["diap"];
    component_scores.push(("身份认证", diap_score));
    
    // 网关组件评分
    let gateway_score = results.gateway_test.connection_success_rate * 0.4 +
                       results.gateway_test.fault_recovery_success_rate * 0.3 +
                       (1.0 - results.gateway_test.average_response_time_ms / 200.0).max(0.0) * 0.3;
    weighted_score += gateway_score * weights["gateway"];
    component_scores.push(("网关", gateway_score));
    
    // Prompt组件评分
    let prompt_score = results.prompt_test.prompt_success_rate;
    weighted_score += prompt_score * weights["prompt"];
    component_scores.push(("交互", prompt_score));
    
    // 显示组件评分
    println!("组件评分:");
    for (component, score) in component_scores {
        let stars = "★".repeat((score * 5.0).round() as usize);
        let empty_stars = "☆".repeat(5 - stars.len());
        println!("  {}: {:.1}/5.0 {}{}", component, score * 5.0, stars, empty_stars);
    }
    
    println!();
    
    // 总体评分
    let overall_score = weighted_score * 100.0;
    let grade = match overall_score {
        s if s >= 90.0 => "A+ (优秀)",
        s if s >= 80.0 => "A (良好)",
        s if s >= 70.0 => "B (中等)",
        s if s >= 60.0 => "C (及格)",
        _ => "D (需要改进)",
    };
    
    println!("总体评分: {:.1}/100.0", overall_score);
    println!("等级: {}", grade);
    
    // 进度条显示
    let progress_width = 50;
    let filled = (overall_score / 100.0 * progress_width as f64).round() as usize;
    let empty = progress_width - filled;
    
    print!("进度: [");
    for _ in 0..filled {
        print!("█");
    }
    for _ in 0..empty {
        print!("░");
    }
    println!("] {:.1}%", overall_score);
}

/// 打印网络拓扑图
pub fn print_topology_graph() {
    println!("🌐 分层网络拓扑图");
    println!("====================");
    println!();
    println!("        ┌─────────┐");
    println!("        │ 核心层  │");
    println!("        │ (2节点) │");
    println!("        └────┬────┘");
    println!("             │");
    println!("        ┌────▼────┐");
    println!("        │ 验证层  │");
    println!("        │ (3节点) │");
    println!("        └────┬────┘");
    println!("             │");
    println!("        ┌────▼────┐");
    println!("        │ 数据层  │");
    println!("        │ (5节点) │");
    println!("        └─────────┘");
    println!();
    println!("网关接入:");
    println!("  ├─ 轻节点网关 (2个)");
    println!("  └─ 移动网关 (1个)");
}

/// 打印共识统计
pub fn print_consensus_stats() {
    println!("🤝 共识性能统计");
    println!("====================");
    println!();
    println!("层级权重分布:");
    println!("  核心层: 2.0x 投票权重");
    println!("  验证层: 1.5x 投票权重");
    println!("  数据层: 1.0x 投票权重");
    println!();
    println!("共识阈值:");
    println!("  核心层: ≥67% 同意");
    println!("  验证层: ≥75% 同意");
    println!("  数据提交: ≥60% 有效");
}

/// 打印身份验证统计
pub fn print_auth_stats() {
    println!("🔐 身份验证统计");
    println!("====================");
    println!();
    println!("DIAP SDK集成:");
    println!("  端点: http://localhost:8080/diap");
    println!("  模拟模式: 启用");
    println!();
    println!("验证流程:");
    println!("  1. 节点注册身份");
    println!("  2. DIAP验证身份");
    println!("  3. 分配初始层级");
    println!("  4. 跨层级身份验证");
}

/// 打印网关负载
pub fn print_gateway_load() {
    println!("🚪 网关负载分布");
    println!("====================");
    println!();
    println!("网关类型和容量:");
    println!("  轻节点网关: 最大100连接");
    println!("  移动网关: 最大50连接 (移动优化)");
    println!();
    println!("负载均衡策略:");
    println!("  1. 基于地理位置的连接分配");
    println!("  2. 基于层级的优先级路由");
    println!("  3. 动态故障转移机制");
}
