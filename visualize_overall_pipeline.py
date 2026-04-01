#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
绘制多智能体预言机系统的整体 Pipeline 流程图
"""

import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch, FancyArrowPatch, ConnectionPatch
import numpy as np

# 设置样式
plt.style.use('seaborn-v0_8-whitegrid')
plt.rcParams['font.family'] = 'DejaVu Sans'
plt.rcParams['font.size'] = 10

def draw_pipeline():
    """绘制整体 Pipeline 流程图"""
    
    fig, ax = plt.subplots(figsize=(16, 12))
    ax.set_xlim(0, 16)
    ax.set_ylim(0, 12)
    ax.axis('off')
    
    # 颜色定义
    colors = {
        'user': '#FFE5B4',        # 浅橙色 - 用户
        'agent': '#E3F2FD',       # 浅蓝色 - Agent
        'consensus': '#E8F5E9',   # 浅绿色 - 共识
        'result': '#F3E5F5',       # 浅紫色 - 结果
        'arrow': '#424242',       # 箭头颜色
        'title': '#1565C0',       # 标题颜色
    }
    
    # ===== 第一行：用户请求 =====
    user_box = FancyBboxPatch((1, 10), 3, 1.5, 
                              boxstyle="round,pad=0.05,rounding_size=0.2",
                              facecolor=colors['user'], edgecolor='black', linewidth=1.5)
    ax.add_patch(user_box)
    ax.text(2.5, 10.9, 'User Request', ha='center', va='center', fontsize=11, fontweight='bold')
    ax.text(2.5, 10.5, '(Prediction Task)', ha='center', va='center', fontsize=9, style='italic')
    
    # ===== 第二行：Agent 生成阶段 =====
    # 主框
    agent_box = FancyBboxPatch((0.5, 5.5), 15, 4, 
                               boxstyle="round,pad=0.1,rounding_size=0.3",
                               facecolor=colors['agent'], edgecolor='#1976D2', linewidth=2)
    ax.add_patch(agent_box)
    ax.text(8, 9.2, 'Agent Generation Phase', ha='center', va='center', fontsize=13, fontweight='bold', color=colors['title'])
    
    # Agent 内部的步骤
    steps = [
        (1.5, 7.5, 'LLM f(x)\n(Baseline)', 'Generate baseline\nprediction'),
        (4.5, 7.5, 'LLM f(x+δ)\n(Perturbation)', 'Generate perturbed\nprediction'),
        (7.5, 7.5, 'Δ = f(x+δ) - f(x)\n(Delta)', 'Compute incremental\nresponse'),
        (10.5, 7.5, 'Causal Graph\nGeneration', 'AI reasoning\ncausal structure'),
        (13.5, 7.5, 'Spectral\nFeatures', 'Extract 8-dim\nfeatures'),
    ]
    
    for x, y, title, subtitle in steps:
        box = FancyBboxPatch((x-0.8, y-0.6), 1.6, 1.2,
                            boxstyle="round,pad=0.03,rounding_size=0.1",
                            facecolor='white', edgecolor='#64B5F6', linewidth=1)
        ax.add_patch(box)
        ax.text(x, y+0.15, title, ha='center', va='center', fontsize=8, fontweight='bold')
        ax.text(x, y-0.35, subtitle, ha='center', va='center', fontsize=6, style='italic', color='#666')
        
        # 箭头连接
        if x < 13.5:
            ax.annotate('', xy=(x+0.9, y), xytext=(x+1.6, y),
                       arrowprops=dict(arrowstyle='->', color=colors['arrow'], lw=1.5))
    
    # 三层验证框
    validation_labels = [
        (1.5, 5.8, 'Layer 1:\nIntervention\nResponse', 'Single task\nconsistency'),
        (5, 5.8, 'Layer 2:\nSpectral\nAnalysis', 'Provider\nconsistency'),
        (8.5, 5.8, 'Layer 3:\nGlobal\nFingerprint', 'Long-term\nreputation'),
    ]
    
    for x, y, title, subtitle in validation_labels:
        box = FancyBboxPatch((x-1.2, y-0.4), 2.4, 0.8,
                            boxstyle="round,pad=0.02,rounding_size=0.1",
                            facecolor='#FFF9C4', edgecolor='#FBC02D', linewidth=1)
        ax.add_patch(box)
        ax.text(x, y+0.15, title, ha='center', va='center', fontsize=7, fontweight='bold')
        ax.text(x, y-0.2, subtitle, ha='center', va='center', fontsize=6, style='italic', color='#666')
    
    ax.text(12.5, 5.8, 'Three-Layer\nVerification', ha='center', va='center', fontsize=7, fontweight='bold', color='#1565C0')
    box = FancyBboxPatch((10.5, 5.6), 4, 0.8,
                        boxstyle="round,pad=0.02,rounding_size=0.1",
                        facecolor='#FFFDE7', edgecolor='#FDD835', linewidth=1, linestyle='--')
    ax.add_patch(box)
    
    # ===== 第三行：共识计算阶段 =====
    consensus_box = FancyBboxPatch((0.5, 1.5), 15, 3, 
                                   boxstyle="round,pad=0.1,rounding_size=0.3",
                                   facecolor=colors['consensus'], edgecolor='#388E3C', linewidth=2)
    ax.add_patch(consensus_box)
    ax.text(8, 4.2, 'Consensus Computation Phase', ha='center', va='center', fontsize=13, fontweight='bold', color=colors['title'])
    
    # 共识步骤
    consensus_steps = [
        (2, 2.8, 'Collect\nResponses', 'Gather all agent\ndelta responses'),
        (5, 2.8, 'Global Spectral\nFeatures', 'Compute from\nresponse matrix'),
        (8, 2.8, 'Causal\nFingerprint', 'Generate cryptographic\nfingerprint'),
        (11, 2.8, 'Cosine\nClustering', 'Similarity-based\nclustering'),
        (14, 2.8, 'Outlier\nDetection', 'Remove malicious\nagents'),
    ]
    
    for x, y, title, subtitle in consensus_steps:
        box = FancyBboxPatch((x-0.9, y-0.7), 1.8, 1.4,
                            boxstyle="round,pad=0.03,rounding_size=0.1",
                            facecolor='white', edgecolor='#81C784', linewidth=1)
        ax.add_patch(box)
        ax.text(x, y+0.25, title, ha='center', va='center', fontsize=8, fontweight='bold')
        ax.text(x, y-0.35, subtitle, ha='center', va='center', fontsize=6, style='italic', color='#666')
        
        if x < 14:
            ax.annotate('', xy=(x+1, y), xytext=(x+1.8, y),
                       arrowprops=dict(arrowstyle='->', color=colors['arrow'], lw=1.5))
    
    # ===== 第四行：结果输出 =====
    result_box = FancyBboxPatch((4, 0.3), 8, 1, 
                                boxstyle="round,pad=0.05,rounding_size=0.2",
                                facecolor=colors['result'], edgecolor='#7B1FA2', linewidth=2)
    ax.add_patch(result_box)
    ax.text(8, 1.0, 'Output: Consensus Value | Accuracy | Convergence Time | Byzantine Detection', 
            ha='center', va='center', fontsize=10, fontweight='bold')
    
    # ===== 连接箭头 =====
    # 用户 → Agent
    ax.annotate('', xy=(8, 9.5), xytext=(2.5, 9.5),
               arrowprops=dict(arrowstyle='->', color=colors['arrow'], lw=2))
    
    # Agent → 共识
    ax.annotate('', xy=(8, 5.5), xytext=(8, 8.5),
               arrowprops=dict(arrowstyle='->', color=colors['arrow'], lw=2))
    
    # 共识 → 结果
    ax.annotate('', xy=(8, 1.3), xytext=(8, 4.5),
               arrowprops=dict(arrowstyle='->', color=colors['arrow'], lw=2))
    
    # ===== 标题 =====
    ax.text(8, 11.5, 'Multi-Agent Oracle System: Overall Pipeline', 
            ha='center', va='center', fontsize=16, fontweight='bold', color='#1565C0')
    ax.text(8, 11.0, '(Causal Fingerprinting based Decentralized Oracle)', 
            ha='center', va='center', fontsize=11, style='italic', color='#666')
    
    # 图例
    legend_elements = [
        mpatches.Patch(facecolor=colors['user'], edgecolor='black', label='User Request'),
        mpatches.Patch(facecolor=colors['agent'], edgecolor='#1976D2', label='Agent Generation'),
        mpatches.Patch(facecolor=colors['consensus'], edgecolor='#388E3C', label='Consensus Computation'),
        mpatches.Patch(facecolor=colors['result'], edgecolor='#7B1FA2', label='Output'),
    ]
    ax.legend(handles=legend_elements, loc='upper right', fontsize=9)
    
    plt.tight_layout()
    plt.savefig('overall_pipeline.png', dpi=300, bbox_inches='tight', facecolor='white')
    print("Saved: overall_pipeline.png")
    plt.close()


def draw_pipeline_vertical():
    """绘制垂直方向的 Pipeline（更适合论文）"""
    
    fig, ax = plt.subplots(figsize=(12, 16))
    ax.set_xlim(0, 12)
    ax.set_ylim(0, 16)
    ax.axis('off')
    
    # 颜色定义
    colors = {
        'user': '#FFE0B2',
        'agent': '#BBDEFB',
        'consensus': '#C8E6C9',
        'result': '#E1BEE7',
        'security': '#FFCDD2',
    }
    
    # ===== 标题 =====
    ax.text(6, 15.5, 'Multi-Agent Oracle System Architecture', 
            ha='center', va='center', fontsize=16, fontweight='bold', color='#1565C0')
    ax.text(6, 15.0, '(Causal Fingerprinting Based Decentralized Oracle Network)', 
            ha='center', va='center', fontsize=10, style='italic', color='#666')
    
    # ===== 第1层：用户请求 =====
    box1 = FancyBboxPatch((3, 13.5), 6, 1.2,
                          boxstyle="round,pad=0.05,rounding_size=0.2",
                          facecolor=colors['user'], edgecolor='#E65100', linewidth=2)
    ax.add_patch(box1)
    ax.text(6, 14.2, 'User Request', ha='center', va='center', fontsize=12, fontweight='bold')
    ax.text(6, 13.7, '(Economic Prediction Task)', ha='center', va='center', fontsize=9, style='italic')
    
    # ===== 第2层：三层验证 =====
    # 外框
    outer_box = FancyBboxPatch((0.5, 8.5), 11, 4.5,
                                boxstyle="round,pad=0.1,rounding_size=0.3",
                                facecolor='#FAFAFA', edgecolor='#757575', linewidth=1, linestyle='--')
    ax.add_patch(outer_box)
    ax.text(6, 12.7, 'Three-Layer Verification System', ha='center', va='center', fontsize=11, fontweight='bold', color='#1565C0')
    
    # 三层
    layers = [
        (1.5, 10.5, 'Layer 1', 'Intervention Response\n(Δ = f(x+δ) - f(x))', 
         'Single task\nlogic consistency', '#FFF9C4'),
        (5, 10.5, 'Layer 2', 'Spectral Analysis\n(8-dim feature vector)', 
         'Provider consistency\nattack detection', '#B3E5FC'),
        (8.5, 10.5, 'Layer 3', 'Global Fingerprint\n(Long-term reputation)', 
         'Sybil attack\nmodel collapse', '#F8BBD9'),
    ]
    
    for x, y, title, desc, detail, color in layers:
        box = FancyBboxPatch((x-1.3, y-1.3), 2.6, 2.6,
                            boxstyle="round,pad=0.05,rounding_size=0.15",
                            facecolor=color, edgecolor='#616161', linewidth=1.5)
        ax.add_patch(box)
        ax.text(x, y+0.8, title, ha='center', va='center', fontsize=9, fontweight='bold')
        ax.text(x, y+0.1, desc, ha='center', va='center', fontsize=7, fontweight='bold')
        ax.text(x, y-0.7, detail, ha='center', va='center', fontsize=6, style='italic', color='#666')
    
    # ===== 第3层：Agent 生成 =====
    agent_box = FancyBboxPatch((0.5, 5.5), 11, 2.8,
                               boxstyle="round,pad=0.1,rounding_size=0.2",
                               facecolor=colors['agent'], edgecolor='#1565C0', linewidth=2)
    ax.add_patch(agent_box)
    ax.text(6, 8.0, 'Agent Generation Pipeline', ha='center', va='center', fontsize=11, fontweight='bold', color='#1565C0')
    
    # Agent 步骤（横向排列）
    agent_steps = [
        (1.2, 6.8, 'LLM f(x)', 'Baseline\nPrediction'),
        (3.4, 6.8, 'LLM f(x+δ)', 'Perturbed\nPrediction'),
        (5.6, 6.8, 'Δ = f(x+δ)-f(x)', 'Delta\nResponse'),
        (7.8, 6.8, 'Causal Graph', 'AI Causal\nReasoning'),
        (10, 6.8, 'Spectral', '8-dim\nFeatures'),
    ]
    
    for x, y, title, subtitle in agent_steps:
        box = FancyBboxPatch((x-0.5, y-0.7), 1, 1.4,
                            boxstyle="round,pad=0.02,rounding_size=0.08",
                            facecolor='white', edgecolor='#64B5F6', linewidth=1)
        ax.add_patch(box)
        ax.text(x, y+0.25, title, ha='center', va='center', fontsize=7, fontweight='bold')
        ax.text(x, y-0.4, subtitle, ha='center', va='center', fontsize=6, style='italic', color='#666')
        
        if x < 10:
            ax.annotate('', xy=(x+0.55, y), xytext=(x+0.95, y),
                       arrowprops=dict(arrowstyle='->', color='#424242', lw=1.2))
    
    # ===== 第4层：共识计算 =====
    consensus_box = FancyBboxPatch((0.5, 2.5), 11, 2.5,
                                    boxstyle="round,pad=0.1,rounding_size=0.2",
                                    facecolor=colors['consensus'], edgecolor='#2E7D32', linewidth=2)
    ax.add_patch(consensus_box)
    ax.text(6, 4.7, 'Consensus Computation', ha='center', va='center', fontsize=11, fontweight='bold', color='#2E7D32')
    
    consensus_steps = [
        (1.5, 3.5, 'Collect', 'All Agents\'\nResponses'),
        (4, 3.5, 'Spectral', 'Global\nFeatures'),
        (6.5, 3.5, 'Fingerprint', 'Causal\nFingerprint'),
        (9, 3.5, 'Cluster', 'Cosine\nSimilarity'),
        (11.5, 3.5, 'Detect', 'Byzantine\nDetection'),
    ]
    
    for x, y, title, subtitle in consensus_steps:
        box = FancyBboxPatch((x-0.7, y-0.6), 1.4, 1.2,
                            boxstyle="round,pad=0.02,rounding_size=0.08",
                            facecolor='white', edgecolor='#81C784', linewidth=1)
        ax.add_patch(box)
        ax.text(x, y+0.15, title, ha='center', va='center', fontsize=7, fontweight='bold')
        ax.text(x, y-0.4, subtitle, ha='center', va='center', fontsize=6, style='italic', color='#666')
        
        if x < 11.5:
            ax.annotate('', xy=(x+0.75, y), xytext=(x+1.35, y),
                       arrowprops=dict(arrowstyle='->', color='#424242', lw=1.2))
    
    # ===== 第5层：结果 =====
    result_box = FancyBboxPatch((2, 0.8), 8, 1.3,
                               boxstyle="round,pad=0.05,rounding_size=0.2",
                               facecolor=colors['result'], edgecolor='#7B1FA2', linewidth=2)
    ax.add_patch(result_box)
    ax.text(6, 1.55, 'Output Results', ha='center', va='center', fontsize=11, fontweight='bold', color='#7B1FA2')
    ax.text(6, 1.0, 'Consensus Value | Accuracy | Convergence Time | Byzantine Detection Rate', 
            ha='center', va='center', fontsize=9)
    
    # ===== 连接箭头 =====
    arrows_y = [13.5, 12.7, 8.3, 5.3, 2.5, 0.8]
    for i in range(len(arrows_y)-1):
        ax.annotate('', xy=(6, arrows_y[i+1]+0.1), xytext=(6, arrows_y[i]-0.1),
                   arrowprops=dict(arrowstyle='->', color='#424242', lw=2))
    
    plt.tight_layout()
    plt.savefig('overall_pipeline_vertical.png', dpi=300, bbox_inches='tight', facecolor='white')
    print("Saved: overall_pipeline_vertical.png")
    plt.close()


if __name__ == '__main__':
    print("=" * 60)
    print("绘制多智能体预言机系统整体 Pipeline")
    print("=" * 60)
    print()
    
    draw_pipeline()
    draw_pipeline_vertical()
    
    print()
    print("✅ Pipeline 图表生成完成!")
    print("生成的文件:")
    print("  1. overall_pipeline.png - 横向布局")
    print("  2. overall_pipeline_vertical.png - 垂直布局（适合论文）")
