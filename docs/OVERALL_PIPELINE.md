# Multi-Agent Oracle Overall Pipeline

## 系统整体架构流程图

```mermaid
flowchart TB
    subgraph Client["🌐 Client Layer"]
        User[        DApp[📱 d👤 User Request]
App]
    end

    subgraph Chain["⛓️ Blockchain Layer (Solana)"]
        VRF[🎲 VRF Random Generator]
        Contract[📜 Smart Contract]
        State["💾 State Management"]
    end

    subgraph Oracle["🔮 Oracle Network Layer"]
        Registry[📋 Agent Registry]
        Pool["🤖 Agent Pool<br/>GPT-4 / Claude / Llama / Custom"]
    end

    subgraph Verification["🔐 Causal Fingerprint Verification"]
        subgraph Phase1["Phase 1: Task Commitment"]
            Commit[📝 Submit Base Prediction y₀]
        end
        
        subgraph Phase2["Phase 2: Causal Perturbation"]
            Perturb[🎯 Generate Random δ Vector]
            Challenge[📢 Publish Challenge]
        end
        
        subgraph Phase3["Phase 3: Fingerprint Extraction"]
            Response[🧠 Calculate Δy = f(x+δ) - f(x)]
            SubmitFP[📤 Submit Fingerprint]
        end
        
        subgraph Phase4["Phase 4: Aggregation & Verification"]
            Normalize[📐 Vector Normalization]
            Similarity[🔗 Cosine Similarity Matrix]
            Cluster[📊 DBSCAN Clustering]
            Consensus[✅ Consensus Reached]
            Filter[❌ Outlier Filtering]
        end
    end

    subgraph Output["📤 Output Layer"]
        Result[🎯 Final Verified Result]
        Reputation[⭐ Update Reputation]
        Fingerprint[🆔 Global Fingerprint]
    end

    User --> DApp
    DApp --> Contract
    
    Contract --> VRF
    VRF --> Perturb
    
    Pool --> Registry
    Registry --> Commit
    Commit --> Challenge
    Challenge --> Response
    Response --> SubmitFP
    SubmitFP --> Normalize
    Normalize --> Similarity
    Similarity --> Cluster
    Cluster --> Filter
    Filter --> Consensus
    
    Consensus --> Result
    Consensus --> Reputation
    Reputation --> Fingerprint
    
    Result --> DApp
    DApp --> User

    style Client fill:#e8f5e9,stroke:#2e7d32
    style Chain fill:#e3f2fd,stroke:#1565c0
    style Oracle fill:#fff3e0,stroke:#ef6c00
    style Verification fill:#fce4ec,stroke:#c2185b
    style Output fill:#f3e5f5,stroke:#7b1fa2
```

## 详细工作流程

```mermaid
sequenceDiagram
    participant U as 👤 User
    participant D as 📱 dApp
    participant C as ⛓️ Smart Contract
    participant V as 🎲 VRF
    participant A1 as 🤖 Agent 1
    participant A2 as 🤖 Agent 2
    participant A3 as 🤖 Agent N
    participant AG as 📊 Aggregator

    U->>D: Submit Query
    D->>C: Create Task
    
    par Parallel Execution
        C->>V: Request Random δ
        V-->>C: Return δ Vector
    and Agent Registration
        C->>A1: Query Available
        C->>A2: Query Available
        C->>A3: Query Available
    end
    
    C-->>D: Task Created
    
    par Phase 1: Commitment
        A1->>C: submit_base(y₀¹)
        A2->>C: submit_base(y₀²)
        A3->>C: submit_base(y₀ⁿ)
    end
    
    C->>V: Generate Perturbation δ
    V-->>C: δ = [δ₁, δ₂, ..., δₖ]
    
    C->>A1: Challenge: δ
    C->>A2: Challenge: δ
    C->>A3: Challenge: δ
    
    par Phase 2: Fingerprint
        A1->>A1: Δy¹ = f(x+δ) - f(x)
        A2->>A2: Δy² = f(x+δ) - f(x)
        A3->>A3: Δyⁿ = f(x+δ) - f(x)
    end
    
    par Phase 3: Submission
        A1->>C: submit_fingerprint(Δy¹)
        A2->>C: submit_fingerprint(Δy²)
        A3->>C: submit_fingerprint(Δyⁿ)
    end
    
    C->>AG: Aggregate & Verify
    
    par Phase 4: Verification
        AG->>AG: Normalize vectors
        AG->>AG: Compute cosine similarity
        AG->>AG: DBSCAN clustering
        AG->>AG: Filter outliers
    end
    
    AG-->>C: Consensus Result
    
    C->>D: Verified Result
    D-->>U: Return Answer
    
    C->>C: Update Reputation
```

