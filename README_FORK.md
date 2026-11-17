# EasyTier fork 工作分支说明

这个 fork 用于承载 EasyTier 上游功能合入前的开发与验证分支。`main` 分支保持跟随上游，不直接承载自定义修改；每个功能或修复放在独立分支中，通过 Pull Request 或移植提交贡献给上游。

## 分支状态

| 分支 | 当前状态 | 说明 |
| --- | --- | --- |
| [`main`](https://github.com/codexvn/EasyTier/tree/main) | 跟随上游 | 未加入自定义修改。 |
| [`fix/quic-multi-interface`](https://github.com/codexvn/EasyTier/tree/fix/quic-multi-interface) | 修复完成，等待上游合入 | 修复 Windows 多网卡环境下 QUIC 候选在握手前被提前释放的问题。 |
| [`feat/member-suspend`](https://github.com/codexvn/EasyTier/tree/feat/member-suspend) | 功能完成，已提交 PR #1 | 为集中式网络提供成员临时挂起能力。 |
| [`feat/network-advanced-defaults`](https://github.com/codexvn/EasyTier/tree/feat/network-advanced-defaults) | 功能开发完成，待整理 PR | 为集中式网络增加网络级高级配置默认值和成员继承模型。 |
| [`main-fix`](https://github.com/codexvn/EasyTier/tree/main-fix) | 个人集成分支 | 合入已完成的自定义功能，并配置完整非 macOS 自动构建矩阵。 |

`feat/peer-speed-selection` 当前仍指向上游 `main`，没有独立提交，因此不作为功能分支维护。

## 分支修改内容

### `fix/quic-multi-interface`

提交：`99e1b40 fix(connectivity): select QUIC candidates after handshake`

问题现象是 Windows 主机同时存在可用 Wi-Fi 和无公网路由的 Hyper-V 虚拟网卡时，开启 `bind_device` 后 QUIC 连接持续失败，日志出现 Windows 套接字错误 `10051` 和 UDP 会话关闭；关闭 `bind_device` 后可以连接。

根因在候选选择边界：`bind_device` 会把多个本地接口地址作为候选并发创建 UDP 会话。原实现对 QUIC 复用了底层传输的 `first_success`，第一个成功创建本地分类会话的候选立即胜出，其余候选被释放；但本地会话创建成功并不代表 QUIC 握手已完成。没有公网路由的 Hyper-V 接口因此可能先被选中，后续发送或握手失败时已没有其他候选可回退。

本分支的修改：

- QUIC 候选必须在完成 UDP 会话创建、QUIC 握手和双向流打开后才算成功；
- 单个候选失败后继续尝试其他候选，第一个完成协议握手的候选胜出；
- 手动连接和自动发现直连共用同一候选调度逻辑；
- 候选共享原有总体超时，并发上限为 8；候选超过一批时按批分配时间，避免后续候选永远没有尝试机会；
- 连接成功、失败、取消或超时后释放未选中的会话；
- 增加 QUIC 候选的传输失败、握手失败、连接成功和超时 debug 日志，不输出网卡地址、网络名称或密钥；
- TCP、EasyTier UDP、WireGuard 的既有连接语义保持不变，EasyTier 节点身份校验仍在 QUIC 建立后执行。

验证结果：

| 构建与配置 | 成功次数 | 观察 |
| --- | ---: | --- |
| 上游原版，`bind_device=true` | 0 / 12 | 每次出现错误 `10051` 和 UDP 会话关闭。 |
| 上游原版，`bind_device=false` | 12 / 12 | 约 0.6 秒内连接成功。 |
| 本分支，`bind_device=true` | 5 / 5 | 保持虚拟网卡启用，QUIC 连接成功。 |

针对性单元测试覆盖候选握手失败、全部候选失败、未指定绑定地址、并发上限与取消清理、排队候选继续尝试，共 5 项。

该分支还包含 `.github/workflows/quic-branch-build.yml`，推送时自动构建 Windows 和 Linux x86_64 的 `easytier-core` 与 `easytier-cli`，并上传为构建产物。

### `feat/member-suspend`

提交：

- `2fa6973 feat(web): add member suspension to central networks`
- `1e17888 fix(web): validate suspended members and align member counts`
- `227cc4d` 合入上游 `main` 的最新提交。

该分支为集中式网络增加成员临时挂起功能。挂起的成员不再进入编译输出，对应设备会停止网络实例；但成员记录、手工或已分配地址、主机名覆盖、配置覆盖、凭证和 ACL 引用都保留，恢复时重新发布原配置。

实现要点：

- `NetworkMemberIntent` 增加 `enabled` 字段，数据库迁移向后兼容，已有成员默认为启用；
- 编译器仍对挂起成员执行地址分配和 ACL 校验，地址保持占用，不能被新成员取得；
- 挂起成员持久化时回退到已存储的分配结果，启用成员缺失于编译输出时显式报错；
- `PATCH /networks/{id}/members/{device}` 支持 `enabled`，Web 网络详情提供挂起和恢复操作；
- 第二个提交修正挂起成员的配置校验，使非法覆盖不能在挂起状态被持久化并阻塞恢复；网络列表和详情不再统计挂起成员，恢复失败提示标明实际失败动作。

### `feat/network-advanced-defaults`

提交：`c8fd45f feat(web): network-level advanced config defaults`

该分支为集中式网络增加网络级高级配置默认值。网络可以维护一份稀疏配置补丁，所有成员继承这份默认值；补丁模型区分未设置、显式 false、显式空列表和恢复为核心默认值四种状态，避免把“显式关闭”误当作“未设置”。

实现要点：

- 新增 `NetworkConfigPatch` 与字段枚举，仅开放成员编辑器可配置的字段；VPN Portal 客户端和密钥仍归成员所有；
- 中央身份字段在合并后重新写回，防止成员覆盖网络身份；
- 设置页新增 Advanced Defaults 区域，支持单独保存；
- 服务端 reset 会填充当前核心默认值，并把等于核心默认值的成员字段标记为继承；
- 旧成员快照原样保留，成员代理网段凭证读取合并后的有效值。

### `main-fix`

这是个人集成分支，不是上游贡献分支。当前包含：

- `feat/member-suspend` 的全部提交；
- `fix/quic-multi-interface` 的修复；
- `ci: build full non-macos matrix on main-fix`。

构建配置调整：

- `core.yml`、`gui.yml`、`mobile.yml`、`test.yml` 在 push 时触发 `main-fix`；
- `main-fix` 按完整构建处理，不启用重复内容跳过；
- 按维护需求排除 macOS 和 Apple 签名步骤，其余上游 Core、Test、Mobile、GUI 构建矩阵保留；
- Windows ARM64 使用的 `windows-11-arm` runner 标签沿用上游配置；`actionlint` 本地版本不识别该标签，但 GitHub 官方公共仓库 runner 支持该标签。

## 维护策略

- `main` 仅跟随上游，不直接提交自定义功能；
- 每个功能或修复先在独立分支开发和验证；
- 优先向上游提交 PR，减少长期维护成本；
- `main-fix` 仅在个人环境需要提前组合多个未合入功能时更新；
- 上游合入后，对应功能分支停止维护，`main-fix` 重新基于上游 `main` 生成。
