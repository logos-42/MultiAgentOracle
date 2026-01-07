//! 信誉存储管理器 - 因果指纹版
//!
//! 负责信誉数据的持久化和文件操作

use crate::reputation::{ReputationScore, ReputationHistory};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, warn, error};

/// 信誉存储管理器
pub struct ReputationStorage {
    /// 数据目录
    data_dir: PathBuf,
    /// 内存缓存
    cache: Arc<RwLock<HashMap<String, ReputationScore>>>,
    /// 是否启用持久化
    persistence_enabled: bool,
}

impl ReputationStorage {
    /// 创建新的信誉存储管理器
    pub fn new(data_dir: &str, persistence_enabled: bool) -> Result<Self> {
        let path = PathBuf::from(data_dir);
        
        // 创建数据目录
        if !path.exists() {
            fs::create_dir_all(&path)
                .map_err(|e| anyhow!("创建数据目录失败: {}", e))?;
        }
        
        info!("📁 初始化信誉存储: {}", path.display());
        
        Ok(Self {
            data_dir: path,
            cache: Arc::new(RwLock::new(HashMap::new())),
            persistence_enabled,
        })
    }
    
    /// 加载所有信誉数据
    pub async fn load_all(&self) -> Result<()> {
        if !self.persistence_enabled {
            return Ok(());
        }
        
        let mut cache = self.cache.write().await;
        cache.clear();
        
        let scores_dir = self.data_dir.join("scores");
        if !scores_dir.exists() {
            return Ok(());
        }
        
        let entries = fs::read_dir(&scores_dir)
            .map_err(|e| anyhow!("读取信誉数据目录失败: {}", e))?;
        
        let mut loaded_count = 0;
        for entry in entries {
            let entry = entry.map_err(|e| anyhow!("读取目录条目失败: {}", e))?;
            let path = entry.path();
            
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load_score_file(&path).await {
                    Ok(score) => {
                        cache.insert(score.agent_did.clone(), score);
                        loaded_count += 1;
                    }
                    Err(e) => {
                        warn!("加载信誉文件失败 {}: {}", path.display(), e);
                    }
                }
            }
        }
        
