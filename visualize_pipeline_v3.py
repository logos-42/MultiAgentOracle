#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
绘制专业美观的多智能体预言机系统 Pipeline 流程图 v3 - 修复居中对齐
"""

import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch
import numpy as np

# 设置样式
plt.style.use('default')
plt.rcParams['font.family'] = 'DejaVu Sans'
plt.rcParams['font.size'] = 10

# 专业配色方案
COLORS = {
    'primary': '#2C3E50',
    'secondary': '#3498DB',
    'success': '#27AE60',
    'accent': '#9B59B6',
    'warning': '#F39C12',
    'light': '#ECF0F1',
    'dark': '#1A252F',
    'phase1': '#EBF5FB',
    'phase2': '#E8F8F5',
    'phase3': '#F5EEF8',
}

def draw_centered_pipeline():
    """绘制居中对齐的 Pipeline 流程图"""
    
    fig, ax = plt.subplots(figsize=(14, 10))
    ax.set_xlim(0, 14)
    ax.set_ylim(0, 10)
    ax.axis('off')
    ax.set_facecolor('white')
    
    center_x = 7  # 中心线
    
    # 标题
    ax.text(center_x, 9.5, 'Multi-Agent Oracle System Architecture', 
            ha='center', va='center', fontsize=18, fontweight='bold', color=COLORS['primary'])
    ax.text(center_x, 9.0, 'Causal Fingerprinting Based Decentralized Consensus', 
            ha='center', va='center', fontsize=11, style='italic', color='#7F8C8D')
    
    # ===== Phase 1: User Request (居中) =====
    phase1_y = 7.5
    
    # 标题 - 居中
    ax.text(center_x, phase1_y+1.0, 'PHASE 1', ha='center', va='center', 
            fontsize=10, fontweight='bold', color=COLORS['secondary'])
    ax.text(center_x, phase1_y+0.6, 'User Request', ha='center', va='center', 
            fontsize=12, fontweight='bold', color=COLORS['dark'])
    
    # 输入框 - 居中
    input_width = 3
    input_box = FancyBboxPatch((center_x - input_width/2, phase1_y-0.4), input_width, 0.8,
                               boxstyle="round,pad=0.02,rounding_size=0.1",
                               facecolor=COLORS['phase1'], edgecolor=COLORS['secondary'], 
                               linewidth=2)
    ax.add_patch(input_box)
    ax.text(center_x, phase1_y, 'Prediction Task\n(e.g., Economic Forecast)', 
            ha='center', va='center', fontsize=9, color=COLORS['dark'])
    
    # ===== Phase 2: Agent Generation (居中) =====
    phase2_y = 4.5
    
    # 大框背景 - 居中
    main_width = 10
    main_height = 3.2
    main_box = FancyBboxPatch((center_x - main_width/2, phase2_y-1.4), main_width, main_height,
                              boxstyle="round,pad=0.05,rounding_size=0.15",
                              facecolor=COLORS['light'], edgecolor=COLORS['primary'], 
                              linewidth=2.5)
    ax.add_patch(main_box)
    
    # 阶段标题 - 居中
    ax.text(center_x, phase2_y+1.3, 'PHASE 2: Agent Generation', ha='center', va='center', 
            fontsize=12, fontweight='bold', color=COLORS['primary'])
    ax.text(center_x, phase2_y+0.9, 'Causal Fingerprinting Protocol', ha='center', va='center', 
            fontsize=9, style='italic', color='#7F8C8D')
    
    # 5个步骤 - 横向排列居中
    step_width = 1.6
    step_height = 1.5
    step_spacing = 0.3
    total_steps_width = 5 * step_width + 4 * step_spacing
    start_x = center_x - total_steps_width/2 + step_width/2
    
    steps = [
        ('1. Baseline', 'f(x)', 'LLM Call'),
        ('2. Perturb', 'f(x+δ)', 'Challenge'),
        ('3. Delta', 'Δy = f(x+δ)-f(x)', 'Fingerprint'),
        ('4. Causal', 'Graph', 'AI Reasoning'),
        ('5. Spectral', '8-dim Vector', 'Features'),
    ]
    
    for i, (title, content, subtitle) in enumerate(steps):
        x = start_x + i * (step_width + step_spacing)
        
        # 步骤框
        step_box = FancyBboxPatch((x - step_width/2, phase2_y-0.6), step_width, step_height,
                                  boxstyle="round,pad=0.02,rounding_size=0.08",
                                  facecolor='white', edgecolor=COLORS['secondary'], 
                                  linewidth=1.5)
        ax.add_patch(step_box)
        ax.text(x, phase2_y+0.5, title, ha='center', va='center', 
                fontsize=8, fontweight='bold', color=COLORS['secondary'])
        ax.text(x, phase2_y+0.0, content, ha='center', va='center', 
                fontsize=9, fontweight='bold', color=COLORS['dark'])
        ax.text(x, phase2_y-0.4, subtitle, ha='center', va='center', 
                fontsize=7, color='#7F8C8D')
        
        # 步骤间的箭头
        if i < len(steps) - 1:
            arrow_start = x + step_width/2 + 0.05
            arrow_end = x + step_width/2 + step_spacing - 0.05
            ax.annotate('', xy=(arrow_end, phase2_y+0.1), xytext=(arrow_start, phase2_y+0.1),
                       arrowprops=dict(arrowstyle='->', color=COLORS['secondary'], lw=1.5))
    
    # 三层验证标注 - 居中
    validation_text = 'Three-Layer Verification: ① Intervention Response → ② Spectral Analysis → ③ Global Fingerprint'
    ax.text(center_x, phase2_y-1.0, validation_text, ha='center', va='center', 
            fontsize=8, style='italic', color='#7F8C8D',
            bbox=dict(boxstyle='round,pad=0.3', facecolor='#FFF9E6', edgecolor='#F39C12', alpha=0.7))
    
    # ===== Phase 3: Consensus (居中) =====
    phase3_y = 1.8
    
    # 标题 - 居中
    ax.text(center_x, phase3_y+0.7, 'PHASE 3', ha='center', va='center', 
            fontsize=10, fontweight='bold', color=COLORS['success'])
    ax.text(center_x, phase3_y+0.3, 'Consensus', ha='center', va='center', 
            fontsize=12, fontweight='bold', color=COLORS['dark'])
    
    # 共识框 - 居中
    consensus_width = 8
    consensus_box = FancyBboxPatch((center_x - consensus_width/2, phase3_y-0.5), consensus_width, 1.0,
                                   boxstyle="round,pad=0.03,rounding_size=0.1",
                                   facecolor=COLORS['phase2'], edgecolor=COLORS['success'], 
                                   linewidth=2)
    ax.add_patch(consensus_box)
    
    # 共识步骤 - 均匀分布
    consensus_steps = ['Collect', 'Spectral', 'Fingerprint', 'Cluster', 'Outlier']
    consensus_subs = ['Responses', 'Global', 'Generation', 'Cosine', 'Detection']
    
    step_spacing_3 = consensus_width / len(consensus_steps)
    start_x_3 = center_x - consensus_width/2 + step_spacing_3/2
    
    for i, (step, sub) in enumerate(zip(consensus_steps, consensus_subs)):
        x = start_x_3 + i * step_spacing_3
        ax.text(x, phase3_y+0.15, step, ha='center', va='center', 
                fontsize=8, fontweight='bold', color=COLORS['dark'])
        ax.text(x, phase3_y-0.2, sub, ha='center', va='center', 
                fontsize=6, color='#7F8C8D')
        if i < len(consensus_steps) - 1:
            ax.text(x + step_spacing_3/2, phase3_y, '→', ha='center', va='center', 
                    fontsize=10, color=COLORS['success'])
    
    # ===== Output (居中) =====
    output_y = 0.4
    
    output_width = 6
    output_box = FancyBboxPatch((center_x - output_width/2, output_y-0.25), output_width, 0.5,
                                boxstyle="round,pad=0.02,rounding_size=0.1",
                                facecolor=COLORS['phase3'], edgecolor=COLORS['accent'], 
                                linewidth=2)
    ax.add_patch(output_box)
    ax.text(center_x, output_y, 'Output: Consensus Value | Accuracy | Convergence Time | Byzantine Detection', 
            ha='center', va='center', fontsize=9, fontweight='bold', color=COLORS['dark'])
    
    # ===== 连接箭头 (垂直居中) =====
    # Phase 1 -> Phase 2
    ax.annotate('', xy=(center_x, phase2_y+1.8), xytext=(center_x, phase1_y-0.5),
               arrowprops=dict(arrowstyle='->', color=COLORS['primary'], lw=2))
    
    # Phase 2 -> Phase 3
    ax.annotate('', xy=(center_x, phase3_y+0.5), xytext=(center_x, phase2_y-1.6),
               arrowprops=dict(arrowstyle='->', color=COLORS['primary'], lw=2))
    
    # Phase 3 -> Output
    ax.annotate('', xy=(center_x, output_y+0.25), xytext=(center_x, phase3_y-0.5),
               arrowprops=dict(arrowstyle='->', color=COLORS['primary'], lw=2))
    
    # ===== 右侧说明框 (保持右侧) =====
    features_y = 5.0
    feature_box = FancyBboxPatch((11.5, features_y-0.8), 2, 1.8,
                                 boxstyle="round,pad=0.03,rounding_size=0.1",
                                 facecolor='#FEF9E7', edgecolor='#F4D03F', 
                                 linewidth=1.5)
    ax.add_patch(feature_box)
    ax.text(12.5, features_y+0.7, 'Key Features', ha='center', va='center', 
            fontsize=10, fontweight='bold', color=COLORS['dark'])
    
    features = [
        '• Non-linear Locking',
        '• High-Dim Defense',
        '• Model Heterogeneity',
        '• Statistical Security',
    ]
    for i, feat in enumerate(features):
        ax.text(11.7, features_y+0.3-i*0.35, feat, ha='left', va='center', 
                fontsize=8, color=COLORS['dark'])
    
    plt.tight_layout()
    plt.savefig('overall_pipeline_v3.png', dpi=300, bbox_inches='tight', facecolor='white')
    print("Saved: overall_pipeline_v3.png")
    plt.close()


if __name__ == '__main__':
    print("=" * 60)
    print("绘制居中对齐版 Multi-Agent Oracle Pipeline")
    print("=" * 60)
    print()
    
    draw_centered_pipeline()
    
    print()
    print("✅ 图表生成完成!")
    print("文件: overall_pipeline_v3.png")
