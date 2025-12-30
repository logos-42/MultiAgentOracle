// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/Counters.sol";
import "@openzeppelin/contracts/utils/cryptography/ECDSA.sol";

/**
 * @title 身份注册合约
 * @dev 管理DIAP身份的注册和验证
 */
contract IdentityRegistry is AccessControl {
    using Counters for Counters.Counter;
    using ECDSA for bytes32;
    
    // 角色定义
    bytes32 public constant REGISTRAR_ROLE = keccak256("REGISTRAR_ROLE");
    bytes32 public constant VERIFIER_ROLE = keccak256("VERIFIER_ROLE");
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    
    // 身份结构
    struct Identity {
        string did;                     // 去中心化标识符
        address owner;                  // 身份所有者
        bytes32 publicKeyHash;          // 公钥哈希
        uint256 registeredAt;           // 注册时间
        uint256 lastVerifiedAt;         // 最后验证时间
        bool isActive;                  // 是否活跃
        bool isVerified;                // 是否已验证
        string metadataUri;             // 元数据URI
    }
    
    // 验证请求结构
    struct VerificationRequest {
        string did;
        address requester;
        bytes zkProof;                  // 零知识证明
        uint256 requestedAt;
        VerificationStatus status;
        address verifier;
        uint256 verifiedAt;
    }
    
    // 验证状态
    enum VerificationStatus {
        Pending,
        Approved,
        Rejected,
        Revoked
    }
    
    // 状态变量
    Counters.Counter private _identityIdCounter;
    Counters.Counter private _requestIdCounter;
    
    // 映射
    mapping(string => Identity) public identities;          // DID -> 身份
    mapping(address => string) public addressToDid;         // 地址 -> DID
    mapping(uint256 => VerificationRequest) public verificationRequests; // 请求ID -> 验证请求
    mapping(string => uint256[]) public didVerificationHistory; // DID -> 验证历史
    
    // 事件
    event IdentityRegistered(
        string indexed did,
        address indexed owner,
        bytes32 publicKeyHash,
        uint256 registeredAt
    );
    
    event IdentityUpdated(
        string indexed did,
        address indexed owner,
        string metadataUri,
        uint256 updatedAt
    );
    
    event IdentityRevoked(
        string indexed did,
        address indexed revoker,
        uint256 revokedAt
    );
    
    event VerificationRequested(
        uint256 indexed requestId,
        string indexed did,
        address indexed requester,
        uint256 requestedAt
    );
    
    event VerificationApproved(
        uint256 indexed requestId,
        string indexed did,
        address indexed verifier,
        uint256 verifiedAt
    );
    
    event VerificationRejected(
        uint256 indexed requestId,
        string indexed did,
        address indexed verifier,
        uint256 rejectedAt
    );
    
    event VerificationRevoked(
        uint256 indexed requestId,
        string indexed did,
        address indexed revoker,
        uint256 revokedAt
    );
    
    // 构造函数
    constructor() {
        _grantRole(DEFAULT_ADMIN_ROLE, msg.sender);
        _grantRole(ADMIN_ROLE, msg.sender);
        _grantRole(REGISTRAR_ROLE, msg.sender);
        _grantRole(VERIFIER_ROLE, msg.sender);
    }
    
    /**
     * @dev 注册新身份
     * @param did 去中心化标识符
     * @param publicKeyHash 公钥哈希
     * @param metadataUri 元数据URI
     */
    function registerIdentity(
        string memory did,
        bytes32 publicKeyHash,
        string memory metadataUri
    ) external onlyRole(REGISTRAR_ROLE) {
        require(bytes(did).length > 0, "DID不能为空");
        require(identities[did].owner == address(0), "DID已注册");
        require(addressToDid[msg.sender] == "", "地址已注册");
        
        Identity memory newIdentity = Identity({
            did: did,
            owner: msg.sender,
            publicKeyHash: publicKeyHash,
            registeredAt: block.timestamp,
            lastVerifiedAt: 0,
            isActive: true,
            isVerified: false,
            metadataUri: metadataUri
        });
        
        identities[did] = newIdentity;
        addressToDid[msg.sender] = did;
        
        emit IdentityRegistered(did, msg.sender, publicKeyHash, block.timestamp);
    }
    
    /**
     * @dev 更新身份信息
     * @param did 去中心化标识符
     * @param publicKeyHash 新公钥哈希
     * @param metadataUri 新元数据URI
     */
    function updateIdentity(
        string memory did,
        bytes32 publicKeyHash,
        string memory metadataUri
    ) external {
        Identity storage identity = identities[did];
        require(identity.owner == msg.sender, "只有身份所有者可以更新");
        require(identity.isActive, "身份已停用");
        
        identity.publicKeyHash = publicKeyHash;
        identity.metadataUri = metadataUri;
        
        emit IdentityUpdated(did, msg.sender, metadataUri, block.timestamp);
    }
    
    /**
     * @dev 请求身份验证
     * @param did 去中心化标识符
     * @param zkProof 零知识证明
     * @return requestId 请求ID
     */
    function requestVerification(
        string memory did,
        bytes memory zkProof
    ) external returns (uint256) {
        Identity storage identity = identities[did];
        require(identity.owner == msg.sender, "只有身份所有者可以请求验证");
        require(identity.isActive, "身份已停用");
        require(!identity.isVerified, "身份已验证");
        
        _requestIdCounter.increment();
        uint256 requestId = _requestIdCounter.current();
        
        VerificationRequest memory request = VerificationRequest({
            did: did,
            requester: msg.sender,
            zkProof: zkProof,
            requestedAt: block.timestamp,
            status: VerificationStatus.Pending,
            verifier: address(0),
            verifiedAt: 0
        });
        
        verificationRequests[requestId] = request;
        didVerificationHistory[did].push(requestId);
        
        emit VerificationRequested(requestId, did, msg.sender, block.timestamp);
        
        return requestId;
    }
    
    /**
     * @dev 批准验证请求
     * @param requestId 请求ID
     */
    function approveVerification(uint256 requestId) external onlyRole(VERIFIER_ROLE) {
        VerificationRequest storage request = verificationRequests[requestId];
        require(request.status == VerificationStatus.Pending, "请求状态无效");
        
        Identity storage identity = identities[request.did];
        require(identity.isActive, "身份已停用");
        
        // 这里应该验证零知识证明
        // 简化实现：直接批准
        
        request.status = VerificationStatus.Approved;
        request.verifier = msg.sender;
        request.verifiedAt = block.timestamp;
        
        identity.isVerified = true;
        identity.lastVerifiedAt = block.timestamp;
        
        emit VerificationApproved(requestId, request.did, msg.sender, block.timestamp);
    }
    
    /**
     * @dev 拒绝验证请求
     * @param requestId 请求ID
     * @param reason 拒绝原因
     */
    function rejectVerification(uint256 requestId, string memory reason) external onlyRole(VERIFIER_ROLE) {
        VerificationRequest storage request = verificationRequests[requestId];
        require(request.status == VerificationStatus.Pending, "请求状态无效");
        
        request.status = VerificationStatus.Rejected;
        request.verifier = msg.sender;
        request.verifiedAt = block.timestamp;
        
        emit VerificationRejected(requestId, request.did, msg.sender, block.timestamp);
    }
    
    /**
     * @dev 撤销身份
     * @param did 去中心化标识符
     */
    function revokeIdentity(string memory did) external {
        Identity storage identity = identities[did];
        require(
            identity.owner == msg.sender || hasRole(ADMIN_ROLE, msg.sender),
            "没有权限撤销身份"
        );
        require(identity.isActive, "身份已停用");
        
        identity.isActive = false;
        identity.isVerified = false;
        
        // 清理地址映射
        addressToDid[identity.owner] = "";
        
        emit IdentityRevoked(did, msg.sender, block.timestamp);
    }
    
    /**
     * @dev 撤销验证
     * @param requestId 请求ID
     */
    function revokeVerification(uint256 requestId) external onlyRole(ADMIN_ROLE) {
        VerificationRequest storage request = verificationRequests[requestId];
        require(
            request.status == VerificationStatus.Approved,
            "只有已批准的验证可以撤销"
        );
        
        Identity storage identity = identities[request.did];
        
        request.status = VerificationStatus.Revoked;
        identity.isVerified = false;
        
        emit VerificationRevoked(requestId, request.did, msg.sender, block.timestamp);
    }
    
    /**
     * @dev 验证签名
     * @param did 去中心化标识符
     * @param message 消息
     * @param signature 签名
     * @return 是否验证通过
     */
    function verifySignature(
        string memory did,
        bytes32 message,
        bytes memory signature
    ) external view returns (bool) {
        Identity memory identity = identities[did];
        require(identity.isActive, "身份已停用");
        require(identity.isVerified, "身份未验证");
        
        address signer = message.recover(signature);
        return signer == identity.owner;
    }
    
    /**
     * @dev 获取身份信息
     * @param did 去中心化标识符
     * @return 身份信息
     */
    function getIdentity(string memory did) external view returns (Identity memory) {
        return identities[did];
    }
    
    /**
     * @dev 获取地址对应的DID
     * @param addr 地址
     * @return DID
     */
    function getDidByAddress(address addr) external view returns (string memory) {
        return addressToDid[addr];
    }
    
    /**
     * @dev 检查身份是否有效
     * @param did 去中心化标识符
     * @return 是否有效
     */
    function isIdentityValid(string memory did) external view returns (bool) {
        Identity memory identity = identities[did];
        return identity.isActive && identity.isVerified;
    }
    
    /**
     * @dev 获取验证请求
     * @param requestId 请求ID
     * @return 验证请求
     */
    function getVerificationRequest(uint256 requestId) external view returns (VerificationRequest memory) {
        return verificationRequests[requestId];
    }
    
    /**
     * @dev 获取身份的验证历史
     * @param did 去中心化标识符
     * @return 验证请求ID列表
     */
    function getVerificationHistory(string memory did) external view returns (uint256[] memory) {
        return didVerificationHistory[did];
    }
    
    /**
     * @dev 获取身份总数
     * @return 身份总数
     */
    function getIdentityCount() external view returns (uint256) {
        return _identityIdCounter.current();
    }
    
    /**
     * @dev 获取验证请求总数
     * @return 验证请求总数
     */
    function getVerificationRequestCount() external view returns (uint256) {
        return _requestIdCounter.current();
    }
    
    /**
     * @dev 批量验证身份
     * @param dids DID列表
     * @return 验证结果列表
     */
    function batchVerifyIdentities(string[] memory dids) external view returns (bool[] memory) {
        bool[] memory results = new bool[](dids.length);
        
        for (uint256 i = 0; i < dids.length; i++) {
            Identity memory identity = identities[dids[i]];
            results[i] = identity.isActive && identity.isVerified;
        }
        
        return results;
    }
    
    /**
     * @dev 紧急停止所有身份（仅管理员）
     */
    function emergencyStopAll() external onlyRole(ADMIN_ROLE) {
        // 这里应该实现紧急停止逻辑
        // 简化实现：记录事件
        emit IdentityRevoked("ALL", msg.sender, block.timestamp);
    }
    
    /**
     * @dev 恢复身份（仅管理员）
     * @param did 去中心化标识符
     */
    function restoreIdentity(string memory did) external onlyRole(ADMIN_ROLE) {
        Identity storage identity = identities[did];
        require(!identity.isActive, "身份已活跃");
        
        identity.isActive = true;
        addressToDid[identity.owner] = did;
        
        emit IdentityUpdated(did, identity.owner, identity.metadataUri, block.timestamp);
    }
}
