# 🚀 真实API测试完整指南

本文档介绍如何运行 `real_api_test.rs` 进行真实数据API测试。

---

## 📋 目录

1. [测试概述](#测试概述)
2. [支持的API](#支持的API)
3. [配置步骤](#配置步骤)
4. [运行测试](#运行测试)
5. [结果解读](#结果解读)
6. [故障排除](#故障排除)

---

## 🎯 测试概述

`real_api_test.rs` 演示了从真实数据源获取数据并执行完整的因果指纹分析流程：

```
真实API → 数据获取 → 因果指纹生成 → 谱特征提取 → ZK证明 → 验证
```

### 测试内容

✅ **加密货币价格** (BTC, ETH - 免费API)
✅ **股票价格** (AAPL - 需要API密钥)
✅ **天气数据** (London - 需要API密钥)
✅ **外汇汇率** (USD/EUR - 需要API密钥)

---

## 🔑 支持的API

### 1. CoinGecko (加密货币)
- **费用**: 免费
- **限制**: 公共API，无密钥要求
- **数据**: BTC, ETH, SOL等加密货币价格

### 2. Binance (加密货币)
- **费用**: 免费
- **限制**: 公共API，无密钥要求
- **数据**: 实时加密货币价格

### 3. Alpha Vantage (股票)
- **费用**: 免费版(5 API/分钟，500/天)
- **注册**: https://www.alphavantage.co/support/#api-key
- **限制**: 需要API密钥
- **数据**: 股票价格(AAPL, GOOGL等)

### 4. OpenWeatherMap (天气)
- **费用**: 免费版(60次调用/分钟，100万次/月)
- **注册**: https://home.openweathermap.org/users/sign_up
- **限制**: 需要API密钥
- **数据**: 天气数据(温度、湿度等)

### 5. ExchangeRate-API (外汇)
- **费用**: 免费版(1500次调用/月)
- **注册**: https://www.exchangerate-api.com/signup
- **限制**: 需要API密钥
- **数据**: 外汇汇率

---

## ⚙️ 配置步骤

### 方法一：PowerShell (推荐)

```powershell
# 立即设置环境变量（当前会话）
$env:ALPHA_VANTAGE_API_KEY="your_alpha_vantage_key"
$env:OPENWEATHER_API_KEY="your_openweather_key"
$env:EXCHANGERATE_API_KEY="your_exchangerate_key"

# 永久设置（需要重启终端）
[System.Environment]::SetEnvironmentVariable("ALPHA_VANTAGE_API_KEY", "your_key", "User")
[System.Environment]::SetEnvironmentVariable("OPENWEATHER_API_KEY", "your_key", "User")
[System.Environment]::SetEnvironmentVariable("EXCHANGERATE_API_KEY", "your_key", "User")
```

### 方法二：Windows批处理

双击运行 `examples/setup_api_keys.bat` 并按提示输入API密钥。

### 方法三：手动配置

在PowerShell中手动执行：

```batch
setx ALPHA_VANTAGE_API_KEY "your_alpha_vantage_key"
setx OPENWEATHER_API_KEY "your_openweather_key"
setx EXCHANGERATE_API_KEY "your_exchangerate_key"
```

**注意**：设置后需要**重启终端**才能生效。

---

## 🎬 运行测试

### 基本运行

```bash
# 运行真实API测试
cargo run --example real_api_test
```

### 带环境变量的运行（一次性）

```powershell
# PowerShell
$env:ALPHA_VANTAGE_API_KEY="your_key"; $env:OPENWEATHER_API_KEY="your_key"; cargo run --example real_api_test
```

### 预期输出

```
🔍 真实API数据获取测试
========================================

🔑 检查API密钥配置:
  ✅ CoinGecko: 公共API (免费)
  ✅ Binance: 公共API (免费)
  ⚠️  AlphaVantage: 使用demo模式 (功能受限)
  ⚠️  OpenWeatherMap: 使用demo模式 (功能受限)

✅ Oracle Agent初始化完成
✅ ZK证明生成器初始化完成

📊 Test 1: CryptoPrice { symbol: "bitcoin" }
--------------------------------------------------
🌐 正在获取数据: CryptoPrice { symbol: "bitcoin", vs_currency: "usd" }
  ✅ 数据获取成功
     原始值: 43250.75
     响应时间: 847ms
  ✅ 因果指纹生成完成
     特征维度: 3
  ✅ 谱特征提取完成
     谱半径: 34600.6000
     谱熵: 0.8234
  ✅ ZK证明: 验证通过

📊 Test 2: StockPrice { symbol: "AAPL" }
--------------------------------------------------
🌐 正在获取数据: StockPrice { symbol: "AAPL", exchange: "NYSE" }
  ✅ 数据获取成功
     原始值: 178.25
     响应时间: 1250ms
  ✅ 因果指纹生成完成
     特征维度: 3
  ✅ 谱特征提取完成
     谱半径: 160.4250
     谱熵: 0.7542
  ✅ ZK证明: 验证通过

... (更多测试)

========================================
📊 真实API测试总结报告
========================================

📈 成功率统计:
  总测试数: 5
  成功数: 5
  成功率: 100.0%
  平均响应时间: 945.6ms

📋 按数据类型统计:
  CryptoPrice: 2/2 (100.0%)
  StockPrice: 1/1 (100.0%)
  WeatherData: 1/1 (100.0%)
  ForexRate: 1/1 (100.0%)

🔐 ZK证明统计:
  验证通过: 5/5
  验证成功率: 100.0%

⚡ 性能评估:
  ✅ 优秀: API可用性高

💡 建议:
  ✅ 系统可以正常处理真实数据
  ✅ ZK证明机制工作正常
  ✅ 因果指纹分析有效
```

---

## 📊 结果解读

### 成功指标

- **成功率 > 80%**: 系统健康，API配置正确
- **平均响应时间 < 2000ms**: API响应良好
- **ZK验证成功率 = 100%**: 证明生成和验证机制正常

### 性能等级

| 等级 | 成功率 | 说明 |
|------|--------|------|
| 🟢 优秀 | ≥ 80% | API可用性高，配置正确 |
| 🟡 良好 | 60-80% | 基本可用，部分API需要检查 |
| 🔴 较差 | < 60% | 需要检查网络和API密钥配置 |

### 数据类型说明

- **CryptoPrice**: 加密货币价格 (免费API)
- **StockPrice**: 股票价格 (需要Alpha Vantage API密钥)
- **WeatherData**: 天气数据 (需要OpenWeatherMap API密钥)
- **ForexRate**: 外汇汇率 (需要ExchangeRate API密钥)

---

## 🔧 故障排除

### 问题1: API调用失败

**症状**: `❌ 数据获取失败: HTTP error: 401 Unauthorized`

**解决**:
```powershell
# 检查API密钥是否设置
echo $env:ALPHA_VANTAGE_API_KEY

# 如果为空，重新设置
$env:ALPHA_VANTAGE_API_KEY="your_key"
```

### 问题2: 响应超时

**症状**: `❌ 数据获取失败: timeout`

**解决**:
- 检查网络连接
- 增加超时时间：
```rust
// 在 config.rs 中
DataSource::new("AlphaVantage", "...", 0.7)
    .with_api_key(&api_key)
    .with_timeout(30)  // 增加到30秒
```

### 问题3: API限流

**症状**: `❌ 数据获取失败: rate limit exceeded`

**解决**:
1. 在测试循环中添加延迟：
```rust
tokio::time::sleep(Duration::from_secs(5)).await; // 等待5秒
```

2. 升级到付费API计划

### 问题4: 编译错误

**症状**: `error[E0432]: unresolved import multi_agent_oracle::...`

**解决**:
```bash
# 清理并重新构建
cargo clean
cargo build --example real_api_test
```

---

## 🎓 高级用法

### 自定义测试用例

在 `real_api_test.rs` 中修改 `test_cases` 向量：

```rust
let test_cases = vec![
    TestDataType::CryptoPrice { symbol: "bitcoin".to_string() },
    TestDataType::CryptoPrice { symbol: "ethereum".to_string() },
    TestDataType::StockPrice { symbol: "TSLA".to_string() },  // 特斯拉
    TestDataType::StockPrice { symbol: "MSFT".to_string() },  // 微软
    TestDataType::WeatherData { location: "Beijing".to_string() },
    TestDataType::WeatherData { location: "Tokyo".to_string() },
    TestDataType::ForexRate { from: "GBP".to_string(), to: "USD".to_string() },
];
```

### 添加新的数据源

在 `config.rs` 的 `with_real_apis()` 方法中添加：

```rust
// 添加新的数据源
let new_source = DataSource::new("NewAPI", "https://api.newapi.com/data", 0.8)
    .with_api_key(&api_key);
config.data_sources.push(new_source);
```

### 调整ZK证明参数

在测试文件中修改：

```rust
// 调整ZK证明生成参数
let zk_proof = zkp_generator.generate_fingerprint_proof(
    &fingerprint,
    &vec![data_point.value],
    &[1.0],  // 干预向量
    &[data_point.value],  // 响应向量
).await?;
```

---

## 📈 测试报告示例

完整的测试报告包含：

1. **成功率统计**: 总测试数、成功数、成功率
2. **性能指标**: 平均响应时间、最大/最小响应时间
3. **按类型统计**: 每种数据类型的成功率
4. **ZK证明统计**: 验证通过率
5. **谱特征分析**: 谱半径和谱熵分布
6. **性能评估**: 系统健康度评估
7. **优化建议**: 根据测试结果的建议

---

## 🔗 相关资源

- **API文档**:
  - Alpha Vantage: https://www.alphavantage.co/documentation/
  - OpenWeatherMap: https://openweathermap.org/api
  - ExchangeRate-API: https://www.exchangerate-api.com/docs

- **项目文档**:
  - `examples/TESTING_GUIDE.md` - 完整测试指南
  - `docs/API_DOCUMENTATION.md` - API文档

- **代码位置**:
  - 测试文件: `examples/real_api_test.rs`
  - 配置: `src/oracle_agent/config.rs`
  - 数据收集: `src/oracle_agent/data_collection.rs`

---

## ✅ 检查清单

在运行测试前，请确认：

- [ ] 已安装 Rust 和 Cargo
- [ ] 已克隆项目仓库
- [ ] 已配置至少一个API密钥（建议Alpha Vantage）
- [ ] 网络连接正常
- [ ] 项目已编译：`cargo build --example real_api_test`
- [ ] 了解API调用限制

---

## 🆘 需要帮助？

如果遇到问题：

1. 检查本指南的 [故障排除](#故障排除) 章节
2. 查看错误日志详细信息
3. 验证API密钥是否有效（在浏览器中测试API端点）
4. 使用 `RUST_LOG=debug` 获取详细日志：
   ```bash
   $env:RUST_LOG="debug"; cargo run --example real_api_test
   ```

---

**版本**: 1.0.0  
**最后更新**: 2025-01-12  
**作者**: MultiAgentOracle Team
