# Authentication & Identity Domain

基于 Better-Auth 核心与全量扩展生态的现代认证、持久化会话管理与多租户组织授权领域模型。

## Language

**User**:
系统中的核心主体与唯一自然人档案。
_Avoid_: Account, Client, Member

**Account**:
关联到 User 的具体身份凭据提供商或第三方 OAuth 连接（如 Google, GitHub, 密码凭证）。
_Avoid_: User, Profile, Identity

**Session**:
代表 User 当前已认证活跃状态的持久化会话记录，由加密安全的 session token 索引。
_Avoid_: Token, Connection, AuthState

**TwoFactor**:
User 绑定的第二认证因子，特指基于 TOTP (Time-Based One-Time Password) 算法的时间同步口令与备用恢复码。
_Avoid_: 2FA, OTP, MFA

**Passkey**:
基于 FIDO2 / WebAuthn 标准注册在用户硬件或生物识别设备上的公钥凭据。
_Avoid_: SecurityKey, Fingerprint, Biometrics

**ApiKey**:
用于机器通信（M2M）与开放平台集成的长期有效开发者密钥，支持权限配额与时效控制。
_Avoid_: Token, Secret, Bearer

**Organization**:
多租户协作边界与工作空间实体，负责团队资源隔离与计费归属。
_Avoid_: Tenant, Team, Workspace, Group

**Member**:
将 User 关联至特定 Organization 的成员从属关系，承载角色与权限控制。
_Avoid_: User, Collaborator, Teammate

**Invitation**:
向指定邮箱发送的加入特定 Organization 的受邀凭证与状态流转记录。
_Avoid_: Invite, Request, Notice

**Verification**:
用于邮箱激活、密码找回、手机校验或免密链接的短期一次性口令或 Token 实体。
_Avoid_: OTP, Code, Ticket

**DeviceCode**:
遵循 RFC 8628 设备授权流标准签发的设备配对码与用户授权凭据流转状态。
_Avoid_: PairingCode, AuthCode
