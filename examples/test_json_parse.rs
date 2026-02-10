//! 测试 JSON 解析逻辑

fn main() {
    // 模拟 Minimax 返回的 markdown 格式响应
    let response = r#"```json
{
   "nodes": [
     {"id": "1", "name": "市场情绪", "node_type": "variable", "importance": 0.8},
     {"id": "2", "name": "财报数据", "node_type": "variable", "importance": 0.9},
     {"id": "3", "name": "股价", "node_type": "variable", "importance": 1.0}
   ],
   "edges": [
     {"id": "e1", "source": "1", "target": "3", "weight": 0.6, "edge_type": "positive"},
     {"id": "e2", "source": "2", "target": "3", "weight": 0.8, "edge_type": "positive"}
   ],
   "paths": [
     {"id": "p1", "nodes": ["1", "3"], "strength": 0.6, "path_type": "direct"},
     {"id": "p2", "nodes": ["2", "3"], "strength": 0.8, "path_type": "direct"}
   ],
   "reasoning": "市场情绪可能会影响投资者对公司未来发展的预期，从而影响股价。财报数据作为公司经营状况的直接体现，对股价有重大影响。两者都是影响股价的关键变量。",
   "confidence": 0.9
}
```"#;

    println!("🔍 测试 JSON 解析\n");
    println!("原始响应长度: {} 字符\n", response.len());

    // 测试策略2：查找 ```json 代码块
    if let Some(code_start) = response.find("```json") {
        let actual_start = code_start + 7;
        println!("✅ 找到 ```json 标记，位置: {}", code_start);

        if let Some(code_end) = response[actual_start..].find("```") {
            let json_content = response[actual_start..actual_start + code_end].trim();
            println!("✅ 找到结束标记，JSON内容长度: {} 字符\n", json_content.len());

            // 尝试解析 JSON
            match serde_json::from_str::<serde_json::Value>(json_content) {
                Ok(json) => {
                    println!("✅ JSON 解析成功！");
                    println!("   - nodes: {}", json["nodes"].as_array().map(|a| a.len()).unwrap_or(0));
                    println!("   - edges: {}", json["edges"].as_array().map(|a| a.len()).unwrap_or(0));
                    println!("   - paths: {}", json["paths"].as_array().map(|a| a.len()).unwrap_or(0));
                }
                Err(e) => {
                    println!("❌ JSON 解析失败: {}", e);
                    println!("   前200字符: {}", &json_content[..json_content.len().min(200)]);
                }
            }
        } else {
            println!("❌ 未找到结束标记 ```");
        }
    } else {
        println!("❌ 未找到 ```json 标记");
    }
}
