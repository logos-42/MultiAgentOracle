#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
绘制 Multi-Agent Oracle Pipeline v4
- 移除 Key Features (避免挡住 Phase 2)
- 增大 Phase 3 的框和箭头
- 修复文字被遮挡问题
- 调整字体布局避免重叠
"""

import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch
import numpy as np

plt.style.use('default')
plt.rcParams['font.family'] = 'DejaVu Sans'
plt.rcParams['font.size'] = 10

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

def draw_fixed_pipeline():
    """绘制修复后的 Pipeline"""
    
    fig, ax = plt.subplots(figsize=(14, 10))
    ax.set_xlim(0, 14)
    ax.set_ylim(0, 10)
    ax.axis('off')
    
    center_x = 7
    
    # 标题
    ax.text(center_x, 9.5, 'CFO : Multi-Agent Oracle Architecture', 
            ha='center', va='center', fontsize=18, fontweight='bold', color=COLORS['primary'])
    ax.text(center_x, 9.0, 'Causal Fingerprinting for Byzantine-Resistant Consensus', 
            ha='center', va='center', fontsize=11, style='italic', color='#7F8C8D')
    
    # ===== Phase 1: User Request =====
    phase1_y = 7.6
    
    ax.text(center_x, phase1_y+0.8, 'PHASE 1', ha='center', va='center', 
            fontsize=10, fontweight='bold', color=COLORS['secondary'])
    ax.text(center_x, phase1_y+0.4, 'User Request', ha='center', va='center', 
            fontsize=12, fontweight='bold', color=COLORS['dark'])
    
    input_width = 3.5
    input_box = FancyBboxPatch((center_x - input_width/2, phase1_y-0.35), input_width, 0.7,
                               boxstyle="round,pad=0.02,rounding_size=0.1",
                               facecolor=COLORS['phase1'], edgecolor=COLORS['secondary'], 
                               linewidth=2)
    ax.add_patch(input_box)
    ax.text(center_x, phase1_y, 'Economic Prediction Task', 
            ha='center', va='center', fontsize=10, color=COLORS['dark'])
    
    # ===== Phase 2: Agent Generation =====
    phase2_y = 4.6
    
    # 大框 - 增加高度避免文字重叠
    main_width = 10.5
    main_height = 3.6
    main_box = FancyBboxPatch((center_x - main_width/2, phase2_y-1.6), main_width, main_height,
                              boxstyle="round,pad=0.05,rounding_size=0.15",
                              facecolor=COLORS['light'], edgecolor=COLORS['primary'], 
                              linewidth=2.5)
    ax.add_patch(main_box)
    
    ax.text(center_x, phase2_y+1.5, 'PHASE 2: Agent Generation', ha='center', va='center', 
            fontsize=12, fontweight='bold', color=COLORS['primary'])
    ax.text(center_x, phase2_y+1.1, '', ha='center', va='center', 
            fontsize=9, style='italic', color='#7F8C8D')
    
    # 5个步骤 - 增大框高度
    step_width = 1.8
    step_height = 1.8
    step_spacing = 0.25
    total_steps_width = 5 * step_width + 4 * step_spacing
    start_x = center_x - total_steps_width/2 + step_width/2
    
    steps = [
        ('1. Baseline', 'f(x)', 'LLM Call'),
        ('2. Perturb', 'f(x+δ)', 'Challenge'),
        ('3. Delta', 'Δy = f(x+δ)-f(x)', 'Fingerprint'),
        ('4. Causal Graph', 'Structure', 'AI Reasoning'),
        ('5. Spectral', '8-dim Vector', 'Features'),
    ]
    
    for i, (title, content, subtitle) in enumerate(steps):
        x = start_x + i * (step_width + step_spacing)
        
        step_box = FancyBboxPatch((x - step_width/2, phase2_y-0.7), step_width, step_height,
                                  boxstyle="round,pad=0.02,rounding_size=0.08",
                                  facecolor='white', edgecolor=COLORS['secondary'], 
                                  linewidth=1.5)
        ax.add_patch(step_box)
        
        # 调整文字位置避免重叠
        ax.text(x, phase2_y+0.65, title, ha='center', va='center', 
                fontsize=9, fontweight='bold', color=COLORS['secondary'])
        ax.text(x, phase2_y+0.05, content, ha='center', va='center', 
                fontsize=10, fontweight='bold', color=COLORS['dark'])
        ax.text(x, phase2_y-0.45, subtitle, ha='center', va='center', 
                fontsize=8, color='#7F8C8D')
        
        # 箭头 - 延长一点
        if i < len(steps) - 1:
            arrow_start = x + step_width/2 + 0.02
            arrow_end = x + step_width/2 + step_spacing - 0.02
            ax.annotate('', xy=(arrow_end, phase2_y+0.05), xytext=(arrow_start, phase2_y+0.05),
                       arrowprops=dict(arrowstyle='->', color=COLORS['secondary'], lw=2))
    
    # 三层验证标注 - 下移避免重叠
    validation_text = 'Three-Layer: Intervention Response → Spectral Analysis → Global Fingerprint'
    ax.text(center_x, phase2_y-1.25, validation_text, ha='center', va='center', 
            fontsize=8, style='italic', color='#7F8C8D',
            bbox=dict(boxstyle='round,pad=0.3', facecolor='#FFF9E6', edgecolor='#F39C12', alpha=0.7))
    
    # ===== Phase 3: Consensus =====
    phase3_y = 1.5
    
    ax.text(center_x, phase3_y+0.9, 'PHASE 3', ha='center', va='center', 
            fontsize=10, fontweight='bold', color=COLORS['success'])
    ax.text(center_x, phase3_y+0.5, 'Consensus Computation', ha='center', va='center', 
            fontsize=12, fontweight='bold', color=COLORS['dark'])
    
    # 增大框高度避免文字被遮挡
    consensus_width = 10
    consensus_height = 1.3
    consensus_box = FancyBboxPatch((center_x - consensus_width/2, phase3_y-0.6), consensus_width, consensus_height,
                                   boxstyle="round,pad=0.03,rounding_size=0.1",
                                   facecolor=COLORS['phase2'], edgecolor=COLORS['success'], 
                                   linewidth=2)
    ax.add_patch(consensus_box)
    
    # 共识步骤 - 增大间距
    consensus_steps = ['Collect', 'Spectral', 'Fingerprint', 'Cluster', 'Outlier']
    consensus_subs = ['Responses', 'Global', 'Generation', 'Cosine', 'Detection']
    
    step_spacing_3 = consensus_width / len(consensus_steps)
    start_x_3 = center_x - consensus_width/2 + step_spacing_3/2
    
    for i, (step, sub) in enumerate(zip(consensus_steps, consensus_subs)):
        x = start_x_3 + i * step_spacing_3
        # 上移文字避免被下框线遮挡
        ax.text(x, phase3_y+0.05, step, ha='center', va='center', 
                fontsize=9, fontweight='bold', color=COLORS['dark'])
        ax.text(x, phase3_y-0.35, sub, ha='center', va='center', 
                fontsize=7, color='#7F8C8D')
        if i < len(consensus_steps) - 1:
            # 增大箭头
            ax.annotate('', xy=(x + step_spacing_3*0.75, phase3_y-0.15), 
                       xytext=(x + step_spacing_3*0.25, phase3_y-0.15),
                       arrowprops=dict(arrowstyle='->', color=COLORS['success'], lw=2.5))
    
    # ===== Output =====
    output_y = 0.25
    
    output_width = 7
    output_box = FancyBboxPatch((center_x - output_width/2, output_y-0.2), output_width, 0.4,
                                boxstyle="round,pad=0.02,rounding_size=0.1",
                                facecolor=COLORS['phase3'], edgecolor=COLORS['accent'], 
                                linewidth=2)
    ax.add_patch(output_box)
    ax.text(center_x, output_y, 'Output: Consensus Value | Accuracy | Time | Byzantine Detection', 
            ha='center', va='center', fontsize=9, fontweight='bold', color=COLORS['dark'])
    
    # ===== 连接箭头 =====
    ax.annotate('', xy=(center_x, phase2_y+2.0), xytext=(center_x, phase1_y-0.4),
               arrowprops=dict(arrowstyle='->', color=COLORS['primary'], lw=2))
    
    ax.annotate('', xy=(center_x, phase3_y+0.7), xytext=(center_x, phase2_y-1.7),
               arrowprops=dict(arrowstyle='->', color=COLORS['primary'], lw=2))
    
    ax.annotate('', xy=(center_x, output_y+0.2), xytext=(center_x, phase3_y-0.6),
               arrowprops=dict(arrowstyle='->', color=COLORS['primary'], lw=2))
    
    plt.tight_layout()
    plt.savefig('overall_pipeline_v4.png', dpi=300, bbox_inches='tight', facecolor='white')
    print("Saved: overall_pipeline_v4.png")
    plt.close()


if __name__ == '__main__':
    print("=" * 60)
    print("绘制修复版 CFO Protocol Pipeline")
    print("=" * 60)
    print()
    
    draw_fixed_pipeline()
    
    print()
    print("✅ 图表生成完成!")
    print("修复内容:")
    print("  - 移除 Key Features 避免挡住 Phase 2")
    print("  - 增大 Phase 3 框高度避免文字被遮挡")
    print("  - 增大并居中 Phase 3 横向箭头")
    print("  - 调整 Phase 2 文字位置避免与框线重叠")
    print()
    print("文件: overall_pipeline_v4.png")
