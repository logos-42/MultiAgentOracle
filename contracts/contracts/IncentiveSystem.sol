// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";

/**
 * @title 激励系统合约
 * @dev 管理预言机智能体的经济激励和奖励分配
 */
contract IncentiveSystem is AccessControl {
    using SafeERC20 for IERC20;
    
    // 角色定义
    bytes32 public constant ORACLE_MANAGER_ROLE = keccak256("ORACLE_MANAGER_ROLE");
    bytes32 public constant REWARD_DISTRIBUTOR_ROLE = keccak256("REWARD_DISTRIBUTOR_ROLE");
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    
    // 奖励代币
    IERC20 public rewardToken;
    
    // 奖励池结构
    struct RewardPool {
        uint256 totalRewards;              // 总奖励金额
        uint256 distributedRewards;        // 已分配奖励
        uint256 startTime;                 // 开始时间
        uint256 endTime;                   // 结束时间
        bool isActive;                     // 是否活跃
        string description;                // 描述
    }
    
    // 奖励分配记录
    struct RewardDistribution {
        string agentDid;                   // 智能体DID
        uint256 amount;                    // 奖励金额
        uint256 timestamp;                 // 分配时间
        string reason;                     // 奖励原因
        uint256 poolId;                    // 奖励池ID
    }
    
    // 服务费记录
    struct ServiceFee {
        string serviceId;                  // 服务ID
        uint256 feeAmount;                 // 费用金额
        uint256 timestamp;                 // 时间戳
        address payer;                     // 支付者
        string agentDid;                   // 服务提供者
    }
    
    // 事件
    event RewardPoolCreated(
        uint256 indexed poolId,
        uint256 totalRewards,
        uint256 startTime,
        uint256 endTime,
        string description
    );
    
    event RewardDistributed(
        uint256 indexed poolId,
        string indexed agentDid,
        uint256 amount,
        string reason
    );
    
    event ServiceFeeCollected(
        string indexed serviceId,
        address indexed payer,
        string agentDid,
        uint256 feeAmount
    );
    
    event RewardsClaimed(
        string indexed agentDid,
        address indexed claimant,
        uint256 amount
    );
    
    // 存储
    RewardPool[] public rewardPools;
    mapping(string => RewardDistribution[]) public rewardHistory;
    mapping(string => uint256) public pendingRewards; // 待领取奖励
    mapping(string => ServiceFee[]) public serviceFees;
    mapping(string => uint256) public totalEarned; // 总收益
    
    // 配置参数
    uint256 public constant MIN_REWARD_POOL_DURATION = 7 days;
    uint256 public constant MAX_REWARD_POOL_DURATION = 365 days;
    uint256 public constant SERVICE_FEE_PERCENT = 30; // 服务费百分比（放大100倍）
    uint256 public constant PLATFORM_FEE_PERCENT = 10; // 平台费百分比（放大100倍）
    
    constructor(address _rewardToken) {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
        _grantRole(ORACLE_MANAGER_ROLE, msg.sender);
        _grantRole(REWARD_DISTRIBUTOR_ROLE, msg.sender);
        
        rewardToken = IERC20(_rewardToken);
    }
    
    /**
     * @dev 创建奖励池
     * @param totalRewards 总奖励金额
     * @param duration 持续时间（秒）
     * @param description 描述
     */
    function createRewardPool(
        uint256 totalRewards,
        uint256 duration,
        string memory description
    ) external onlyRole(ORACLE_MANAGER_ROLE) {
        require(totalRewards > 0, "Invalid reward amount");
        require(
            duration >= MIN_REWARD_POOL_DURATION && duration <= MAX_REWARD_POOL_DURATION,
            "Invalid duration"
        );
        
        // 转移奖励代币到合约
        rewardToken.safeTransferFrom(msg.sender, address(this), totalRewards);
        
        uint256 poolId = rewardPools.length;
        uint256 startTime = block.timestamp;
        uint256 endTime = startTime + duration;
        
        RewardPool memory newPool = RewardPool({
            totalRewards: totalRewards,
            distributedRewards: 0,
            startTime: startTime,
            endTime: endTime,
            isActive: true,
            description: description
        });
        
        rewardPools.push(newPool);
        
        emit RewardPoolCreated(poolId, totalRewards, startTime, endTime, description);
    }
    
    /**
     * @dev 分配奖励给智能体
     * @param poolId 奖励池ID
     * @param agentDid 智能体DID
     * @param amount 奖励金额
     * @param reason 奖励原因
     */
    function distributeReward(
        uint256 poolId,
        string memory agentDid,
        uint256 amount,
        string memory reason
    ) external onlyRole(REWARD_DISTRIBUTOR_ROLE) {
        require(poolId < rewardPools.length, "Invalid pool ID");
        RewardPool storage pool = rewardPools[poolId];
        
        require(pool.isActive, "Pool is not active");
        require(block.timestamp >= pool.startTime, "Pool not started");
        require(block.timestamp <= pool.endTime, "Pool ended");
        require(amount > 0, "Invalid reward amount");
        require(
            pool.distributedRewards + amount <= pool.totalRewards,
            "Insufficient pool balance"
        );
        
        // 更新奖励池
        pool.distributedRewards += amount;
        
        // 记录奖励分配
        RewardDistribution memory distribution = RewardDistribution({
            agentDid: agentDid,
            amount: amount,
            timestamp: block.timestamp,
            reason: reason,
            poolId: poolId
        });
        
        rewardHistory[agentDid].push(distribution);
        pendingRewards[agentDid] += amount;
        totalEarned[agentDid] += amount;
        
        emit RewardDistributed(poolId, agentDid, amount, reason);
    }
    
    /**
     * @dev 基于数据准确性分配奖励
     * @param poolId 奖励池ID
     * @param agentDid 智能体DID
     * @param expected 预期值
     * @param actual 实际值
     * @param tolerance 容忍度（百分比，放大100倍）
     * @param baseReward 基础奖励
     * @param serviceId 服务ID
     */
    function distributeRewardForAccuracy(
        uint256 poolId,
        string memory agentDid,
        uint256 expected,
        uint256 actual,
        uint256 tolerance,
        uint256 baseReward,
        string memory serviceId
    ) external onlyRole(REWARD_DISTRIBUTOR_ROLE) {
        require(poolId < rewardPools.length, "Invalid pool ID");
        RewardPool storage pool = rewardPools[poolId];
        
        require(pool.isActive, "Pool is not active");
        require(block.timestamp >= pool.startTime, "Pool not started");
        require(block.timestamp <= pool.endTime, "Pool ended");
        
        if (expected == 0) {
            // 避免除零错误，给予基础奖励
            distributeReward(poolId, agentDid, baseReward, 
                string(abi.encodePacked("Data accuracy: ", serviceId)));
            return;
        }
        
        // 计算误差百分比
        uint256 error;
        if (actual > expected) {
            error = (actual - expected) * 10000 / expected; // 放大10000倍
        } else {
            error = (expected - actual) * 10000 / expected;
        }
        
        uint256 rewardAmount;
        string memory reason;
        
        if (error <= tolerance) {
            // 在容忍范围内，计算奖励
            uint256 accuracyBonus = (tolerance * 100) / (error + 1); // 误差越小，奖励越多
            rewardAmount = baseReward * accuracyBonus / 100;
            
            // 限制最大奖励为10倍基础奖励
            if (rewardAmount > baseReward * 10) {
                rewardAmount = baseReward * 10;
            }
            
            reason = string(abi.encodePacked("High accuracy: ", serviceId));
        } else {
            // 超出容忍范围，给予基础奖励的10%
            rewardAmount = baseReward / 10;
            reason = string(abi.encodePacked("Low accuracy: ", serviceId));
        }
        
        distributeReward(poolId, agentDid, rewardAmount, reason);
    }
    
    /**
     * @dev 收集服务费
     * @param serviceId 服务ID
     * @param agentDid 服务提供者DID
     * @param feeAmount 费用金额
     */
    function collectServiceFee(
        string memory serviceId,
        string memory agentDid,
        uint256 feeAmount
    ) external payable {
        require(feeAmount > 0, "Invalid fee amount");
        require(msg.value >= feeAmount, "Insufficient payment");
        
        // 计算分配
        uint256 agentShare = feeAmount * (10000 - SERVICE_FEE_PERCENT - PLATFORM_FEE_PERCENT) / 10000;
        uint256 platformFee = feeAmount * PLATFORM_FEE_PERCENT / 10000;
        uint256 serviceFee = feeAmount * SERVICE_FEE_PERCENT / 10000;
        
        // 记录服务费
        ServiceFee memory feeRecord = ServiceFee({
            serviceId: serviceId,
            feeAmount: feeAmount,
            timestamp: block.timestamp,
            payer: msg.sender,
            agentDid: agentDid
        });
        
        serviceFees[agentDid].push(feeRecord);
        
        // 分配奖励给智能体
        pendingRewards[agentDid] += agentShare;
        totalEarned[agentDid] += agentShare;
        
        // 服务费进入奖励池（简化：直接分配给智能体）
        pendingRewards[agentDid] += serviceFee;
        totalEarned[agentDid] += serviceFee;
        
        // 平台费保留在合约中（可由管理员提取）
        
        emit ServiceFeeCollected(serviceId, msg.sender, agentDid, feeAmount);
    }
    
    /**
     * @dev 领取奖励
     * @param amount 领取金额
     */
    function claimRewards(uint256 amount) external {
        // 在实际实现中，这里需要验证调用者是智能体所有者
        // 简化版本：任何人都可以调用
        
        string memory agentDid = "default"; // 简化：使用默认DID
        require(pendingRewards[agentDid] >= amount, "Insufficient pending rewards");
        
        pendingRewards[agentDid] -= amount;
        
        // 转移奖励代币
        rewardToken.safeTransfer(msg.sender, amount);
        
        emit RewardsClaimed(agentDid, msg.sender, amount);
    }
    
    /**
     * @dev 领取ETH奖励（来自服务费）
     */
    function claimEthRewards(uint256 amount) external {
        // 在实际实现中，这里需要验证调用者是智能体所有者
        // 简化版本：任何人都可以调用
        
        string memory agentDid = "default"; // 简化：使用默认DID
        require(address(this).balance >= amount, "Insufficient contract balance");
        require(amount > 0, "Invalid amount");
        
        // 转移ETH
        payable(msg.sender).transfer(amount);
        
        emit RewardsClaimed(agentDid, msg.sender, amount);
    }
    
    /**
     * @dev 提取平台费（仅管理员）
     * @param amount 提取金额
     */
    function withdrawPlatformFees(uint256 amount) external onlyRole(ADMIN_ROLE) {
        require(address(this).balance >= amount, "Insufficient balance");
        payable(msg.sender).transfer(amount);
    }
    
    /**
     * @dev 提取代币（仅管理员）
     * @param amount 提取金额
     */
    function withdrawTokens(uint256 amount) external onlyRole(ADMIN_ROLE) {
        rewardToken.safeTransfer(msg.sender, amount);
    }
    
    /**
     * @dev 获取奖励池信息
     * @param poolId 奖励池ID
     */
    function getPoolInfo(uint256 poolId) external view returns (
        uint256 totalRewards,
        uint256 distributedRewards,
        uint256 remainingRewards,
        uint256 startTime,
        uint256 endTime,
        bool isActive,
        string memory description
    ) {
        require(poolId < rewardPools.length, "Invalid pool ID");
        RewardPool storage pool = rewardPools[poolId];
        
        return (
            pool.totalRewards,
            pool.distributedRewards,
            pool.totalRewards - pool.distributedRewards,
            pool.startTime,
            pool.endTime,
            pool.isActive && block.timestamp >= pool.startTime && block.timestamp <= pool.endTime,
            pool.description
        );
    }
    
    /**
     * @dev 获取智能体总收益
     * @param agentDid 智能体DID
     */
    function getAgentEarnings(string memory agentDid) external view returns (
        uint256 totalEarnedAmount,
        uint256 pendingRewardAmount,
        uint256 serviceCount
    ) {
        return (
            totalEarned[agentDid],
            pendingRewards[agentDid],
            serviceFees[agentDid].length
        );
    }
    
    /**
     * @dev 获取奖励历史
     * @param agentDid 智能体DID
     * @param limit 限制数量
     */
    function getRewardHistory(
        string memory agentDid,
        uint256 limit
    ) external view returns (RewardDistribution[] memory) {
        RewardDistribution[] storage history = rewardHistory[agentDid];
        uint256 count = history.length < limit ? history.length : limit;
        RewardDistribution[] memory result = new RewardDistribution[](count);
        
        for (uint256 i = 0; i < count; i++) {
            result[i] = history[history.length - 1 - i]; // 从最新开始
        }
        
        return result;
    }
    
    /**
     * @dev 获取服务费历史
     * @param agentDid 智能体DID
     * @param limit 限制数量
     */
    function getServiceFeeHistory(
        string memory agentDid,
        uint256 limit
    ) external view returns (ServiceFee[] memory) {
        ServiceFee[] storage history = serviceFees[agentDid];
        uint256 count = history.length < limit ? history.length : limit;
        ServiceFee[] memory result = new ServiceFee[](count);
        
        for (uint256 i = 0; i < count; i++) {
            result[i] = history[history.length - 1 - i]; // 从最新开始
        }
        
        return result;
    }
    
    /**
     * @dev 获取总奖励池数
     */
    function getPoolCount() external view returns (uint256) {
        return rewardPools.length;
    }
    
    /**
     * @dev 获取合约ETH余额
     */
    function getContractEthBalance() external view returns (uint256) {
        return address(this).balance;
    }
    
    /**
     * @dev 获取合约代币余额
     */
    function getContractTokenBalance() external view returns (uint256) {
        return rewardToken.balanceOf(address(this));
    }
    
    // 接收ETH
    receive() external payable {}
}
