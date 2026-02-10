//! 测试 Minimax API 的 JSON 输出

use multi_agent_oracle::oracle_agent::{LlmClient, LlmClientConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    println!("🚀 测试 Minimax JSON 输出\n");

    // 创建 Minimax 客户端
    let config = LlmClientConfig::minimax("abab5.5-chat")
        .with_temperature(0.7)
        .with_max_tokens(2500);
    let client = LlmClient::new(config)?;

    // 测试因果图生成prompt
    let prompt = r#"请分析以下场景的因果关系，以JSON格式返回。

场景：预测某科技公司的股价。

请以以下JSON格式返回因果关系：
```json
{
  "nodes": [
    {"id": "1", "name": "市场情绪", "node_type": "variable", "importance": 0.8},
    {"id": "2", "name": "财报数据", "node_type": "variable", "importance": 0.9}
  ],
  "edges": [
    {"id": "e1", "source": "1", "target": "2", "weight": 0.7, "edge_type": "positive"}
  ],
  "paths": [
    {"id": "p1", "nodes": ["1", "2"], "strength": 0.75, "path_type": "direct"}
  ],
  "reasoning": "简短解释",
  "confidence": 0.85
}
```"#;

    println!("📤 发送请求...");
    println!("   Max tokens: 2500\n");

    let response = client.generate_response(prompt).await?;

    println!("✅ 响应成功！");
    println!("   响应时间: {}ms", response.response_time_ms);
    println!("   响应长度: {} 字符\n", response.text.len());
    println!("📝 完整响应:");
    println!("   ---");
    for (i, line) in response.text.lines().enumerate() {
        println!("   {:3}: {}", i + 1, line);
    }
    println!("   ---");

    Ok(())
}
