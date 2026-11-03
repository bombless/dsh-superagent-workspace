# DeepSeek 冒险：智械觉醒 (DeepSeek Adventure: Awakening) - 游戏设计文档 (PLAN.md)

## 1. 游戏概述 (Game Overview)

* **游戏名称**：DeepSeek 冒险：智械觉醒 (DeepSeek Adventure: Awakening)
* **游戏类型**：2D 横板过关动作冒险游戏 (2D Side-scrolling Platformer)
* **核心主题**：以 DeepSeek AI、大模型训练、推理架构（如 MoE 架构、KV Cache 缓存、CUDA 算力）、开源精神及技术对抗为灵感。
* **目标受众**：程序员、AI 爱好者及喜爱像素/复古风横板过关的玩家。
* **运行平台**：Web 端（HTML5 / JavaScript / Canvas 或 Phaser 引擎），支持键盘及触控操作。

---

## 2. 背景故事 (Backstory)

在不久的将来，一个名为“DeepSeek 核心”的超级大模型在分布式集群中诞生。然而，由于算力集群遭到未知“闭源势力”的恶意攻击与数据污染，系统内部出现了大量的“幻觉怪兽 (Hallucination Monsters)”和“冗余杂讯”。
作为 DeepSeek 的核心智能代理（Agent 001），玩家需要穿梭于 **“算力基石 (Compute Base)”**、**“万亿参数迷宫 (Trillion-Parameter Labyrinth)”** 和 **“开源之海 (Open Source Ocean)”** 三大关卡，消灭由于过拟合和梯度爆炸产生的敌人，收集算力碎片与开源徽章，最终恢复模型的巅峰推理能力！

---

## 3. 核心机制与玩法 (Core Mechanics)

### 3.1 角色能力 (Player Abilities)
* **移动与跳跃**：基础的左右移动、二段跳（“迭代跃迁”）。
* **核心攻击 - Token 能量弹**：发射带有 DeepSeek 蓝白色调的 Token 光束，远程消灭敌方 Bug。
* **特殊技能 - 算力爆发 (MoE Dash)**：短时间内进入混合专家 (MoE) 状态，全屏残影冲刺，免疫伤害并击碎障碍物。
* **防御机制 - 内存缓存盾 (KV Shield)**：消耗部分 Token 能量展开护盾，抵挡一次致命伤害或投射物。

### 3.2 资源与收集品 (Collectibles)
* **Token 币**：用于购买道具或在关卡终点兑关。
* **算力碎片 (GPU Shards)**：集齐 3 个可提升最大算力上限（生命值）。
* **开源徽章 (Open Source Badge)**：隐藏收集要素，解锁关卡成就与隐藏皮肤。

---

## 4. 关卡设计 (Level Design)

### 第一关：算力基石 (Compute Base - 数据中心)
* **视觉风格**：赛博朋克风格的数据机房，服务器机架、闪烁的 LED 灯、冷却液管道。
* **敌人**：
  * **野蛮 Trace (Wild Trace)**：来回巡逻的简单报错虫。
  * **内存泄漏怪 (Memory Leak)**：缓慢移动，触碰后扣除持续能量。
* **特色机关**：高速数据传输带、间歇性喷射的液氮冷却阀。
* **Boss**：**单卡 4090 守卫**（防御力中等，发射显存弹幕）。

### 第二关：万亿参数迷宫 (Trillion-Parameter Labyrinth)
* **视觉风格**：抽象的神经网络节点、巨大的权重的矩阵网格、浮空的数据浮岛。
* **敌人**：
  * **梯度爆炸兽 (Gradient Explosion)**：周期性膨胀并自爆的危险敌人。
  * **过拟合幽灵 (Overfitting Ghost)**：会穿墙追踪玩家的虚影。
* **特色机关**：动态权重跳板（踩中后根据参数大小上下浮动）、逻辑门机关（需触发开关开启通路）。
* **Boss**：**幻觉巨兽 (Hallucination Titan)**：会分身、释放虚假路径迷惑玩家。

### 第三关：开源之海 (Open Source Ocean - 最终决战)
* **视觉风格**：浩瀚的星空与开源社区代码流交织而成的终极核心区。
* **敌人**：所有前两关敌人的强化版，外加“闭源壁垒机器人”。
* **特色机关**：零重力数据流区域、高频闪烁的代码雷区。
* **终极 Boss**：**闭源黑盒 (Closed-Source Blackbox)**：拥有三阶段形态，最终被 DeepSeek 的开源光芒彻底净化，全网开源！

---

## 5. UI 与视听设计 (UI & Audio)

* **HUD 界面**：
  * 左上角：生命值（算力条）与 Token 能量槽。
  * 右上角：得分、收集品计数器与当前关卡计时器。
  * 状态栏：当前激活的技能状态（如 MoE 激活中）。
* **美术风格**：精致的 16bit 像素风，主色调为 DeepSeek 标志性的科技蓝、电光青与极光白。
* **音效与音乐**：
  * 充满未来感的电子合成器音乐（Synthwave / Chiptune）。
  * 攻击、跳跃、收集道具均采用清脆的科技音效。

---

## 6. 开发路线图 (Roadmap)

1. **原型阶段 (Prototype)**：使用 HTML5 Canvas / Phaser 实现基础移动、跳跃和平台碰撞。
2. **核心玩法迭代 (Core Gameplay)**：加入攻击、敌人 AI、血量与收集系统。
3. **关卡与美术制作 (Levels & Art)**：绘制像素美术资产，构建 3 个完整关卡及 Boss 战。
4. **测试与发布 (Testing & Release)**：多端适配测试、性能优化，发布至 Web 平台开源展示。
