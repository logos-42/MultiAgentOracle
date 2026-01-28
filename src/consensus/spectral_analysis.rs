//! 谱分析模块
//!
//! 实现用于因果指纹验证的谱分析。
//! 使用矩阵特征值分解从智能体响应中提取逻辑"骨架"。
//!
//! 核心概念：
//! - 谱特征代表智能体的"逻辑骨架"
//! - 不同模型（GPT-4、Claude）具有不同的谱分布
//! - 同质性检测用于识别智能体是否共享相同的底层模型

#![allow(missing_docs)]

use serde::{Deserialize, Serialize};

/// 从智能体响应中提取的谱特征
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpectralFeatures {
    pub eigenvalues: Vec<f64>,           // 主特征值
    pub spectral_radius: f64,            // 最大特征值幅度
    pub trace: f64,                      // 特征值之和
    pub rank: usize,                     // 有效秩
    pub entropy: f64,                    // 谱熵
    pub timestamp: u64,
}

/// 谱分析配置
#[derive(Debug, Clone)]
pub struct SpectralConfig {
    pub num_eigenvalues: usize,          // 提取的特征值数量
    pub entropy_threshold: f64,          // 有效模型的最小熵值
    pub homogeneity_threshold: f64,      // 模型检测的相似度阈值
    pub min_samples_for_analysis: usize, // 分析所需的最小样本数
}

impl Default for SpectralConfig {
    fn default() -> Self {
        Self {
            num_eigenvalues: 8,
            entropy_threshold: 0.5,
            homogeneity_threshold: 0.95,
            min_samples_for_analysis: 3,
        }
    }
}

/// 从响应矩阵中提取谱特征
/// 
/// 在实际实现中，这会使用 nalgebra 或类似的库进行 SVD。
/// 现在我们使用简化的方法来捕捉其核心思想。
#[allow(dead_code)]
pub fn extract_spectral_features(responses: &[Vec<f64>]) -> SpectralFeatures {
    let num_eigenvalues = 8;
    
    if responses.len() < 3 || responses.is_empty() || responses[0].is_empty() {
        return SpectralFeatures {
            eigenvalues: vec![0.0; num_eigenvalues],
            spectral_radius: 0.0,
            trace: 0.0,
            rank: 0,
            entropy: 0.0,
            timestamp: 0,
        };
    }
    
    let n = responses.len();
    let m = responses[0].len();
    let _dim = m.min(num_eigenvalues);
    
    // 计算每个维度的均值
    let mut means = vec![0.0; m];
    for response in responses {
        for (j, val) in response.iter().enumerate() {
            means[j] += val;
        }
    }
    for mean in &mut means {
        *mean /= n as f64;
    }
    
    // 计算协方差矩阵
    let mut cov: Vec<Vec<f64>> = vec![vec![0.0; m]; m];
    for response in responses {
        for i in 0..m {
            for j in 0..m {
                let di = response[i] - means[i];
                let dj = response[j] - means[j];
                cov[i][j] += di * dj;
            }
        }
    }
    for i in 0..m {
        for j in 0..m {
            cov[i][j] /= (n - 1) as f64;
        }
    }
    
    // 简化的特征值近似（使用幂迭代法计算主特征值）
    let eigenvalues = approximate_eigenvalues(&cov, num_eigenvalues);
    
    // 计算谱属性
    let spectral_radius = eigenvalues.iter().fold(0.0f64, |max, &e| max.max(e.abs()));
    let trace: f64 = eigenvalues.iter().sum();
    
    // 计算有效秩（超过阈值的特征值数量）
    let threshold = spectral_radius * 0.01;
    let rank = eigenvalues.iter().filter(|&&e| e.abs() > threshold).count();
    
    // 计算谱熵
    let total: f64 = eigenvalues.iter().map(|e| e.abs()).sum();
    let entropy = if total > 0.0 {
        -eigenvalues.iter()
            .map(|e| {
                let p = e.abs() / total;
                if p > 0.0 { p * p.log2() } else { 0.0 }
            })
            .sum::<f64>()
    } else {
        0.0
    };
    
    SpectralFeatures {
        eigenvalues,
        spectral_radius,
        trace,
        rank,
        entropy,
        timestamp: 0,
    }
}