        info!("📂 加载信誉数据: {} 个智能体", loaded_count);
        Ok(())
    }
    
    /// 保存所有信誉数据
    pub async fn save_all(&self) -> Result<()> {
        if !self.persistence_enabled {
            return Ok(());
        }
        
        let cache = self.cache.read().await;
        let scores_dir = self.data_dir.join("scores");
        
        if !scores_dir.exists() {
            fs::create_dir_all(&scores_dir)
                .map_err(|e| anyhow!("创建分数目录失败: {}", e))?;
        }
        
        let mut saved_count = 0;
        for score in cache.values() {
            match self.save_score(score).await {
                Ok(_) => saved_count += 1,
                Err(e) => {
                    error!("保存信誉数据失败 {}: {}", score.agent_did, e);
                }
            }
        }
        
        info!("💾 保存信誉数据: {} 个智能体", saved_count);
        Ok(())
    }
    
    /// 加载单个信誉分数文件
    async fn load_score_file(&self, path: &Path) -> Result<ReputationScore> {
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow!("读取文件失败: {}", e))?;
        
        let score: ReputationScore = serde_json::from_str(&content)
            .map_err(|e| anyhow!("解析JSON失败: {}", e))?;
        
        Ok(score)
    }
    
    /// 保存单个信誉分数
    pub async fn save_score(&self, score: &ReputationScore) -> Result<()> {
        if !self.persistence_enabled {
            return Ok(());
        }
        
        let scores_dir = self.data_dir.join("scores");
        if !scores_dir.exists() {
            fs::create_dir_all(&scores_dir)
                .map_err(|e| anyhow!("创建分数目录失败: {}", e))?;
        }
        
        let filename = format!("{}.json", score.agent_did.replace(":", "_"));
        let filepath = scores_dir.join(filename);
        
        let content = serde_json::to_string_pretty(score)
            .map_err(|e| anyhow!("序列化JSON失败: {}", e))?;
        
        fs::write(&filepath, content)
            .map_err(|e| anyhow!("写入文件失败: {}", e))?;
        
        Ok(())
    }
    
    /// 保存信誉更新历史
    pub async fn save_history(&self, history: &ReputationHistory) -> Result<()> {
        if !self.persistence_enabled {
            return Ok(());
        }
        
        let history_dir = self.data_dir.join("history");
        if !history_dir.exists() {
            fs::create_dir_all(&history_dir)
                .map_err(|e| anyhow!("创建历史目录失败: {}", e))?;
        }
        
        let filename = format!("{}_{}_{}.json", 
            history.agent_did.replace(":", "_"),
            history.start_time,
            history.end_time);
        let filepath = history_dir.join(filename);
        
        let content = serde_json::to_string_pretty(history)
            .map_err(|e| anyhow!("序列化历史JSON失败: {}", e))?;
        
        fs::write(&filepath, content)
            .map_err(|e| anyhow!("写入历史文件失败: {}", e))?;
        
        Ok(())
    }
    
    /// 加载信誉历史
    pub async fn load_history(
        &self, 
        agent_did: &str, 
        start_time: u64, 
        end_time: u64
    ) -> Result<Option<ReputationHistory>> {
        if !self.persistence_enabled {
            return Ok(None);
        }
        
        let history_dir = self.data_dir.join("history");
        if !history_dir.exists() {
            return Ok(None);
        }
        
        let filename = format!("{}_{}_{}.json", 
            agent_did.replace(":", "_"),
            start_time,
            end_time);
        let filepath = history_dir.join(filename);
        
        if !filepath.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&filepath)
            .map_err(|e| anyhow!("读取历史文件失败: {}", e))?;
        
        let history: ReputationHistory = serde_json::from_str(&content)
            .map_err(|e| anyhow!("解析历史JSON失败: {}", e))?;
        
        Ok(Some(history))
    }
    
    /// 获取智能体的所有历史记录
    pub async fn get_all_history(&self, agent_did: &str) -> Result<Vec<ReputationHistory>> {
        if !self.persistence_enabled {
            return Ok(Vec::new());
        }
        
        let history_dir = self.data_dir.join("history");
        if !history_dir.exists() {
            return Ok(Vec::new());
        }
        
        let prefix = format!("{}_", agent_did.replace(":", "_"));
        let mut histories = Vec::new();
        
        let entries = fs::read_dir(&history_dir)
            .map_err(|e| anyhow!("读取历史目录失败: {}", e))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| anyhow!("读取目录条目失败: {}", e))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with(&prefix) && filename.ends_with(".json") {
                        match self.load_history_file(&path).await {
                            Ok(history) => histories.push(history),
                            Err(e) => {
                                warn!("加载历史文件失败 {}: {}", path.display(), e);
                            }
                        }
                    }
                }
            }
        }
        
        // 按时间排序
        histories.sort_by(|a, b| a.start_time.cmp(&b.start_time));
        
        Ok(histories)
    }
    
    /// 加载历史文件
    async fn load_history_file(&self, path: &Path) -> Result<ReputationHistory> {
        let content = fs::read_to_string(path)
            .map_err(|e| anyhow!("读取历史文件失败: {}", e))?;
        
        let history: ReputationHistory = serde_json::from_str(&content)
            .map_err(|e| anyhow!("解析历史JSON失败: {}", e))?;
        
        Ok(history)
    }
    
    /// 清理旧的历史记录
    pub async fn cleanup_old_history(&self, max_age_days: u64) -> Result<usize> {
        if !self.persistence_enabled {
            return Ok(0);
        }
        
        let history_dir = self.data_dir.join("history");
        if !history_dir.exists() {
            return Ok(0);
        }
        
        let max_age_secs = max_age_days * 86400;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let mut removed_count = 0;
        
        let entries = fs::read_dir(&history_dir)
            .map_err(|e| anyhow!("读取历史目录失败: {}", e))?;
        
        for entry in entries {
            let entry = entry.map_err(|e| anyhow!("读取目录条目失败: {}", e))?;
            let path = entry.path();
            
            if path.is_file() {
                if let Ok(metadata) = fs::metadata(&path) {
                    if let Ok(modified) = metadata.modified() {
                        let modified_secs = modified
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or(std::time::Duration::from_secs(0))
                            .as_secs();
                        
                        if now - modified_secs > max_age_secs {
                            match fs::remove_file(&path) {
                                Ok(_) => {
                                    removed_count += 1;
                                    info!("🗑️ 清理旧历史文件: {}", path.display());
                                }
                                Err(e) => {
                                    warn!("删除历史文件失败 {}: {}", path.display(), e);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        info!("🧹 清理历史记录: {} 个文件", removed_count);
        Ok(removed_count)
    }
    
    /// 导出所有数据
    pub async fn export_all(&self, export_dir: &str) -> Result<()> {
        let export_path = PathBuf::from(export_dir);
        
        if !export_path.exists() {
            fs::create_dir_all(&export_path)
                .map_err(|e| anyhow!("创建导出目录失败: {}", e))?;
        }
        
        // 导出信誉分数
        let cache = self.cache.read().await;
        let scores_export: Vec<&ReputationScore> = cache.values().collect();
        
        let scores_json = serde_json::to_string_pretty(&scores_export)
            .map_err(|e| anyhow!("序列化信誉分数失败: {}", e))?;
        
        let scores_file = export_path.join("reputation_scores.json");
        fs::write(&scores_file, scores_json)
            .map_err(|e| anyhow!("写入信誉分数文件失败: {}", e))?;
        
        // 导出统计信息
        let stats = self.generate_stats(&cache).await;
        let stats_json = serde_json::to_string_pretty(&stats)
            .map_err(|e| anyhow!("序列化统计信息失败: {}", e))?;
        
        let stats_file = export_path.join("reputation_stats.json");
        fs::write(&stats_file, stats_json)
            .map_err(|e| anyhow!("写入统计文件失败: {}", e))?;
        
        info!("📤 导出信誉数据到: {}", export_path.display());
        Ok(())
    }
    
    /// 生成统计信息
    async fn generate_stats(&self, cache: &HashMap<String, ReputationScore>) -> StorageStats {
        let mut stats = StorageStats {
            total_agents: cache.len(),
            active_agents: 0,
            average_credit: 0.0,
            tier_distribution: HashMap::new(),
            total_history_entries: 0,
            storage_size_mb: 0.0,
        };
        
        for score in cache.values() {
            if score.is_active {
                stats.active_agents += 1;
            }
            
            stats.average_credit += score.causal_credit;
            stats.total_history_entries += score.history.len();
            
            *stats.tier_distribution.entry(score.tier.name().to_string())
                .or_insert(0) += 1;
        }
        
        if !cache.is_empty() {
            stats.average_credit /= cache.len() as f64;
        }
        
        // 计算存储大小
        if let Ok(metadata) = fs::metadata(&self.data_dir) {
            stats.storage_size_mb = metadata.len() as f64 / 1024.0 / 1024.0;
        }
        
        stats
    }
    
    /// 备份数据
    pub async fn backup(&self, backup_dir: &str) -> Result<()> {
        let backup_path = PathBuf::from(backup_dir);
        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("reputation_backup_{}", timestamp);
        let full_backup_path = backup_path.join(&backup_name);
        
        if !backup_path.exists() {
            fs::create_dir_all(&backup_path)
                .map_err(|e| anyhow!("创建备份目录失败: {}", e))?;
        }
        
        // 复制整个数据目录
        if self.data_dir.exists() {
            copy_dir_all(&self.data_dir, &full_backup_path)
                .map_err(|e| anyhow!("复制数据目录失败: {}", e))?;
        }
        
        info!("💾 备份信誉数据到: {}", full_backup_path.display());
        Ok(())
    }
    
    /// 恢复数据
    pub async fn restore(&self, backup_dir: &str) -> Result<()> {
        let backup_path = PathBuf::from(backup_dir);
        
        if !backup_path.exists() {
            return Err(anyhow!("备份目录不存在: {}", backup_dir));
        }
        
        // 清空当前数据目录
        if self.data_dir.exists() {
            fs::remove_dir_all(&self.data_dir)
                .map_err(|e| anyhow!("清空数据目录失败: {}", e))?;
        }
        
        // 从备份恢复
        copy_dir_all(&backup_path, &self.data_dir)
            .map_err(|e| anyhow!("恢复数据目录失败: {}", e))?;
        
        // 重新加载数据
        self.load_all().await?;
        
        info!("🔄 从备份恢复信誉数据: {}", backup_path.display());
        Ok(())
    }
}

/// 存储统计 - 因果指纹版
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    /// 总智能体数
    pub total_agents: usize,
    /// 活跃智能体数
    pub active_agents: usize,
    /// 平均因果信用分
    pub average_credit: f64,
    /// 等级分布
    pub tier_distribution: HashMap<String, usize>,
    /// 总历史记录条目数
    pub total_history_entries: usize,
    /// 存储大小 (MB)
    pub storage_size_mb: f64,
}

/// 递归复制目录
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    if !src.exists() {
        return Err(anyhow!("源目录不存在: {}", src.display()));
    }
    
    if !dst.exists() {
        fs::create_dir_all(dst)
            .map_err(|e| anyhow!("创建目标目录失败: {}", e))?;
    }
    
    for entry in fs::read_dir(src)
        .map_err(|e| anyhow!("读取源目录失败: {}", e))? 
    {
        let entry = entry.map_err(|e| anyhow!("读取目录条目失败: {}", e))?;
        let ty = entry.file_type()
            .map_err(|e| anyhow!("获取文件类型失败: {}", e))?;
        
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        
        if ty.is_dir() {
            copy_dir_all(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)
                .map_err(|e| anyhow!("复制文件失败: {}", e))?;
        }
    }
    
    Ok(())
}
