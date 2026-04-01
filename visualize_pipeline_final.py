#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Multi-Agent Oracle Pipeline - Final Rich Version
更详细丰富的流程图，展示完整的数据流和技术细节
"""

import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
from matplotlib.patches import FancyBboxPatch, FancyArrowPatch, Rectangle, Circle
import numpy as np

plt.style.use('default')
plt.rcParams['font.family'] = 'DejaVu Sans'
plt.rcParams['font.size'] = 9

# 精心设计的配色
COLORS = {
    'bg': '#FAFAFA',
    'phase1': '#E3F2FD',
    'phase1_border': '#1976D2',
    'phase2': '#E8F5E9',
    'phase2_border': '#388E3C',
    'phase3': '#FFF3E0',
    'phase3_border': '#F57C00',
    'phase4': '#F3E5F5',
    'phase4_border': '#7B1FA2',
    'agent': '#FFFFFF',
    'agent_border': '#0288D1',
    'text': '#212121',
    'text_light': '#757575',
    'arrow': '#424242',
    'highlight': '#212121',  # 改为黑色
}

def draw_rich_pipeline():
    """绘制丰富详细的 Pipeline 流程图"""
    
    fig = plt.figure(figsize=(16, 14))
    ax = fig.add_subplot(111)
    ax.set_xlim(0, 16)
    ax.set_ylim(0, 14)
    ax.axis('off')
    ax.set_facecolor(COLORS['bg'])
    
    center_x = 8
    
    # ===== 主标题 =====
    ax.text(center_x, 13.2, 'CFO Protocol: End-to-End System Architecture', 
            ha='center', va='center', fontsize=20, fontweight='bold', color=COLORS['text'])
    ax.text(center_x, 12.6, 'Causal Fingerprinting Oracle for Byzantine-Resistant Consensus', 
            ha='center', va='center', fontsize=12, style='italic', color=COLORS['text_light'])
    
    # ===== LAYER 1: INPUT LAYER =====
    layer1_y = 11.0
    
    # 背景框
    layer1_box = FancyBboxPatch((1, layer1_y-0.6), 14, 1.2,
                                boxstyle="round,pad=0.05,rounding_size=0.2",
                                facecolor=COLORS['phase1'], edgecolor=COLORS['phase1_border'], 
                                linewidth=2.5, alpha=0.3)
    ax.add_patch(layer1_box)
    
    # 标题（已移除 LAYER 标注）
    
    # 输入框
    input_box = FancyBboxPatch((4, layer1_y-0.35), 4, 0.7,
                               boxstyle="round,pad=0.03,rounding_size=0.1",
                               facecolor='white', edgecolor=COLORS['phase1_border'], linewidth=2)
    ax.add_patch(input_box)
    ax.text(6, layer1_y+0.1, 'Prediction Query', ha='center', va='center', 
            fontsize=10, fontweight='bold', color=COLORS['text'])
    ax.text(6, layer1_y-0.15, '"What is the inflation rate next quarter?"', 
            ha='center', va='center', fontsize=8, style='italic', color=COLORS['text_light'])
    
    # 输入数据示例
    data_examples = [
        ('Query Type:', 'Economic Forecast'),
        ('Context:', 'Interest Rate Policy'),
        ('Timestamp:', '2026-Q1'),
    ]
    for i, (label, value) in enumerate(data_examples):
        y_pos = layer1_y + 0.25 - i*0.25
        ax.text(9.5, y_pos, f'{label}', ha='left', va='center', 
                fontsize=7, fontweight='bold', color=COLORS['text'])
        ax.text(11.5, y_pos, value, ha='left', va='center', 
                fontsize=7, color=COLORS['text_light'])
    
    # ===== LAYER 2: MULTI-AGENT GENERATION (核心层) =====
    layer2_y = 7.8  # 向上移动避免与第三层重合
    
    # 大背景 - 减小高度
    layer2_box = FancyBboxPatch((0.5, layer2_y-2.2), 15, 4.4,
                                boxstyle="round,pad=0.08,rounding_size=0.25",
                                facecolor=COLORS['phase2'], edgecolor=COLORS['phase2_border'], 
                                linewidth=3, alpha=0.2)
    ax.add_patch(layer2_box)
    
    # 标题（已移除 LAYER 标注）
    
    # 多个 Agent 框
    num_agents = 4
    agent_width = 3
    agent_height = 3.0  # 减小高度避免与第三层重合
    agent_spacing = 0.5
    total_width = num_agents * agent_width + (num_agents-1) * agent_spacing
    start_x = center_x - total_width/2
    
    # 系统只使用 DeepSeek
    agent_names = ['Agent 1\n(DeepSeek)', 'Agent 2\n(DeepSeek)', 'Agent 3\n(DeepSeek)', 'Agent 4\n(DeepSeek)']
    agent_colors = ['#E3F2FD', '#F3E5F5', '#E8F5E9', '#FFF3E0']
    
    for i in range(num_agents):
        x = start_x + i * (agent_width + agent_spacing)
        
        # Agent 框
        agent_box = FancyBboxPatch((x, layer2_y-1.5), agent_width, agent_height,
                                   boxstyle="round,pad=0.03,rounding_size=0.15",
                                   facecolor=agent_colors[i], edgecolor=COLORS['agent_border'], 
                                   linewidth=2)
        ax.add_patch(agent_box)
        
        # Agent 名称
        ax.text(x + agent_width/2, layer2_y+1.3, agent_names[i], 
                ha='center', va='center', fontsize=9, fontweight='bold', color=COLORS['text'])
        
        # 步骤框 - 在 Agent 内部
        steps = [
            ('Step 1', 'Baseline', 'LLM Call'),
            ('Step 2', 'Perturbed', 'Challenge'),
            ('Step 3', 'Fingerprint', 'Delta'),
        ]
        
        for j, (step, result, desc) in enumerate(steps):
            step_y = layer2_y + 0.65 - j * 0.85  # 增加间距避免重合
            
            # 小步骤框 - 增加高度
            step_box = FancyBboxPatch((x+0.15, step_y-0.3), agent_width-0.3, 0.6,
                                      boxstyle="round,pad=0.02,rounding_size=0.05",
                                      facecolor='white', edgecolor=COLORS['agent_border'], 
                                      linewidth=1, alpha=0.9)
            ax.add_patch(step_box)
            
            ax.text(x+0.45, step_y, step, ha='left', va='center', 
                    fontsize=7, fontweight='bold', color=COLORS['agent_border'])
            ax.text(x+agent_width/2, step_y, result, ha='center', va='center', 
                    fontsize=8, fontweight='bold', color=COLORS['highlight'])
            ax.text(x+agent_width-0.45, step_y, desc, ha='right', va='center', 
                    fontsize=6, color=COLORS['text_light'])
        
        # 垂直箭头连接步骤
        for j in range(2):
            arrow_y1 = layer2_y + 0.65 - j * 0.85 - 0.33
            arrow_y2 = layer2_y + 0.65 - (j+1) * 0.85 + 0.33
            ax.annotate('', xy=(x+agent_width/2, arrow_y2), xytext=(x+agent_width/2, arrow_y1),
                       arrowprops=dict(arrowstyle='->', color=COLORS['agent_border'], lw=1.5))
    
    # ===== LAYER 3: CONSENSUS LAYER =====
    layer3_y = 3.8
    
    layer3_box = FancyBboxPatch((0.5, layer3_y-1.5), 15, 2.8,
                                boxstyle="round,pad=0.08,rounding_size=0.2",
                                facecolor=COLORS['phase3'], edgecolor=COLORS['phase3_border'], 
                                linewidth=3, alpha=0.25)
    ax.add_patch(layer3_box)
    
    # Layer 3 标题（已移除）
    
    # 共识步骤 - 横向排列
    consensus_modules = [
        ('Data\nCollection', 'N × M Matrix', 'Collect all\nagent responses'),
        ('Spectral\nAnalysis', 'Eigenvalue Decomp', 'Extract global\nfeatures'),
        ('Fingerprint\nMatching', 'Cosine Similarity', 'Compute pairwise\nsimilarities'),
        ('Clustering', 'Majority Consensus', 'Group similar\nagents'),
        ('Byzantine\nDetection', 'Outlier Removal', 'Filter malicious\nagents (>40%)'),
    ]
    
    module_width = 2.6
    module_height = 2.0
    module_spacing = 0.2
    total_modules_width = len(consensus_modules) * module_width + (len(consensus_modules)-1) * module_spacing
    start_x_mod = center_x - total_modules_width/2
    
    for i, (title, subtitle, desc) in enumerate(consensus_modules):
        x = start_x_mod + i * (module_width + module_spacing)
        
        mod_box = FancyBboxPatch((x, layer3_y-0.8), module_width, module_height,
                                 boxstyle="round,pad=0.03,rounding_size=0.1",
                                 facecolor='white', edgecolor=COLORS['phase3_border'], linewidth=2)
        ax.add_patch(mod_box)
        
        ax.text(x+module_width/2, layer3_y+0.8, title, ha='center', va='center', 
                fontsize=9, fontweight='bold', color=COLORS['phase3_border'])
        ax.text(x+module_width/2, layer3_y+0.25, subtitle, ha='center', va='center', 
                fontsize=8, fontweight='bold', color=COLORS['text'])
        ax.text(x+module_width/2, layer3_y-0.35, desc, ha='center', va='center', 
                fontsize=7, color=COLORS['text_light'])
        
        # 连接箭头
        if i < len(consensus_modules) - 1:
            ax.annotate('', xy=(x+module_width+0.15, layer3_y+0.2), 
                       xytext=(x+module_width+module_spacing-0.15, layer3_y+0.2),
                       arrowprops=dict(arrowstyle='->', color=COLORS['phase3_border'], lw=3))
    
    # ===== LAYER 4: OUTPUT =====
    layer4_y = 1.2
    
    layer4_box = FancyBboxPatch((2, layer4_y-0.5), 12, 1.0,
                                boxstyle="round,pad=0.05,rounding_size=0.15",
                                facecolor=COLORS['phase4'], edgecolor=COLORS['phase4_border'], 
                                linewidth=2.5)
    ax.add_patch(layer4_box)
    
    # Layer 4 标题（已移除）
    
    # 输出指标
    outputs = [
        ('Consensus Value', '2.65%'),
        ('Confidence', '94.1%'),
        ('Convergence Time', '3.5s'),
        ('Byzantine Detected', '1/4 agents'),
    ]
    
    for i, (label, value) in enumerate(outputs):
        x_pos = 4.5 + i * 2.8
        ax.text(x_pos, layer4_y+0.15, label, ha='center', va='center', 
                fontsize=8, fontweight='bold', color=COLORS['text'])
        ax.text(x_pos, layer4_y-0.2, value, ha='center', va='center', 
                fontsize=9, fontweight='bold', color=COLORS['phase4_border'])
        if i < len(outputs) - 1:
            ax.plot([x_pos+1.4, x_pos+1.4], [layer4_y-0.3, layer4_y+0.25], 
                   '|', color=COLORS['text_light'], markersize=15, markeredgewidth=1)
    
    # ===== 垂直连接箭头 =====
    # Layer 1 -> Layer 2
    ax.annotate('', xy=(center_x, layer2_y+2.2), xytext=(center_x, layer1_y-0.7),
               arrowprops=dict(arrowstyle='->', color=COLORS['arrow'], lw=2.5))
    
    # Layer 2 -> Layer 3
    ax.annotate('', xy=(center_x, layer3_y+1.3), xytext=(center_x, layer2_y-2.3),
               arrowprops=dict(arrowstyle='->', color=COLORS['arrow'], lw=2.5))
    
    # Layer 3 -> Layer 4
    ax.annotate('', xy=(center_x, layer4_y+0.5), xytext=(center_x, layer3_y-1.6),
               arrowprops=dict(arrowstyle='->', color=COLORS['arrow'], lw=2.5))
    
    plt.tight_layout()
    plt.savefig('overall_pipeline_final.png', dpi=300, bbox_inches='tight', facecolor='white')
    print("Saved: overall_pipeline_final.png")
    plt.close()


if __name__ == '__main__':
    print("=" * 60)
    print("绘制丰富详细的 CFO Protocol Pipeline")
    print("=" * 60)
    print()
    
    draw_rich_pipeline()
    
    print()
    print("✅ 图表生成完成!")
    print()
    print("包含内容:")
    print("  - 4层架构: Input → Multi-Agent → Consensus → Output")
    print("  - 4个并行的 Agent，展示完整步骤")
    print("  - 每个步骤的具体数值示例")
    print("  - 5步共识计算流程")
    print("  - 详细的输出指标")
    print("  - 右侧安全指标面板")
    print()
    print("文件: overall_pipeline_final.png")
