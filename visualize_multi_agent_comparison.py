#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Multi-Agent Consensus Methods Comparison
"""

import matplotlib.pyplot as plt
import numpy as np
from matplotlib import rcParams

# Set style
plt.style.use('seaborn-v0_8-whitegrid')
rcParams['font.family'] = 'DejaVu Sans'
rcParams['font.size'] = 11

def plot_multi_agent_comparison():
    """Comparison with Multi-Agent Consensus Methods"""
    
    methods = ['CFO', 'DAgger', 'Aggregation', 'Trust-aware', 'Weighted\nVoting']
    byzantine_tolerance = [40, 0, 30, 35, 20]  # %
    accuracy = [94.08, 0, 87.5, 90, 82.5]  # % (0 for DAgger as it depends on annotation)
    agent_count = [10, 100, 20, 15, 100]  # Unrestricted shown as 100
    
    # Time per agent (seconds) - consensus algorithm processing time per agent
    # CFO: ~3.5s total / 10 agents = 0.35s per agent
    # Others estimated based on their reported speed
    time_per_agent = [0.35, 0.05, 0.02, 0.15, 0.03]  # seconds per agent
    
    # Create figure with 4 subplots in 2x2 layout
    fig, axes = plt.subplots(2, 2, figsize=(14, 10))
    
    colors = ['#2E86AB', '#A23B72', '#F18F01', '#C73E1D', '#3B1F2B']
    
    # Subplot 1: Byzantine Fault Tolerance
    ax1 = axes[0, 0]
    bars1 = ax1.bar(methods, byzantine_tolerance, color=colors, alpha=0.8, edgecolor='black', linewidth=1.2)
    ax1.set_ylabel('Byzantine Fault Tolerance (%)', fontsize=11, fontweight='bold')
    ax1.set_title('(a) Byzantine Fault Tolerance', fontsize=12, fontweight='bold')
    ax1.set_ylim(0, 50)
    
    for bar in bars1:
        height = bar.get_height()
        if height > 0:
            ax1.text(bar.get_x() + bar.get_width()/2., height,
                    f'{height:.0f}%',
                    ha='center', va='bottom', fontsize=10, fontweight='bold')
        else:
            ax1.text(bar.get_x() + bar.get_width()/2., 2,
                    'N/A',
                    ha='center', va='bottom', fontsize=9, style='italic', color='gray')
    
    ax1.grid(True, alpha=0.3, axis='y')
    
    # Subplot 2: Consensus Accuracy
    ax2 = axes[0, 1]
    bars2 = ax2.bar(methods, accuracy, color=colors, alpha=0.8, edgecolor='black', linewidth=1.2)
    ax2.set_ylabel('Consensus Accuracy (%)', fontsize=11, fontweight='bold')
    ax2.set_title('(b) Consensus Accuracy', fontsize=12, fontweight='bold')
    ax2.set_ylim(0, 100)
    
    for bar in bars2:
        height = bar.get_height()
        if height > 0:
            ax2.text(bar.get_x() + bar.get_width()/2., height,
                    f'{height:.1f}%',
                    ha='center', va='bottom', fontsize=10, fontweight='bold')
        else:
            ax2.text(bar.get_x() + bar.get_width()/2., 5,
                    'Annotation\nDependent',
                    ha='center', va='bottom', fontsize=8, style='italic', color='gray')
    
    ax2.grid(True, alpha=0.3, axis='y')
    
    # Subplot 3: Time Per Agent
    ax3 = axes[1, 0]
    bars3 = ax3.bar(methods, time_per_agent, color=colors, alpha=0.8, edgecolor='black', linewidth=1.2)
    ax3.set_ylabel('Consensus Time Per Agent (seconds)', fontsize=11, fontweight='bold')
    ax3.set_title('(c) Consensus Algorithm Time Per Agent', fontsize=12, fontweight='bold')
    ax3.set_ylim(0, 0.5)
    
    for bar in bars3:
        height = bar.get_height()
        ax3.text(bar.get_x() + bar.get_width()/2., height,
                f'{height:.2f}s',
                ha='center', va='bottom', fontsize=10, fontweight='bold')
    
    ax3.grid(True, alpha=0.3, axis='y')
    
    # Subplot 4: Number of Agents Supported
    ax4 = axes[1, 1]
    bars4 = ax4.bar(methods, agent_count, color=colors, alpha=0.8, edgecolor='black', linewidth=1.2)
    ax4.set_ylabel('Number of Agents Supported', fontsize=11, fontweight='bold')
    ax4.set_title('(d) Scalability (Agents Supported)', fontsize=12, fontweight='bold')
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
                ha='center', va='bottom', fontsize=10, fontweight='bold')
    
    ax4.grid(True, alpha=0.3, axis='y')
    
    # Overall title
    fig.suptitle('Comparison with Multi-Agent Consensus Methods\n', 
                 fontsize=14, fontweight='bold', y=1.02)
    
    plt.tight_layout()
    plt.savefig('multi_agent_comparison_detailed.png', dpi=300, bbox_inches='tight')
    print("Saved: multi_agent_comparison_detailed.png")
    plt.close()
    
    # Create comparison table
    fig, ax = plt.subplots(figsize=(14, 5))
    ax.axis('off')
    
    table_data = [
        ['Method', 'Agents', 'Byzantine\nTolerance', 'Accuracy', 'Time Per\nAgent', 'Main Limitation'],
        ['CFO', '10', '40%', '94.08%', '0.35s', 'Requires LLM calls'],
        ['DAgger [Ross et al., 2011]', 'Unlimited', 'N/A', 'Annotation\nDependent', '~0.05s', 'Requires expert demonstrations'],
        ['Aggregation [Chen et al., 2020]', '20', '30%', '85-90%', '~0.02s', 'Simple averaging'],
        ['Trust-aware [Li et al., 2021]', '15', '35%', '88-92%', '~0.15s', 'Requires reputation accumulation'],
        ['Weighted Voting', 'Unlimited', '20%', '80-85%', '~0.03s', 'Weight design difficulty'],
    ]
    
    table = ax.table(cellText=table_data, cellLoc='center', loc='center',
                    colWidths=[0.20, 0.12, 0.15, 0.12, 0.12, 0.29])
    
    table.auto_set_font_size(False)
    table.set_fontsize(10)
    table.scale(1, 2.2)
    
    # Style header row
    for i in range(6):
        cell = table[(0, i)]
        cell.set_facecolor('#2E86AB')
        cell.set_text_props(weight='bold', color='white')
    
    # Highlight CFO
    for i in range(6):
        cell = table[(1, i)]
        cell.set_facecolor('#E8F4F8')
        cell.set_text_props(weight='bold')
    
    # Style other rows
    for i in range(2, 6):
        for j in range(6):
            cell = table[(i, j)]
            if i % 2 == 0:
                cell.set_facecolor('#F5F5F5')
    
    ax.set_title('Detailed Comparison of Multi-Agent Consensus Methods\n', 
                fontsize=14, fontweight='bold', pad=20)
    
    plt.tight_layout()
    plt.savefig('multi_agent_comparison_table.png', dpi=300, bbox_inches='tight')
    print("Saved: multi_agent_comparison_table.png")
    plt.close()

if __name__ == '__main__':
    print("=" * 60)
    print("Multi-Agent Consensus Methods Comparison")
    print("=" * 60)
    print()
    
    plot_multi_agent_comparison()
    
    print()
    print("=" * 60)
    print("All figures generated successfully!")
    print("=" * 60)
    print()
    print("Generated files:")
    print("  1. multi_agent_comparison_detailed.png - Detailed comparison charts")
    print("  2. multi_agent_comparison_table.png - Comparison table")
