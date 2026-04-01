#!/usr/bin/env python3
"""
分析两个25轮实验的统计数据
"""

import json
import statistics
from pathlib import Path

def load_experiment_data(exp_dir):
    """加载实验数据"""
    with open(f"{exp_dir}/results.json", 'r', encoding='utf-8') as f:
        return json.load(f)

def analyze_experiments(exp1_data, exp2_data):
    """分析两个实验的数据"""
    
    # 合并两个实验的数据
    all_rounds = []
    for exp_data in [exp1_data, exp2_data]:
        for round_data in exp_data:
            all_rounds.append(round_data)
    
    print("=" * 70)
    print("两轮25轮实验综合统计分析报告")
    print("=" * 70)
    
    # 1. 基础统计
    total_rounds = len(all_rounds)
    consensus_reached_count = sum(1 for r in all_rounds if r.get('consensus_reached', False))
    
    print(f"\n【1. 基础信息】")
    print(f"  总轮次: {total_rounds}")
    print(f"  共识达成轮次: {consensus_reached_count}")
    print(f"  共识达成率: {consensus_reached_count/total_rounds*100:.2f}%")
    
    # 2. 准确率分析
    accuracies = [r['accuracy'] for r in all_rounds]
    print(f"\n【2. 准确率分析】")
    print(f"  平均准确率: {statistics.mean(accuracies)*100:.2f}%")
    print(f"  准确率标准差: {statistics.stdev(accuracies)*100:.2f}%")
    print(f"  准确率最小值: {min(accuracies)*100:.2f}%")
    print(f"  准确率最大值: {max(accuracies)*100:.2f}%")
    print(f"  准确率中位数: {statistics.median(accuracies)*100:.2f}%")
    
    # 准确率分布
    high_acc = sum(1 for a in accuracies if a >= 0.95)
    mid_acc = sum(1 for a in accuracies if 0.85 <= a < 0.95)
    low_acc = sum(1 for a in accuracies if a < 0.85)
    print(f"\n  准确率分布:")
    print(f"    >= 95%: {high_acc}轮 ({high_acc/total_rounds*100:.1f}%)")
    print(f"    85%-95%: {mid_acc}轮 ({mid_acc/total_rounds*100:.1f}%)")
    print(f"    < 85%: {low_acc}轮 ({low_acc/total_rounds*100:.1f}%)")
    
    # 3. 收敛时间分析
    convergence_times = [r['convergence_time_ms'] for r in all_rounds]
    print(f"\n【3. 收敛时间分析】")
    print(f"  平均收敛时间: {statistics.mean(convergence_times)/1000:.2f}秒")
    print(f"  收敛时间标准差: {statistics.stdev(convergence_times)/1000:.2f}秒")
    print(f"  最小收敛时间: {min(convergence_times)/1000:.2f}秒")
    print(f"  最大收敛时间: {max(convergence_times)/1000:.2f}秒")
    print(f"  收敛时间中位数: {statistics.median(convergence_times)/1000:.2f}秒")
    
    # 4. 拜占庭节点分析
    byzantine_counts = [r['byzantine_count'] for r in all_rounds]
    print(f"\n【4. 拜占庭节点分析】")
    print(f"  平均拜占庭节点数: {statistics.mean(byzantine_counts):.2f}")
    print(f"  拜占庭节点范围: {min(byzantine_counts)} - {max(byzantine_counts)}")
    
    # 按拜占庭节点数分组统计准确率
    print(f"\n  按拜占庭节点数分组的准确率:")
    byz_groups = {}
    for r in all_rounds:
        byz = r['byzantine_count']
        if byz not in byz_groups:
            byz_groups[byz] = []
        byz_groups[byz].append(r['accuracy'])
    
    for byz in sorted(byz_groups.keys()):
        accs = byz_groups[byz]
        print(f"    {byz}个拜占庭节点: 平均准确率 {statistics.mean(accs)*100:.2f}% (共{len(accs)}轮)")
    
    # 5. 异常值检测分析
    outlier_counts = [len(r.get('outliers', [])) for r in all_rounds]
    rounds_with_outliers = sum(1 for c in outlier_counts if c > 0)
    total_outliers = sum(outlier_counts)
    
    print(f"\n【5. 异常值检测分析】")
    print(f"  检测到异常的轮次: {rounds_with_outliers}轮 ({rounds_with_outliers/total_rounds*100:.1f}%)")
    print(f"  总异常检测次数: {total_outliers}")
    print(f"  平均每轮异常数: {statistics.mean(outlier_counts):.2f}")
    
    # 6. 共识相似度分析
    similarities = [r.get('consensus_similarity', 1.0) for r in all_rounds]
    print(f"\n【6. 共识相似度分析】")
    print(f"  平均共识相似度: {statistics.mean(similarities)*100:.2f}%")
    print(f"  完全共识(=1.0)的轮次: {sum(1 for s in similarities if s >= 0.999)}轮")
    
    # 7. 两个实验分别的统计
    print(f"\n【7. 两轮实验分别统计】")
    
    for i, (exp_name, exp_data) in enumerate([("实验1", exp1_data), ("实验2", exp2_data)], 1):
        accs = [r['accuracy'] for r in exp_data]
        times = [r['convergence_time_ms'] for r in exp_data]
        byz = [r['byzantine_count'] for r in exp_data]
        
        print(f"\n  {exp_name}:")
        print(f"    平均准确率: {statistics.mean(accs)*100:.2f}%")
        print(f"    平均收敛时间: {statistics.mean(times)/1000:.2f}秒")
        print(f"    平均拜占庭节点: {statistics.mean(byz):.2f}")
        
    # 8. API调用统计
    api_calls = [r.get('api_calls_count', 0) for r in all_rounds]
    print(f"\n【8. API调用统计】")
    print(f"  总API调用次数: {sum(api_calls)}")
    print(f"  平均每轮API调用: {statistics.mean(api_calls):.0f}")
    
    # 9. 性能总结
    print(f"\n【9. 性能总结】")
    print(f"  ✓ 系统在{consensus_reached_count/total_rounds*100:.1f}%的轮次中达成共识")
    print(f"  ✓ 平均准确率达到{statistics.mean(accuracies)*100:.2f}%")
    print(f"  ✓ 在存在拜占庭节点({statistics.mean(byzantine_counts):.1f}个平均)的情况下仍能保持{statistics.mean(accuracies)*100:.1f}%的准确率")
    print(f"  ✓ 平均收敛时间{statistics.mean(convergence_times)/1000:.1f}秒")
    
    print("\n" + "=" * 70)
    
    return {
        'total_rounds': total_rounds,
        'consensus_rate': consensus_reached_count/total_rounds*100,
        'avg_accuracy': statistics.mean(accuracies)*100,
        'avg_convergence_time': statistics.mean(convergence_times)/1000,
        'avg_byzantine': statistics.mean(byzantine_counts)
    }

if __name__ == "__main__":
    # 加载两个实验的数据
    exp1_data = load_experiment_data("experiments/output/real_experiment_1770712878")
    exp2_data = load_experiment_data("experiments/output/real_experiment_1770719164")
    
    # 分析数据
    results = analyze_experiments(exp1_data, exp2_data)
