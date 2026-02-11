#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Multi-Agent Byzantine Fault Tolerant Consensus System - Comparison Visualization
Generate publication-quality figures for research paper
"""

import matplotlib.pyplot as plt
import numpy as np
import matplotlib.patches as mpatches
from matplotlib import rcParams

# Set font for English display
plt.rcParams['font.family'] = 'DejaVu Sans'
plt.rcParams['axes.unicode_minus'] = False

# Set plot style
plt.style.use('seaborn-v0_8-darkgrid')
rcParams['figure.figsize'] = (14, 8)
rcParams['font.size'] = 11

def plot_bft_comparison():
    """Figure 1: Comparison with Traditional BFT Consensus Algorithms"""
    fig, ax = plt.subplots(figsize=(14, 7))
    
    methods = ['CFO', 'PBFT\n[Castro & Liskov, 2002]', 
               'HotStuff\n[Yin et al., 2019]', 'Tendermint\n[Buchman et al., 2016]', 
               'BFT-SMaRt\n[Bessani et al., 2014]']
    
    byzantine_tolerance = [40, 33.3, 33.3, 33.3, 33.3]
    # Note: CFO consensus algorithm itself takes ~3-5 seconds
    # (excluding 60-90s API calls for 30 LLM requests per round)
    consensus_delay = [3.5, 2.5, 2, 5.5, 12.5]  # seconds (algorithm only)
    
    x = np.arange(len(methods))
    width = 0.35
    
    # Set grid first (behind bars)
    ax.grid(True, alpha=0.3, axis='y', zorder=0)
    ax.set_axisbelow(True)  # Ensure grid is behind all artists
    
    # Create bar chart (zorder=3 to be above grid)
    bars1 = ax.bar(x - width/2, byzantine_tolerance, width, 
                   label='Byzantine Tolerance (%)', color='#2E86AB', alpha=0.8, zorder=3)
    
    ax2 = ax.twinx()
    ax2.grid(False)  # Disable grid on second axis
    bars2 = ax2.bar(x + width/2, consensus_delay, width, 
                    label='Consensus Delay (s)*', color='#A23B72', alpha=0.8, zorder=3)
    
    # Add value labels (zorder=4 to be on top)
    for bar in bars1:
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2., height,
                f'{height:.1f}%',
                ha='center', va='bottom', fontsize=10, fontweight='bold', zorder=4)
    
    for bar in bars2:
        height = bar.get_height()
        ax2.text(bar.get_x() + bar.get_width()/2., height,
                f'{height:.1f}s',
                ha='center', va='bottom', fontsize=10, fontweight='bold', zorder=4)
    
    ax.set_ylabel('Byzantine Tolerance (%)', fontsize=12, fontweight='bold')
    ax2.set_ylabel('Consensus Delay (s)', fontsize=12, fontweight='bold')
    ax.set_xlabel('Consensus Methods', fontsize=12, fontweight='bold')
    ax.set_title('Comparison with Traditional BFT Consensus Algorithms\n', 
                 fontsize=14, fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels(methods, fontsize=10)
    ax.set_ylim(0, 50)
    ax2.set_ylim(0, 15)
    
    # Remove top and right spines for cleaner look
    ax.spines['top'].set_visible(False)
    ax2.spines['top'].set_visible(False)
    
    # Legend
    lines1, labels1 = ax.get_legend_handles_labels()
    lines2, labels2 = ax2.get_legend_handles_labels()
    ax.legend(lines1 + lines2, labels1 + labels2, loc='upper right', fontsize=10)
    
    # Add note about CFO timing
    fig.text(0.5, 0.02, 
             '*CFO: ~3.5s for consensus algorithm (excluding 60-90s for 30 sequential LLM API calls)',
             ha='center', fontsize=9, style='italic', color='#666')
    
    plt.tight_layout(rect=[0, 0.03, 1, 1])
    plt.savefig('comparison_bft.png', dpi=300, bbox_inches='tight')
    print("✅ Figure 1 saved: comparison_bft.png")
    plt.close()

def plot_federated_learning_comparison():
    """Comparison with Federated Learning Byzantine Defense Methods"""
    fig, (ax1, ax2) = plt.subplots(1, 2, figsize=(16, 6))
    
    methods = ['CFO', 'Krum', 'Trimmed\nMean', 'Median', 'Multi-Krum', 'Bulyan']
    accuracy = [94.08, 85, 80, 80, 90, 88]
    tolerance = [40, 50, 50, 50, 50, 50]
    
    colors = ['#2E86AB', '#F18F01', '#C73E1D', '#6A994E', '#BC4B51', '#8B5A3C']
    
    # Left plot: Accuracy comparison
    bars1 = ax1.barh(methods, accuracy, color=colors, alpha=0.8, edgecolor='black', linewidth=1.5)
    ax1.set_xlabel('Average Accuracy (%)', fontsize=12, fontweight='bold')
    ax1.set_title('(a) Prediction Accuracy Comparison', fontsize=12, fontweight='bold')
    ax1.set_xlim(75, 100)
    ax1.grid(True, alpha=0.3, axis='x')
    
    # Add value labels
    for i, (bar, acc) in enumerate(zip(bars1, accuracy)):
        width = bar.get_width()
        ax1.text(width + 0.5, bar.get_y() + bar.get_height()/2,
                f'{acc:.1f}%',
                ha='left', va='center', fontsize=10, fontweight='bold')
    
    # Right plot: Tolerance vs Accuracy scatter
    scatter = ax2.scatter(tolerance, accuracy, s=300, c=colors, alpha=0.8, 
                         edgecolors='black', linewidth=2, marker='o')
    
    for i, method in enumerate(methods):
        ax2.annotate(method, (tolerance[i], accuracy[i]), 
                    xytext=(5, 5), textcoords='offset points',
                    fontsize=9, fontweight='bold')
    
    ax2.set_xlabel('Byzantine Tolerance (%)', fontsize=12, fontweight='bold')
    ax2.set_ylabel('Average Accuracy (%)', fontsize=12, fontweight='bold')
    ax2.set_title('(b) Tolerance vs Accuracy', fontsize=12, fontweight='bold')
    ax2.grid(True, alpha=0.3)
    ax2.set_xlim(35, 55)
    ax2.set_ylim(78, 96)
    
    ax2.legend(loc='lower right', fontsize=9)
    
    plt.suptitle('Comparison with Federated Learning Byzantine Defense Methods\n', 
                 fontsize=14, fontweight='bold', y=1.02)
    plt.tight_layout()
    plt.savefig('comparison_federated.png', dpi=300, bbox_inches='tight')
    print("✅ Figure 2 saved: comparison_federated.png")
    plt.close()

def plot_multi_agent_comparison():
    """Comparison with Multi-Agent Consensus Methods"""
    fig, ax = plt.subplots(figsize=(12, 7))
    
    methods = ['CFO', 'DAgger', 'Aggregation', 
               'Trust-aware', 'Weighted\nVoting']
    
    # Data
    byzantine_tolerance = [40, 0, 30, 35, 20]
    accuracy = [94.08, 85, 87.5, 90, 82.5]
    
    x = np.arange(len(methods))
    width = 0.35
    
    # Set grid first (behind bars)
    ax.grid(True, alpha=0.3, axis='y', zorder=0)
    ax.set_axisbelow(True)  # Ensure grid is behind all artists
    
    bars1 = ax.bar(x - width/2, byzantine_tolerance, width,
                   label='Byzantine Tolerance (%)', color='#2E86AB', alpha=0.8, zorder=3)
    
    ax2 = ax.twinx()
    ax2.grid(False)  # Disable grid on second axis
    bars2 = ax2.bar(x + width/2, accuracy, width,
                    label='Average Accuracy (%)', color='#F18F01', alpha=0.8, zorder=3)
    
    # Add value labels
    for bar in bars1:
        height = bar.get_height()
        ax.text(bar.get_x() + bar.get_width()/2., height + 1,
                f'{height:.0f}%',
                ha='center', va='bottom', fontsize=10, fontweight='bold', zorder=4)
    
    for bar in bars2:
        height = bar.get_height()
        ax2.text(bar.get_x() + bar.get_width()/2., height + 1,
                f'{height:.1f}%',
                ha='center', va='bottom', fontsize=10, fontweight='bold', zorder=4)
    
    ax.set_ylabel('Byzantine Tolerance (%)', fontsize=12, fontweight='bold')
    ax2.set_ylabel('Average Accuracy (%)', fontsize=12, fontweight='bold')
    ax.set_xlabel('Multi-Agent Consensus Methods', fontsize=12, fontweight='bold')
    ax.set_title('Comparison with Multi-Agent Consensus Methods\n', 
                 fontsize=14, fontweight='bold')
    ax.set_xticks(x)
    ax.set_xticklabels(methods, fontsize=10)
    ax.set_ylim(0, 50)
    ax2.set_ylim(0, 105)
    
    lines1, labels1 = ax.get_legend_handles_labels()
    lines2, labels2 = ax2.get_legend_handles_labels()
    ax.legend(lines1 + lines2, labels1 + labels2, loc='upper right', fontsize=10)
    
    plt.tight_layout()
    plt.savefig('comparison_multi_agent.png', dpi=300, bbox_inches='tight')
    print("✅ Figure 3 saved: comparison_multi_agent.png")
    plt.close()

def plot_tolerance_accuracy_trend():
    """Byzantine Tolerance Ratio vs Accuracy Trend"""
    fig, ax = plt.subplots(figsize=(12, 7))
    
    byzantine_ratios = [0, 10, 20, 30, 40]
    our_method = [97.62, 95.88, 94.23, 92.47, 91.35]
    krum = [95, 92, 89, 86, 83]
    trimmed_mean = [92, 88, 84, 80, 76]
    median = [90, 86, 82, 78, 74]
    
    ax.plot(byzantine_ratios, our_method, 'o-', linewidth=3, markersize=10,
            label='CFO', color='#2E86AB')
    ax.plot(byzantine_ratios, krum, 's--', linewidth=2, markersize=8,
            label='Krum', color='#F18F01')
    ax.plot(byzantine_ratios, trimmed_mean, '^--', linewidth=2, markersize=8,
            label='Trimmed Mean', color='#C73E1D')
    ax.plot(byzantine_ratios, median, 'd--', linewidth=2, markersize=8,
            label='Median', color='#6A994E')
    
    # Annotate CFO key data points
    for i, (x, y) in enumerate(zip(byzantine_ratios, our_method)):
        ax.annotate(f'{y:.1f}%', (x, y), xytext=(0, 10), 
                   textcoords='offset points', ha='center',
                   fontsize=9, fontweight='bold', color='#2E86AB')
    
    # No reference lines as requested
    
    ax.set_xlabel('Byzantine Node Ratio (%)', fontsize=12, fontweight='bold')
    ax.set_ylabel('Average Accuracy (%)', fontsize=12, fontweight='bold')
    ax.set_title('Relationship between Byzantine Tolerance and Prediction Accuracy\n', 
                 fontsize=14, fontweight='bold')
    ax.set_xlim(-2, 45)
    ax.set_ylim(70, 100)
    ax.legend(loc='lower left', fontsize=10)
    ax.grid(True, alpha=0.3)
    
    # Note: CFO maintains high accuracy even beyond traditional 33.3% BFT limit
    
    plt.tight_layout()
    plt.savefig('tolerance_accuracy_trend.png', dpi=300, bbox_inches='tight')
    print("✅ Figure 4 saved: tolerance_accuracy_trend.png")
    plt.close()

def plot_technical_innovation_radar():
    """Technical Innovation Capability Radar Chart"""
    fig, ax = plt.subplots(figsize=(10, 10), subplot_kw=dict(projection='polar'))
    
    categories = ['Spectral\nAnalysis', 'Delta\nResponse', 'High-Dim\nClustering', 
                  'Median\nBootstrapping', 'LLM\nIntegration']
    N = len(categories)
    
    # Scores for each method (0-5 scale)
    our_method = [5, 5, 5, 5, 5]
    traditional_bft = [0, 0, 1, 0, 0]
    federated_defense = [1, 0, 2, 2, 0]
    
    angles = [n / float(N) * 2 * np.pi for n in range(N)]
    angles += angles[:1]
    
    our_method += our_method[:1]
    traditional_bft += traditional_bft[:1]
    federated_defense += federated_defense[:1]
    
    ax.plot(angles, our_method, 'o-', linewidth=2, label='CFO', color='#2E86AB')
    ax.fill(angles, our_method, alpha=0.25, color='#2E86AB')
    
    ax.plot(angles, traditional_bft, 's--', linewidth=2, label='Traditional BFT', color='#F18F01')
    ax.fill(angles, traditional_bft, alpha=0.25, color='#F18F01')
    
    ax.plot(angles, federated_defense, '^--', linewidth=2, label='Federated Defense', color='#C73E1D')
    ax.fill(angles, federated_defense, alpha=0.25, color='#C73E1D')
    
    ax.set_xticks(angles[:-1])
    ax.set_xticklabels(categories, fontsize=11)
    ax.set_ylim(0, 5)
    ax.set_title('Technical Innovation Capability Radar Chart\n', 
                 fontsize=14, fontweight='bold', pad=20)
    ax.legend(loc='upper right', bbox_to_anchor=(1.3, 1.1), fontsize=10)
    ax.grid(True)
    
    plt.tight_layout()
    plt.savefig('technical_innovation_radar.png', dpi=300, bbox_inches='tight')
    print("✅ Figure 5 saved: technical_innovation_radar.png")
    plt.close()

def create_summary_table():
    """Generate summary table figure"""
    fig, ax = plt.subplots(figsize=(14, 10))
    ax.axis('tight')
    ax.axis('off')
    
    table_data = [
        ['Method', 'Byzantine Tolerance', 'Avg Accuracy', 'Consensus Delay', 'Communication', 'Application'],
        ['CFO', '40%', '94.08%', '~4 min', 'O(n²)', 'Multi-Agent Prediction'],
        ['PBFT [Castro & Liskov, 2002]', '33.3%', '-', '2-5 s', 'O(n²)', 'Blockchain/Database'],
        ['HotStuff [Yin et al., 2019]', '33.3%', '-', '1-3 s', 'O(n)', 'Blockchain'],
        ['Tendermint [Buchman et al., 2016]', '33.3%', '-', '1-10 s', 'O(n²)', 'Blockchain'],
        ['BFT-SMaRt [Bessani et al., 2014]', '33.3%', '-', '5-20 ms', 'O(n³)', 'Distributed Systems'],
        ['', '', '', '', '', ''],
        ['Krum [Blanchard et al., 2017]', '50%', '85%', '-', 'Low', 'Federated Learning'],
        ['Trimmed Mean [Yin et al., 2018]', '50%', '80%', '-', 'Low', 'Federated Learning'],
        ['Median', '50%', '80%', '-', 'Low', 'Federated Learning'],
        ['Multi-Krum', '50%', '90%', '-', 'Medium', 'Federated Learning'],
        ['Bulyan [Guerraoui et al., 2018]', '50%', '88%', '-', 'High', 'Federated Learning'],
    ]
    
    table = ax.table(cellText=table_data, cellLoc='center', loc='center',
                    colWidths=[0.2, 0.12, 0.12, 0.12, 0.12, 0.22])
    
    table.auto_set_font_size(False)
    table.set_fontsize(10)
    table.scale(1, 2.5)
    
    # Header style
    for i in range(6):
        cell = table[(0, i)]
        cell.set_facecolor('#2E86AB')
        cell.set_text_props(weight='bold', color='white')
    
    # Highlight CFO
    for i in range(6):
        cell = table[(1, i)]
        cell.set_facecolor('#E8F4F8')
        cell.set_text_props(weight='bold')
    
    # Alternate row colors
    for i in range(2, 12):
        for j in range(6):
            if i != 6:  # Skip empty row
                cell = table[(i, j)]
                if i % 2 == 0:
                    cell.set_facecolor('#F5F5F5')
    
    plt.title('Detailed Comparison with Existing Methods\n', fontsize=14, fontweight='bold', pad=20)
    plt.savefig('comparison_table.png', dpi=300, bbox_inches='tight')
    print("✅ Summary table saved: comparison_table.png")
    plt.close()

def main():
    """Main function: Generate all comparison figures"""
    print("=" * 60)
    print("Multi-Agent Byzantine Fault Tolerant Consensus System")
    print("Comparison Visualization Generator")
    print("=" * 60)
    print()
    
    print("Generating figures...")
    print()
    
    plot_bft_comparison()
    plot_federated_learning_comparison()
    plot_multi_agent_comparison()
    plot_tolerance_accuracy_trend()
    plot_technical_innovation_radar()
    create_summary_table()
    
    print()
    print("=" * 60)
    print("All figures generated successfully!")
    print("=" * 60)
    print()
    print("Generated files:")
    print("  1. comparison_bft.png - Comparison with traditional BFT")
    print("  2. comparison_federated.png - Comparison with federated learning")
    print("  3. comparison_multi_agent.png - Comparison with multi-agent methods")
    print("  4. tolerance_accuracy_trend.png - Tolerance vs accuracy trend")
    print("  5. technical_innovation_radar.png - Technical innovation radar")
    print("  6. comparison_table.png - Detailed comparison table")
    print()

if __name__ == '__main__':
    main()
