//! 测试 Minimax API 调用

use multi_agent_oracle::oracle_agent::LlmClient;
use multi_agent_oracle::oracle_agent::LlmClientConfig;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 加载环境变量
    dotenv::dotenv().ok();

    println!("🚀 测试 Minimax API 调用\n");

    // 创建 Minimax 客户端
    let config = LlmClientConfig::minimax("abab5.5-chat");
    let client = LlmClient::new(config)?;

    println!("✅ Minimax 客户端创建成功");
    println!("📝 模型: abab5.5-chat\n");

    // 测试简单调用
    let test_prompt = "请简要回答：什么是区块链？";

    println!("📤 发送测试请求...");
    println!("   Prompt: {}\n", test_prompt);

    let response = client.generate_response(test_prompt).await?;

    println!("✅ Minimax API 响应成功！");
    println!("   响应时间: {}ms", response.response_time_ms);
    println!("   响应长度: {} 字符", response.text.len());
    println!("   响应内容:\n");
    println!("   ---");
    println!("   {}", response.text);
    println!("   ---");

    Ok(())
}