/// 使用幂迭代和压缩方法近似计算特征值
#[allow(dead_code)]
fn approximate_eigenvalues(matrix: &[Vec<f64>], num: usize) -> Vec<f64> {
    let n = matrix.len();
    if n == 0 {
        return vec![0.0; num];
    }
    
    let m = matrix[0].len();
    let dim = m.min(num);
    let mut eigenvalues = Vec::with_capacity(num);
    
    // 创建可变副本用于压缩
    let mut working_matrix: Vec<Vec<f64>> = matrix.iter().map(|row| row.clone()).collect();
    
    for _ in 0..dim {
        if working_matrix.is_empty() || working_matrix[0].is_empty() {
            eigenvalues.push(0.0);
            continue;
        }
        
        // 幂迭代
        let mut v: Vec<f64> = (0..working_matrix.len()).map(|_| rand::random::<f64>()).collect();
        let norm: f64 = v.iter().map(|x| x * x).sum::<f64>().sqrt();
        for val in &mut v {
            *val /= norm;
        }
        
        for _iter in 0..100 {
            // 矩阵-向量乘法
            let mut new_v = vec![0.0; working_matrix.len()];
            for i in 0..working_matrix.len() {
                for j in 0..working_matrix[i].len() {
                    new_v[i] += working_matrix[i][j] * v[j];
                }
            }
            
            // 归一化
            let new_norm: f64 = new_v.iter().map(|x| x * x).sum::<f64>().sqrt();
            if new_norm > 0.0 {
                for val in &mut new_v {
                    *val /= new_norm;
                }
            }
            
            // 检查收敛
            let diff: f64 = v.iter().zip(new_v.iter())
                .map(|(a, b)| (a - b).powi(2))
                .sum::<f64>()
                .sqrt();
            
            v = new_v;
            if diff < 1e-10 {
                break;
            }
        }
        
        // 瑞利商计算特征值
        let av: Vec<f64> = (0..working_matrix.len())
            .map(|i| working_matrix[i].iter().zip(v.iter()).map(|(a, b)| a * b).sum::<f64>())
            .collect();
        let vv: f64 = v.iter().zip(v.iter()).map(|(a, b)| a * b).sum();
        let eigenvalue = if vv != 0.0 { av.iter().zip(v.iter()).map(|(a, b)| a * b).sum::<f64>() / vv } else { 0.0 };
        
        eigenvalues.push(eigenvalue.abs());
        
        // 压缩（简化版）- 由于矩阵可变，现在可以工作
        for i in 0..working_matrix.len() {
            for j in 0..working_matrix[i].len() {
                working_matrix[i][j] -= eigenvalue * v[i] * v[j];
            }
        }
    }
    
    // 如有需要，填充
    while eigenvalues.len() < num {
        eigenvalues.push(0.0);
    }
    
    eigenvalues
}

/// 计算两个智能体之间的谱距离
pub fn spectral_distance(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() {
        return f64::MAX;
    }
    
    let diff: f64 = a.iter().zip(b.iter())
        .map(|(x, y)| (x - y).powi(2))
        .sum::<f64>()
        .sqrt();
    
    diff
}

/// 检查两个智能体是否具有同质谱特征（相同模型）
pub fn is_homogeneous(a: &[f64], b: &[f64], threshold: f64) -> bool {
    let similarity = spectral_similarity(a, b);
    similarity > threshold
}

/// 计算谱相似度（特征值的余弦相似度）
pub fn spectral_similarity(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    
    let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
    let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
    
    if norm_a == 0.0 || norm_b == 0.0 {
        0.0
    } else {
        dot_product / (norm_a * norm_b)
    }
}

/// 检测智能体是否使用相同的底层模型
#[allow(dead_code)]
pub fn detect_model_homogeneity(features: &[SpectralFeatures], threshold: f64) -> Vec<(usize, usize)> {
    let mut homogeneous_pairs = Vec::new();
    
    for i in 0..features.len() {
        for j in (i + 1)..features.len() {
            if is_homogeneous(&features[i].eigenvalues, &features[j].eigenvalues, threshold) {
                homogeneous_pairs.push((i, j));
            }
        }
    }
    
    homogeneous_pairs
}

/// 计算当前指纹与历史指纹之间的一致性分数
pub fn fingerprint_consistency_score(
    current: &SpectralFeatures,
    historical: &[SpectralFeatures],
) -> f64 {
    if historical.is_empty() {
        return 1.0; // 无历史记录可供比较
    }
    
    let avg_historical: Vec<f64> = (0..current.eigenvalues.len())
        .map(|i| {
            let sum: f64 = historical.iter().map(|f| f.eigenvalues[i].abs()).sum();
            sum / historical.len() as f64
        })
        .collect();
    
    spectral_similarity(&current.eigenvalues, &avg_historical)
}