## 三层验证体系

```mermaid
flowchart TB
    subgraph L1["🛡️ Layer 1: Intervention Response"]
        L1_Title["因果指纹提取 Δy"]
        L1_1[单次任务逻辑自洽性]
        L1_2[非线性锁定验证]
    end
    
    subgraph L2["🛡️ Layer 2: Spectral Analysis"]
        L2_Title["谱分析"]
        L2_1[特征值分布提取]
        L2_2[供应商一致性检测]
        L2_3[投毒攻击检测]
    end
    
    subgraph L3["🛡️ Layer 3: Global Fingerprint"]
        L3_Title["全局指纹"]
        L3_1[长期逻辑声誉积累]
        L3_2[女巫攻击防御]
        L3_3[模型崩溃防护]
    end
    
    L1 --> L2
    L2 --> L3
    
    style L1 fill:#ffebee,stroke:#c62828
    style L2 fill:#fff8e1,stroke:#f9a825
    style L3 fill:#e8f5e9,stroke:#2e7d32
```

## 安全博弈模型

```mermaid
flowchart LR
    subgraph Attack["🦹 Attack Vectors"]
        A1[单点篡改]
        A2[共谋攻击]
        A3[女巫攻击]
        A4[模型投毒]
    end
    
    subgraph Defense["🛡️ Defense Mechanisms"]
        D1[非线性锁定]
        D2[高维防御]
        D3[异构性红利]
        D4[谱分析检测]
    end
    
    A1 -->|应对| D1
    A2 -->|应对| D2
    A3 -->|应对| D3
    A4 -->|应对| D4
    
    style Attack fill:#ffebee,stroke:#d32f2f
    style Defense fill:#e8f5e9,stroke:#388e3c
```

## 技术架构

```mermaid
flowchart TB
    subgraph Frontend["🎨 Frontend"]
        Web[Web App]
        Mobile[Mobile App]
        API[API Gateway]
    end
    
    subgraph Backend["⚙️ Backend Services"]
        Oracle[Oracle Controller]
        P2P[P2P Network<br/>Iroh]
        Engine[Consensus Engine]
        DB[(Database)]
    end
    
    subgraph Blockchain["⛓️ Blockchain"]
        Solana[Solana]
        Contract[Smart Contract]
        VRF[VRF Oracle]
    end
    
    subgraph Compute["💻 Compute Layer"]
        Rust[Rust Runtime]
        Linear[Linear Algebra<br/>nalgebra]
        ML[ML Models]
    end
    
    Frontend --> API
    API --> Oracle
    Oracle --> P2P
    Oracle --> Engine
    Engine --> DB
    
    Oracle --> Solana
    Solana --> Contract
    Contract --> VRF
    
    Engine --> Rust
    Rust --> Linear
    Rust --> ML
    
    style Frontend fill:#e3f2fd,stroke:#1565c0
    style Backend fill:#fff3e0,stroke:#ef6c00
    style Blockchain fill:#fce4ec,stroke:#c2185b
    style Compute fill:#e8f5e9,stroke:#2e7d32
```
