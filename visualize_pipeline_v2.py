#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
绘制专业美观的多智能体预言机系统 Pipeline 流程图 v2
"""

import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch, Rectangle, Circle, Polygon
import numpy as np

# 设置样式 - 使用更专业的颜色方案
plt.style.use('default')
plt.rcParams['font.family'] = 'DejaVu Sans'
plt.rcParams['font.size'] = 10
plt.rcParams['axes.unicode_minus'] = False

# 专业配色方案 (类似 Nature/Science 论文风格)
COLORS = {
    'primary': '#2C3E50',      # 深蓝灰 - 主色
    'secondary': '#3498DB',     # 亮蓝 - 强调
    'accent': '#E74C3C',        # 红色 - 警告/重要
    'success': '#27AE60',       # 绿色 - 成功
    'warning': '#F39C12',       # 橙色 - 中间状态
    'light': '#ECF0F1',         # 浅灰 - 背景
    'dark': '#1A252F',          # 深色 - 文字
    'white': '#FFFFFF',
    'phase1': '#EBF5FB',        # 浅蓝 - Phase 1
    'phase2': '#E8F8F5',        # 浅绿 - Phase 2
    'phase3': '#F5EEF8',        # 浅紫 - Phase 3
}

def draw_clean_pipeline():
    """绘制简洁清晰的 Pipeline 流程图"""
    
    fig, ax = plt.subplots(figsize=(14, 10))
    ax.set_xlim(0, 14)
    ax.set_ylim(0, 10)
    ax.axis('off')
    ax.set_facecolor('white')
    
    # 标题
    ax.text(7, 9.5, 'Multi-Agent Oracle System Architecture', 
            ha='center', va='center', fontsize=18, fontweight='bold', color=COLORS['primary'])
    ax.text(7, 9.0, 'Causal Fingerprinting Based Decentralized Consensus', 
            ha='center', va='center', fontsize=11, style='italic', color='#7F8C8D')
    
    # ===== Phase 1: User Input =====
    phase1_y = 7.5
    
    # 阶段标题
    ax.text(1.5, phase1_y+0.8, 'PHASE 1', ha='center', va='center', 
            fontsize=9, fontweight='bold', color=COLORS['secondary'])
    ax.text(1.5, phase1_y+0.4, 'User Request', ha='center', va='center', 
            fontsize=10, fontweight='bold', color=COLORS['dark'])
    
    # 输入框
    input_box = FancyBboxPatch((0.5, phase1_y-0.5), 2, 0.8,
                               boxstyle="round,pad=0.02,rounding_size=0.1",
                               facecolor=COLORS['phase1'], edgecolor=COLORS['secondary'], 
                               linewidth=2)
    ax.add_patch(input_box)
    ax.text(1.5, phase1_y-0.1, 'Prediction Task\n(e.g., Economic Forecast)', 
            ha='center', va='center', fontsize=8, color=COLORS['dark'])
    
    # ===== Phase 2: Agent Generation (Main Box) =====
    phase2_y = 4.5
    
    # 大框背景
    main_box = FancyBboxPatch((3, phase2_y-1.5), 8, 3.2,
                              boxstyle="round,pad=0.05,rounding_size=0.15",
                              facecolor=COLORS['light'], edgecolor=COLORS['primary'], 
                              linewidth=2.5)
    ax.add_patch(main_box)
    
    # 阶段标题
    ax.text(7, phase2_y+1.4, 'PHASE 2: Agent Generation', ha='center', va='center', 
            fontsize=11, fontweight='bold', color=COLORS['primary'])
    ax.text(7, phase2_y+1.0, 'Causal Fingerprinting Protocol', ha='center', va='center', 
            fontsize=9, style='italic', color='#7F8C8D')
    
    # 5个步骤 - 横向排列
    steps = [
        (4, '1. Baseline', 'f(x)', 'LLM Call'),
        (5.5, '2. Perturb', 'f(x+δ)', 'Challenge'),
        (7, '3. Delta', 'Δy = f(x+δ)-f(x)', 'Fingerprint'),
        (8.5, '4. Causal Graph', 'Structure', 'AI Reasoning'),
        (10, '5. Spectral', '8-dim Vector', 'Features'),
    ]
    
    for x, title, content, subtitle in steps:
        # 步骤框
        step_box = FancyBboxPatch((x-0.6, phase2_y-0.6), 1.2, 1.5,
                                  boxstyle="round,pad=0.02,rounding_size=0.08",
                                  facecolor='white', edgecolor=COLORS['secondary'], 
                                  linewidth=1.5)
        ax.add_patch(step_box)
        ax.text(x, phase2_y+0.6, title, ha='center', va='center', 
                fontsize=7, fontweight='bold', color=COLORS['secondary'])
        ax.text(x, phase2_y+0.1, content, ha='center', va='center', 
                fontsize=8, fontweight='bold', color=COLORS['dark'])
        ax.text(x, phase2_y-0.35, subtitle, ha='center', va='center', 
                fontsize=6, color='#7F8C8D')
        
        # 步骤间的箭头
        if x < 10:
            ax.annotate('', xy=(x+0.7, phase2_y+0.2), xytext=(x+0.65, phase2_y+0.2),
                       arrowprops=dict(arrowstyle='->', color=COLORS['secondary'], lw=1.5))
    
    # 三层验证标注 (在步骤下方)
    validation_text = 'Three-Layer Verification: ① Intervention Response  →  ② Spectral Analysis  →  ③ Global Fingerprint'
    ax.text(7, phase2_y-1.0, validation_text, ha='center', va='center', 
            fontsize=8, style='italic', color='#7F8C8D',
            bbox=dict(boxstyle='round,pad=0.3', facecolor='#FFF9E6', edgecolor='#F39C12', alpha=0.7))
    
    # ===== Phase 3: Consensus =====
    phase3_y = 1.8
    
    # 阶段标题
    ax.text(1.5, phase3_y+0.6, 'PHASE 3', ha='center', va='center', 
            fontsize=9, fontweight='bold', color=COLORS['success'])
    ax.text(1.5, phase3_y+0.2, 'Consensus', ha='center', va='center', 
            fontsize=10, fontweight='bold', color=COLORS['dark'])
    
    # 共识框
    consensus_box = FancyBboxPatch((3, phase3_y-0.5), 8, 1.3,
                                   boxstyle="round,pad=0.03,rounding_size=0.1",
                                   facecolor=COLORS['phase2'], edgecolor=COLORS['success'], 
                                   linewidth=2)
    ax.add_patch(consensus_box)
    
    consensus_steps = ['Collect', 'Spectral', 'Fingerprint', 'Cluster', 'Outlier']
    consensus_subs = ['Responses', 'Global', 'Generation', 'Cosine', 'Detection']
    
    for i, (step, sub) in enumerate(zip(consensus_steps, consensus_subs)):
        x = 3.8 + i * 1.6
        ax.text(x, phase3_y+0.3, step, ha='center', va='center', 
                fontsize=8, fontweight='bold', color=COLORS['dark'])
        ax.text(x, phase3_y-0.1, sub, ha='center', va='center', 
                fontsize=6, color='#7F8C8D')
        if i < len(consensus_steps) - 1:
            ax.text(x+0.8, phase3_y+0.1, '→', ha='center', va='center', 
                    fontsize=10, color=COLORS['success'])
    
    # ===== Output =====
    output_y = 0.4
    
    output_box = FancyBboxPatch((4, output_y-0.25), 6, 0.5,
                                boxstyle="round,pad=0.02,rounding_size=0.1",
                                facecolor=COLORS['phase3'], edgecolor='#9B59B6', 
                                linewidth=2)
    ax.add_patch(output_box)
    ax.text(7, output_y, 'Output: Consensus Value  |  Accuracy  |  Convergence Time  |  Byzantine Detection', 
            ha='center', va='center', fontsize=9, fontweight='bold', color=COLORS['dark'])
    
    # ===== 连接箭头 =====
    # Phase 1 -> Phase 2
    ax.annotate('', xy=(5, phase2_y+1.6), xytext=(2.5, phase1_y+0.3),
               arrowprops=dict(arrowstyle='->', color=COLORS['primary'], lw=2,
                              connectionstyle="arc3,rad=0.1"))
    
    # Phase 2 -> Phase 3
    ax.annotate('', xy=(7, phase3_y+0.8), xytext=(7, phase2_y-1.5),
               arrowprops=dict(arrowstyle='->', color=COLORS['primary'], lw=2))
    
    # Phase 3 -> Output
    ax.annotate('', xy=(7, output_y+0.25), xytext=(7, phase3_y-0.5),
               arrowprops=dict(arrowstyle='->', color=COLORS['primary'], lw=2))
    
    # ===== 右侧说明框 =====
    # 关键特性
    features_y = 5.5
    feature_box = FancyBboxPatch((11.5, features_y-1), 2, 2.5,
                                 boxstyle="round,pad=0.03,rounding_size=0.1",
                                 facecolor='#FEF9E7', edgecolor='#F4D03F', 
                                 linewidth=1.5)
    ax.add_patch(feature_box)
    ax.text(12.5, features_y+1.2, 'Key Features', ha='center', va='center', 
            fontsize=9, fontweight='bold', color=COLORS['dark'])
    
    features = [
        '• Non-linear Locking',
        '• High-Dim Defense',
        '• Model Heterogeneity',
        '• Statistical Security',
    ]
    for i, feat in enumerate(features):
        ax.text(11.7, features_y+0.7-i*0.35, feat, ha='left', va='center', 
                fontsize=7, color=COLORS['dark'])
    
    plt.tight_layout()
    plt.savefig('overall_pipeline_v2.png', dpi=300, bbox_inches='tight', facecolor='white')
    print("Saved: overall_pipeline_v2.png")
    plt.close()


def draw_detailed_pipeline():
    """绘制更详细的流程图（带数据流向）"""
    
    fig, ax = plt.subplots(figsize=(16, 12))
    ax.set_xlim(0, 16)
    ax.set_ylim(0, 12)
    ax.axis('off')
    
    # 标题
    ax.text(8, 11.5, 'Causal Fingerprinting: End-to-End Pipeline', 
            ha='center', va='center', fontsize=20, fontweight='bold', color=COLORS['primary'])
    ax.text(8, 11.0, 'From User Request to Consensus Output', 
            ha='center', va='center', fontsize=12, style='italic', color='#7F8C8D')
    
    # ===== Row 1: Input Layer =====
    row1_y = 9.5
    
    input_rect = FancyBboxPatch((1, row1_y-0.4), 3, 0.8,
                                boxstyle="round,pad=0.03,rounding_size=0.1",
                                facecolor='#E8F6F3', edgecolor='#1ABC9C', linewidth=2)
    ax.add_patch(input_rect)
    ax.text(2.5, row1_y+0.1, 'USER REQUEST', ha='center', va='center', 
            fontsize=10, fontweight='bold', color=COLORS['primary'])
    ax.text(2.5, row1_y-0.2, 'Economic Prediction Task', ha='center', va='center', 
            fontsize=8, color='#7F8C8D')
    
    # ===== Row 2: Agent Layer =====
    row2_y = 7.5
    
    # 背景
    agent_bg = FancyBboxPatch((0.5, row2_y-1.5), 15, 2.8,
                              boxstyle="round,pad=0.05,rounding_size=0.15",
                              facecolor='#F8F9F9', edgecolor='#BDC3C7', linewidth=1, linestyle='--')
    ax.add_patch(agent_bg)
    
    # Agent 1-N
    num_agents = 5
    agent_width = 2.2
    agent_spacing = 0.3
    start_x = 1
    
    for i in range(num_agents):
        x = start_x + i * (agent_width + agent_spacing)
        
        # Agent 框
        agent_rect = FancyBboxPatch((x, row2_y-1.2), agent_width, 2.2,
                                    boxstyle="round,pad=0.02,rounding_size=0.08",
                                    facecolor='white', edgecolor=COLORS['secondary'], linewidth=1.5)
        ax.add_patch(agent_rect)
        
        # Agent 标题
        ax.text(x+agent_width/2, row2_y+0.8, f'Agent {i+1}', ha='center', va='center',
                fontsize=9, fontweight='bold', color=COLORS['secondary'])
        
        # 步骤
        steps = ['f(x)', 'f(x+δ)', 'Δy', 'Graph', 'Spectral']
        for j, step in enumerate(steps):
            step_y = row2_y + 0.3 - j * 0.4
            ax.text(x+agent_width/2, step_y, step, ha='center', va='center',
                    fontsize=7, color=COLORS['dark'])
        
        # 向下箭头
        ax.annotate('', xy=(x+agent_width/2, 5.8), xytext=(x+agent_width/2, 6.3),
                   arrowprops=dict(arrowstyle='->', color='#BDC3C7', lw=1))
    
    ax.text(8, row2_y+1.1, 'PARALLEL AGENT GENERATION', ha='center', va='center',
            fontsize=11, fontweight='bold', color=COLORS['primary'])
    
    # ===== Row 3: Data Aggregation =====
    row3_y = 5
    
    agg_box = FancyBboxPatch((2, row3_y-0.4), 12, 0.8,
                             boxstyle="round,pad=0.03,rounding_size=0.1",
                             facecolor='#FEF5E7', edgecolor='#E67E22', linewidth=2)
    ax.add_patch(agg_box)
    ax.text(8, row3_y+0.1, 'DATA AGGREGATION', ha='center', va='center',
            fontsize=10, fontweight='bold', color=COLORS['primary'])
    ax.text(8, row3_y-0.2, 'Collect all δ-responses & spectral features from N agents', 
            ha='center', va='center', fontsize=8, color='#7F8C8D')
    
    # ===== Row 4: Consensus Core =====
    row4_y = 3
    
    # 主框
    consensus_main = FancyBboxPatch((1, row4_y-1), 14, 2.2,
                                    boxstyle="round,pad=0.05,rounding_size=0.15",
                                    facecolor='#EAFAF1', edgecolor=COLORS['success'], linewidth=2.5)
    ax.add_patch(consensus_main)
    
    ax.text(8, row4_y+0.9, 'CONSENSUS COMPUTATION', ha='center', va='center',
            fontsize=11, fontweight='bold', color=COLORS['success'])
    
    # 内部步骤
    consensus_modules = [
        (3, 'Response\nMatrix', 'N × M'),
        (5.5, 'Spectral\nAnalysis', 'Eigenvalues'),
        (8, 'Causal\nFingerprint', 'Cosine Sim'),
        (10.5, 'Clustering\n& Detection', 'Majority Vote'),
    ]
    
    for x, title, subtitle in consensus_modules:
        mod_box = FancyBboxPatch((x-0.9, row4_y-0.5), 1.8, 1.1,
                                 boxstyle="round,pad=0.02,rounding_size=0.08",
                                 facecolor='white', edgecolor=COLORS['success'], linewidth=1)
        ax.add_patch(mod_box)
        ax.text(x, row4_y+0.2, title, ha='center', va='center',
                fontsize=8, fontweight='bold', color=COLORS['dark'])
        ax.text(x, row4_y-0.2, subtitle, ha='center', va='center',
                fontsize=7, color='#7F8C8D')
        
        if x < 10.5:
            ax.annotate('', xy=(x+1.05, row4_y), xytext=(x+0.95, row4_y),
                       arrowprops=dict(arrowstyle='->', color=COLORS['success'], lw=2))
    
    # ===== Row 5: Output =====
    row5_y = 1
    
    output_box = FancyBboxPatch((4, row5_y-0.4), 8, 0.8,
                                boxstyle="round,pad=0.03,rounding_size=0.1",
                                facecolor='#F5EEF8', edgecolor='#8E44AD', linewidth=2)
    ax.add_patch(output_box)
    ax.text(8, row5_y+0.1, 'CONSENSUS OUTPUT', ha='center', va='center',
            fontsize=11, fontweight='bold', color='#8E44AD')
    
    # 输出指标
    metrics = [
        ('Value', 'Consensus'),
        ('Accuracy', 'Error Rate'),
        ('Time', 'Convergence'),
        ('Security', 'Detection'),
    ]
    
    for i, (metric, sub) in enumerate(metrics):
        x = 5 + i * 2
        ax.text(x, row5_y-0.2, f'{metric}: {sub}', ha='center', va='center',
                fontsize=7, color=COLORS['dark'])
    
    # ===== 连接箭头 =====
    # Input -> Agents
    ax.annotate('', xy=(2.5, 6.3), xytext=(2.5, 9.1),
               arrowprops=dict(arrowstyle='->', color=COLORS['primary'], lw=2))
    
    # Agents -> Aggregation
    ax.annotate('', xy=(8, 5.4), xytext=(8, 6.3),
               arrowprops=dict(arrowstyle='->', color=COLORS['primary'], lw=2))
    
    # Aggregation -> Consensus
    ax.annotate('', xy=(8, 3.2), xytext=(8, 4.6),
               arrowprops=dict(arrowstyle='->', color=COLORS['primary'], lw=2))
    
    # Consensus -> Output
    ax.annotate('', xy=(8, 0.6), xytext=(8, 2.0),
               arrowprops=dict(arrowstyle='->', color=COLORS['primary'], lw=2))
    
    # ===== 右侧安全说明 =====
    sec_x = 15
    sec_y = 6
    
    sec_box = FancyBboxPatch((sec_x-0.3, sec_y-2), 1, 4,
                             boxstyle="round,pad=0.03,rounding_size=0.1",
                             facecolor='#FDEDEC', edgecolor=COLORS['accent'], linewidth=1.5)
    ax.add_patch(sec_box)
    
    ax.text(sec_x+0.2, sec_y+1.5, 'SECURITY', ha='center', va='center',
            fontsize=9, fontweight='bold', color=COLORS['accent'], rotation=90)
    
    security_items = [
        'Byzantine',
        'Tolerance:',
        '40%',
        '',
        'Attack Cost:',
        'Exponential',
        '',
        'Detection:',
        'Spectral',
    ]
    
    for i, item in enumerate(security_items):
        ax.text(sec_x+0.2, sec_y+0.8-i*0.25, item, ha='center', va='center',
                fontsize=6, color=COLORS['dark'], rotation=90)
    
    plt.tight_layout()
    plt.savefig('overall_pipeline_detailed.png', dpi=300, bbox_inches='tight', facecolor='white')
    print("Saved: overall_pipeline_detailed.png")
    plt.close()


if __name__ == '__main__':
    print("=" * 60)
    print("绘制专业版 Multi-Agent Oracle Pipeline")
    print("=" * 60)
    print()
    
    draw_clean_pipeline()
    draw_detailed_pipeline()
    
    print()
    print("✅ 图表生成完成!")
    print()
    print("生成的文件:")
    print("  1. overall_pipeline_v2.png - 简洁清晰版")
    print("  2. overall_pipeline_detailed.png - 详细完整版")
