// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Counters.sol";

/**
 * @title 信誉存储合约
 * @dev 存储和管理预言机智能体的信誉分
 */
contract ReputationStorage is AccessControl {
    using Counters for Counters.Counter;
    
    // 角色定义
    bytes32 public constant ORACLE_MANAGER_ROLE = keccak256("ORACLE_MANAGER_ROLE");
    bytes32 public constant REPUTATION_UPDATER_ROLE = keccak256("REPUTATION_UPDATER_ROLE");
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    
    // 信誉分结构
    struct ReputationScore {
        string agentDid;                   // 智能体DID
        uint256 score;                     // 信誉分（放大100倍存储，避免小数）
        uint256 stakedAmount;              // 质押金额
        uint256 totalServices;             // 总服务次数
        uint256 successfulServices;        // 成功服务次数
        uint256 lastUpdated;               // 最后更新时间
        uint256 lastActive;                // 最后活跃时间
        bool isActive;                     // 是否活跃
        uint256 penaltyCount;              // 惩罚次数
        uint256 rewardCount;               // 奖励次数
    }
    
    // 信誉更新记录
    struct ReputationUpdate {
        string agentDid;
        int256 delta;                      // 变化值（正数为奖励，负数为惩罚）
        uint256 timestamp;
        string reason;                     // 更新原因
        address updater;                   // 更新者
    }
    
    // 质押记录
    struct StakeRecord {
        string agentDid;
        uint256 amount;
        uint256 stakedAt;
        uint256 unstakedAt;
        bool isActive;
    }
    
    // 事件
    event ReputationUpdated(
        string indexed agentDid,
        int256 delta,
        uint256 newScore,
        string reason,
        address updater
    );
    
    event AgentRegistered(
        string indexed agentDid,
        address indexed owner,
        uint256 initialScore,
        uint256 initialStake
    );
    
    event StakeAdded(
        string indexed agentDid,
        uint256 amount,
        uint256 totalStake
    );
    
    event StakeRemoved(
        string indexed agentDid,
        uint256 amount,
        uint256 totalStake
    );
    
    event AgentSlashed(
        string indexed agentDid,
        uint256 amount,
        string reason
    );
    
    // 存储
    mapping(string => ReputationScore) public reputationScores;
    mapping(string => ReputationUpdate[]) public reputationHistory;
    mapping(string => StakeRecord[]) public stakeHistory;
    mapping(string => address) public didToOwner;
    mapping(address => string) public ownerToDid;
    
    Counters.Counter private _agentCount;
    
    // 配置参数
    uint256 public constant SCORE_MULTIPLIER = 100; // 分数放大倍数
    uint256 public constant INITIAL_SCORE = 100 * SCORE_MULTIPLIER; // 初始分数100分
    uint256 public constant MIN_SCORE = 0; // 最低分数
    uint256 public constant MAX_SCORE = 1000 * SCORE_MULTIPLIER; // 最高分数1000分
    uint256 public constant MIN_STAKE = 0.01 ether; // 最小质押金额
    
    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
        _grantRole(ORACLE_MANAGER_ROLE, msg.sender);
        _grantRole(REPUTATION_UPDATER_ROLE, msg.sender);
    }
    
    /**
     * @dev 注册新的预言机智能体
     * @param agentDid 智能体DID
     * @param initialStake 初始质押金额
     */
    function registerAgent(
        string memory agentDid,
        uint256 initialStake
    ) external payable onlyRole(ORACLE_MANAGER_ROLE) {
        require(bytes(agentDid).length > 0, "Invalid DID");
        require(reputationScores[agentDid].score == 0, "Agent already registered");
        require(msg.value >= initialStake, "Insufficient stake amount");
        require(initialStake >= MIN_STAKE, "Stake below minimum");
        
        // 创建信誉记录
        ReputationScore memory newScore = ReputationScore({
            agentDid: agentDid,
            score: INITIAL_SCORE,
            stakedAmount: initialStake,
            totalServices: 0,
            successfulServices: 0,
            lastUpdated: block.timestamp,
            lastActive: block.timestamp,
            isActive: true,
            penaltyCount: 0,
            rewardCount: 0
        });
        
        reputationScores[agentDid] = newScore;
        didToOwner[agentDid] = msg.sender;
        ownerToDid[msg.sender] = agentDid;
        
        // 记录质押
        StakeRecord memory stakeRecord = StakeRecord({
            agentDid: agentDid,
            amount: initialStake,
            stakedAt: block.timestamp,
            unstakedAt: 0,
            isActive: true
        });
        
        stakeHistory[agentDid].push(stakeRecord);
        
        _agentCount.increment();
        
        emit AgentRegistered(agentDid, msg.sender, INITIAL_SCORE, initialStake);
        emit StakeAdded(agentDid, initialStake, initialStake);
    }
    
    /**
     * @dev 更新信誉分
     * @param agentDid 智能体DID
     * @param delta 变化值（放大100倍）
     * @param reason 更新原因
     */
    function updateReputation(
        string memory agentDid,
        int256 delta,
        string memory reason
    ) external onlyRole(REPUTATION_UPDATER_ROLE) {
        ReputationScore storage score = reputationScores[agentDid];
        require(score.score > 0, "Agent not registered");
        require(score.isActive, "Agent is inactive");
        
        uint256 oldScore = score.score;
        uint256 newScore;
        
        if (delta > 0) {
            // 奖励
            newScore = oldScore + uint256(delta);
            if (newScore > MAX_SCORE) {
                newScore = MAX_SCORE;
            }
            score.rewardCount++;
        } else {
            // 惩罚
            uint256 absDelta = uint256(-delta);
            if (absDelta > oldScore) {
                newScore = MIN_SCORE;
            } else {
                newScore = oldScore - absDelta;
            }
            score.penaltyCount++;
        }
        
        score.score = newScore;
        score.lastUpdated = block.timestamp;
        score.lastActive = block.timestamp;
        
        // 记录更新历史
        ReputationUpdate memory update = ReputationUpdate({
            agentDid: agentDid,
            delta: delta,
            timestamp: block.timestamp,
            reason: reason,
            updater: msg.sender
        });
        
        reputationHistory[agentDid].push(update);
        
        emit ReputationUpdated(agentDid, delta, newScore, reason, msg.sender);
    }
    
    /**
     * @dev 基于数据准确性更新信誉分
     * @param agentDid 智能体DID
     * @param expected 预期值
     * @param actual 实际值
     * @param tolerance 容忍度（百分比，放大100倍）
     * @param serviceId 服务ID
     */
    function updateForDataAccuracy(
        string memory agentDid,
        uint256 expected,
        uint256 actual,
        uint256 tolerance,
        string memory serviceId
    ) external onlyRole(REPUTATION_UPDATER_ROLE) {
        ReputationScore storage score = reputationScores[agentDid];
        require(score.score > 0, "Agent not registered");
        require(score.isActive, "Agent is inactive");
        
        score.totalServices++;
        
        if (expected == 0) {
            // 避免除零错误
            score.successfulServices++;
            return;
        }
        
        // 计算误差百分比
        uint256 error;
        if (actual > expected) {
            error = (actual - expected) * 10000 / expected; // 放大10000倍
        } else {
            error = (expected - actual) * 10000 / expected;
        }
        
        int256 delta;
        string memory reason;
        
        if (error <= tolerance) {
            // 在容忍范围内，奖励
            delta = int256(calculateReward(score.score, error, tolerance));
            score.successfulServices++;
            reason = string(abi.encodePacked("Data accuracy: ", serviceId));
        } else {
            // 超出容忍范围，惩罚
            delta = -int256(calculatePenalty(score.score, error, tolerance));
            reason = string(abi.encodePacked("Data inaccuracy: ", serviceId));
        }
        
        updateReputation(agentDid, delta, reason);
    }
    
    /**
     * @dev 增加质押
     * @param amount 质押金额
     */
    function addStake(uint256 amount) external payable {
        string memory agentDid = ownerToDid[msg.sender];
        require(bytes(agentDid).length > 0, "Agent not registered");
        require(msg.value >= amount, "Insufficient funds");
        
        ReputationScore storage score = reputationScores[agentDid];
        require(score.isActive, "Agent is inactive");
        
        score.stakedAmount += amount;
        
        // 记录质押
        StakeRecord memory stakeRecord = StakeRecord({
            agentDid: agentDid,
            amount: amount,
            stakedAt: block.timestamp,
            unstakedAt: 0,
            isActive: true
        });
        
        stakeHistory[agentDid].push(stakeRecord);
        
        emit StakeAdded(agentDid, amount, score.stakedAmount);
    }
    
    /**
     * @dev 减少质押
     * @param amount 减少金额
     */
    function removeStake(uint256 amount) external {
        string memory agentDid = ownerToDid[msg.sender];
        require(bytes(agentDid).length > 0, "Agent not registered");
        
        ReputationScore storage score = reputationScores[agentDid];
        require(score.isActive, "Agent is inactive");
        require(score.stakedAmount >= amount, "Insufficient stake");
        
        // 检查是否有足够的质押余额
        uint256 availableStake = getAvailableStake(agentDid);
        require(availableStake >= amount, "Not enough available stake");
        
        score.stakedAmount -= amount;
        
        // 标记最近的质押记录为已提取
        uint256 remaining = amount;
        StakeRecord[] storage records = stakeHistory[agentDid];
        
        for (uint256 i = records.length; i > 0 && remaining > 0; i--) {
            StakeRecord storage record = records[i - 1];
            if (record.isActive && record.amount > 0) {
                uint256 toUnstake = record.amount < remaining ? record.amount : remaining;
                record.amount -= toUnstake;
                if (record.amount == 0) {
                    record.isActive = false;
                    record.unstakedAt = block.timestamp;
                }
                remaining -= toUnstake;
            }
        }
        
        // 转账给用户
        payable(msg.sender).transfer(amount);
        
        emit StakeRemoved(agentDid, amount, score.stakedAmount);
    }
    
    /**
     * @dev 惩罚智能体（扣除质押）
     * @param agentDid 智能体DID
     * @param amount 惩罚金额
     * @param reason 惩罚原因
     */
    function slashAgent(
        string memory agentDid,
        uint256 amount,
        string memory reason
    ) external onlyRole(ORACLE_MANAGER_ROLE) {
        ReputationScore storage score = reputationScores[agentDid];
        require(score.score > 0, "Agent not registered");
        require(score.stakedAmount >= amount, "Insufficient stake to slash");
        
        score.stakedAmount -= amount;
        score.penaltyCount++;
        
        // 转账给合约（作为惩罚）
        // 在实际实现中，这部分资金可以分配给其他诚实节点或DAO
        
        emit AgentSlashed(agentDid, amount, reason);
        emit StakeRemoved(agentDid, amount, score.stakedAmount);
    }
    
    /**
     * @dev 激活/停用智能体
     * @param agentDid 智能体DID
     * @param active 是否激活
     */
    function setAgentActive(
        string memory agentDid,
        bool active
    ) external onlyRole(ORACLE_MANAGER_ROLE) {
        ReputationScore storage score = reputationScores[agentDid];
        require(score.score > 0, "Agent not registered");
        
        score.isActive = active;
        score.lastActive = block.timestamp;
    }
    
    /**
     * @dev 获取信誉分
     * @param agentDid 智能体DID
     * @return 信誉分（原始值，已除以放大倍数）
     */
    function getScore(string memory agentDid) external view returns (uint256) {
        ReputationScore storage score = reputationScores[agentDid];
        if (score.score == 0) {
            return 0;
        }
        return score.score / SCORE_MULTIPLIER;
    }
    
    /**
     * @dev 获取成功率
     * @param agentDid 智能体DID
     * @return 成功率（百分比，放大100倍）
     */
    function getSuccessRate(string memory agentDid) external view returns (uint256) {
        ReputationScore storage score = reputationScores[agentDid];
        if (score.totalServices == 0) {
            return 10000; // 100.00%
        }
        return score.successfulServices * 10000 / score.totalServices;
    }
    
    /**
     * @dev 获取可用质押金额（考虑锁定期）
     * @param agentDid 智能体DID
     * @return 可用质押金额
     */
    function getAvailableStake(string memory agentDid) public view returns (uint256) {
        ReputationScore storage score = reputationScores[agentDid];
        uint256 totalStake = score.stakedAmount;
        
        // 简化：所有质押都有30天锁定期
        StakeRecord[] storage records = stakeHistory[agentDid];
        uint256 lockedStake = 0;
        
        for (uint256 i = 0; i < records.length; i++) {
            StakeRecord storage record = records[i];
            if (record.isActive && block.timestamp < record.stakedAt + 30 days) {
                lockedStake += record.amount;
            }
        }
        
        if (lockedStake > totalStake) {
            return 0;
        }
        
        return totalStake - lockedStake;
    }
    
    /**
     * @dev 获取信誉历史
     * @param agentDid 智能体DID
     * @param limit 限制数量
     * @return 信誉更新记录数组
     */
    function getReputationHistory(
        string memory agentDid,
        uint256 limit
    ) external view returns (ReputationUpdate[] memory) {
        ReputationUpdate[] storage history = reputationHistory[agentDid];
        uint256 count = history.length < limit ? history.length : limit;
        ReputationUpdate[] memory result = new ReputationUpdate[](count);
        
        for (uint256 i = 0; i < count; i++) {
            result[i] = history[history.length - 1 - i]; // 从最新开始
        }
        
        return result;
    }
    
    /**
     * @dev 计算奖励金额
     */
    function calculateReward(
        uint256 currentScore,
        uint256 error,
        uint256 tolerance
    ) internal pure returns (uint256) {
        // 误差越小，奖励越多
        uint256 accuracyRatio = (tolerance * 100) / (error + 1); // 避免除零
        uint256 baseReward = 1 * SCORE_MULTIPLIER; // 基础奖励1分
        
        // 当前分数越低，奖励比例越高（鼓励低分节点）
        uint256 scoreRatio = (MAX_SCORE - currentScore) * 100 / MAX_SCORE;
        
        return baseReward * accuracyRatio * (100 + scoreRatio) / 10000;
    }
    
    /**
     * @dev 计算惩罚金额
     */
    function calculatePenalty(
        uint256 currentScore,
        uint256 error,
        uint256 tolerance
    ) internal pure returns (uint256) {
        // 误差越大，惩罚越多
        uint256 errorRatio = error * 100 / tolerance;
        uint256 basePenalty = 5 * SCORE_MULTIPLIER; // 基础惩罚5分
        
        // 当前分数越高，惩罚比例越高（高分节点责任更大）
        uint256 scoreRatio = currentScore * 100 / MAX_SCORE;
        
        return basePenalty * errorRatio * (100 + scoreRatio) / 10000;
    }
    
    /**
     * @dev 获取总智能体数
     */
    function getAgentCount() external view returns (uint256) {
        return _agentCount.current();
    }
    
    /**
     * @dev 获取合约余额
     */
    function getContractBalance() external view returns (uint256) {
        return address(this).balance;
    }
    
    // 接收ETH
    receive() external payable {}
}
