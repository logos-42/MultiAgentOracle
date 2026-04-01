#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
将 multi_agent_comparison_detailed.png 拆分成4个独立的子图
"""

import matplotlib.pyplot as plt
import numpy as np
from matplotlib import rcParams

# Set style
plt.style.use('seaborn-v0_8-whitegrid')
rcParams['font.family'] = 'DejaVu Sans'
rcParams['font.size'] = 11

def split_multi_agent_comparison():
    """将组合图拆分成4个独立的图"""
    
    methods = ['CFO', 'DAgger', 'Aggregation', 'Trust-aware', 'Weighted\nVoting']
    byzantine_tolerance = [40, 0, 30, 35, 20]  # %
    accuracy = [94.08, 0, 87.5, 90, 82.5]  # % (0 for DAgger as it depends on annotation)
    agent_count = [10, 100, 20, 15, 100]  # Unrestricted shown as 100
    
    # Consensus convergence time (seconds)
    consensus_time = [3.5, 0.5, 0.2, 1.5, 0.3]  # seconds total
    
    colors = ['#2E86AB', '#A23B72', '#F18F01', '#C73E1D', '#3B1F2B']
    
    # ==================== 图1: Byzantine Fault Tolerance ====================
    fig1, ax1 = plt.subplots(figsize=(8, 6))
    bars1 = ax1.bar(methods, byzantine_tolerance, color=colors, alpha=0.8, edgecolor='black', linewidth=1.2)
    ax1.set_ylabel('Byzantine Fault Tolerance (%)', fontsize=12, fontweight='bold')
    ax1.set_title('Byzantine Fault Tolerance Comparison', fontsize=14, fontweight='bold')
    ax1.set_ylim(0, 50)
    
    for bar in bars1:
        height = bar.get_height()
        if height > 0:
            ax1.text(bar.get_x() + bar.get_width()/2., height,
                    f'{height:.0f}%',
                    ha='center', va='bottom', fontsize=11, fontweight='bold')
        else:
            ax1.text(bar.get_x() + bar.get_width()/2., 2,
                    'N/A',
                    ha='center', va='bottom', fontsize=9, style='italic', color='gray')
    
    ax1.grid(True, alpha=0.3, axis='y')
    plt.tight_layout()
    plt.savefig('multi_agent_1_byzantine_tolerance.png', dpi=300, bbox_inches='tight')
    print("Saved: multi_agent_1_byzantine_tolerance.png")
    plt.close()
    
    # ==================== 图2: Consensus Accuracy ====================
    fig2, ax2 = plt.subplots(figsize=(8, 6))
    bars2 = ax2.bar(methods, accuracy, color=colors, alpha=0.8, edgecolor='black', linewidth=1.2)
    ax2.set_ylabel('Consensus Accuracy (%)', fontsize=12, fontweight='bold')
    ax2.set_title('Consensus Accuracy Comparison', fontsize=14, fontweight='bold')
    ax2.set_ylim(0, 100)
    
    for bar in bars2:
        height = bar.get_height()
        if height > 0:
            ax2.text(bar.get_x() + bar.get_width()/2., height,
                    f'{height:.1f}%',
                    ha='center', va='bottom', fontsize=11, fontweight='bold')
        else:
            ax2.text(bar.get_x() + bar.get_width()/2., 5,
                    'Annotation\nDependent',
                    ha='center', va='bottom', fontsize=8, style='italic', color='gray')
    
    ax2.grid(True, alpha=0.3, axis='y')
    plt.tight_layout()
    plt.savefig('multi_agent_2_accuracy.png', dpi=300, bbox_inches='tight')
    print("Saved: multi_agent_2_accuracy.png")
    plt.close()
    
    # ==================== 图3: Consensus Convergence Time ====================
    fig3, ax3 = plt.subplots(figsize=(8, 6))
    bars3 = ax3.bar(methods, consensus_time, color=colors, alpha=0.8, edgecolor='black', linewidth=1.2)
    ax3.set_ylabel('Consensus Convergence Time (seconds)', fontsize=12, fontweight='bold')
    ax3.set_title('Consensus Algorithm Convergence Time', fontsize=14, fontweight='bold')
    ax3.set_ylim(0, 5)
    
    for bar in bars3:
        height = bar.get_height()
        ax3.text(bar.get_x() + bar.get_width()/2., height,
                f'{height:.1f}s',
                ha='center', va='bottom', fontsize=11, fontweight='bold')
    
    ax3.grid(True, alpha=0.3, axis='y')
    plt.tight_layout()
    plt.savefig('multi_agent_3_convergence_time.png', dpi=300, bbox_inches='tight')
    print("Saved: multi_agent_3_convergence_time.png")
    plt.close()
    
    # ==================== 图4: Scalability ====================
    fig4, ax4 = plt.subplots(figsize=(8, 6))
    bars4 = ax4.bar(methods, agent_count, color=colors, alpha=0.8, edgecolor='black', linewidth=1.2)
    ax4.set_ylabel('Number of Agents Supported', fontsize=12, fontweight='bold')
    ax4.set_title('Scalability (Agents Supported)', fontsize=14, fontweight='bold')
    ax4.set_yscale('log')
    ax4.set_ylim(5, 200)
    
    for bar in bars4:
        height = bar.get_height()
        if height >= 100:
            label = 'Unlimited'
        else:
            label = f'{int(height)}'
        ax4.text(bar.get_x() + bar.get_width()/2., height,
                label,
                ha='center', va='bottom', fontsize=11, fontweight='bold')
    
    ax4.grid(True, alpha=0.3, axis='y')
    plt.tight_layout()
    plt.savefig('multi_agent_4_scalability.png', dpi=300, bbox_inches='tight')
    print("Saved: multi_agent_4_scalability.png")
    plt.close()
    
    print("\n✅ 所有图片已拆分完成!")
    print("生成的文件:")
    print("  1. multi_agent__tolerance.png -1_byzantine 拜占庭容错")
    print("  2. multi_agent_2_accuracy.png - 共识精度")
    print("  3. multi_agent_3_convergence_time.png - 共识收敛时间")
    print("  4. multi_agent_4_scalability.png - 可扩展性")

if __name__ == '__main__':
    print("=" * 60)
    print("拆分 multi_agent_comparison_detailed.png")
    print("=" * 60)
    print()
    
    split_multi_agent_comparison()