/// 检查谱特征是否表示有效模型（非幻觉）
#[allow(dead_code)]
pub fn is_valid_spectral(features: &SpectralFeatures, min_entropy: f64) -> bool {
    // 有效模型应具有非平凡的谱结构
    features.spectral_radius > 0.0 && 
    features.entropy >= min_entropy &&
    features.rank >= 2
}

/// 提取谱特征作为 i64 数组用于链上存储
pub fn features_to_i64(features: &SpectralFeatures, target_len: usize) -> [i64; 16] {
    let mut result = [0i64; 16];
    let scale = 1_000_000.0; // 定点转换的缩放因子
    
    for (i, eigenvalue) in features.eigenvalues.iter().enumerate().take(target_len) {
        result[i] = (eigenvalue.abs() * scale) as i64;
    }
    
    // 添加派生特征
    if target_len > 8 {
        result[8] = (features.spectral_radius * scale) as i64;
        result[9] = (features.trace * scale) as i64;
        result[10] = features.rank as i64;
        result[11] = ((features.entropy * 100.0) as i64).clamp(0, 100);
    }
    
    result
}

/// 将 i64 数组转换回谱特征
pub fn i64_to_features(data: &[i64; 16], num_eigenvalues: usize) -> SpectralFeatures {
    let scale = 1_000_000.0;
    let mut eigenvalues = Vec::with_capacity(num_eigenvalues);
    
    for i in 0..num_eigenvalues.min(8) {
        eigenvalues.push(data[i] as f64 / scale);
    }
    
    SpectralFeatures {
        eigenvalues,
        spectral_radius: data[8] as f64 / scale,
        trace: data[9] as f64 / scale,
        rank: data[10] as usize,
        entropy: data[11] as f64 / 100.0,
        timestamp: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_spectral_features_extraction() {
        let responses = vec![
            vec![1.0, 2.0, 3.0],
            vec![1.1, 2.1, 3.1],
            vec![0.9, 1.9, 2.9],
            vec![1.2, 2.2, 3.2],
        ];
        
        let features = extract_spectral_features(&responses);
        
        assert!(features.eigenvalues.len() == 8);
        assert!(features.spectral_radius >= 0.0);
    }
    
    #[test]
    fn test_spectral_distance() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![1.0, 2.0, 3.0];
        assert_eq!(spectral_distance(&a, &b), 0.0);
        
        let c = vec![2.0, 4.0, 6.0];
        let distance = spectral_distance(&a, &c);
        assert!(distance > 0.0);
    }
    
    #[test]
    fn test_homogeneity_detection() {
        let a = vec![1.0, 0.5, 0.25];
        let b = vec![1.1, 0.55, 0.275]; // 相似但按比例缩放
        let c = vec![0.1, 0.05, 0.025]; // 差异很大
        
        assert!(is_homogeneous(&a, &b, 0.9));
        assert!(!is_homogeneous(&a, &c, 0.9));
    }
    
    #[test]
    fn test_i64_conversion() {
        let responses = vec![
            vec![1.0, 2.0, 3.0],
            vec![1.1, 2.1, 3.1],
            vec![0.9, 1.9, 2.9],
        ];
        
        let features = extract_spectral_features(&responses);
        let i64_array = features_to_i64(&features, 8);
        
        // 应该能够转换回来
        let recovered = i64_to_features(&i64_array, 8);
        
        // 特征值应该被近似保留
        for (original, recovered_eig) in features.eigenvalues.iter().zip(recovered.eigenvalues.iter()) {
            let diff = (original - recovered_eig).abs();
            assert!(diff < 0.001, "Difference too large: {}", diff);
        }
    }
    
    #[test]
    fn test_valid_spectral() {
        let valid = SpectralFeatures {
            eigenvalues: vec![5.0, 3.0, 1.0],
            spectral_radius: 5.0,
            trace: 9.0,
            rank: 3,
            entropy: 0.8,
            timestamp: 0,
        };
        
        assert!(is_valid_spectral(&valid, 0.5));
        
        let invalid = SpectralFeatures {
            eigenvalues: vec![0.0, 0.0, 0.0],
            spectral_radius: 0.0,
            trace: 0.0,
            rank: 0,
            entropy: 0.0,
            timestamp: 0,
        };
        
        assert!(!is_valid_spectral(&invalid, 0.5));
    }
}
