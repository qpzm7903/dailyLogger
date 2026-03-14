# API Key 加密存储规格

## 功能需求
实现 API Key 的加密存储，提升敏感配置的安全性，防止明文存储导致的安全风险。

## 优化范围
1. API Key 加密存储到数据库
2. 应用启动时解密加载
3. 内存中使用后及时清除
4. 支持已存在明文 Key 的迁移

## 不在范围内
- 不加密其他配置项（仅 API Key）
- 不使用系统 Keychain（简化实现）
- 不支持多密钥管理

## 验收条件（Given/When/Then）

### AC1 - 加密存储
- Given 用户在设置界面输入 API Key
- When 保存设置
- Then API Key 使用 AES-256 加密后存入数据库，非明文

### AC2 - 解密读取
- Given 数据库中存在加密的 API Key
- When 应用启动加载设置
- Then API Key 正确解密并可正常使用

### AC3 - 迁移兼容
- Given 数据库中存在旧的明文 API Key
- When 应用启动
- Then 自动将明文 Key 加密存储，并删除明文记录

### AC4 - 内存安全
- Given API Key 已加载到内存
- When 调用 AI API 完成后
- Then 敏感字符串及时归零清除（best effort）

### AC5 - 日志安全
- Given 应用记录日志
- When 输出配置相关信息
- Then API Key 以 `sk-...****` 格式脱敏显示

## 技术约束
- 使用 Rust `aes-gcm` 或 `ring` crate 进行加密
- 加密密钥基于机器特征生成（设备 ID 或类似）
- Rust 测试：`cargo test` 须通过

## 接口定义

### 后端函数
- `encrypt_api_key(plain: String) -> Result<String, String>` - 加密
- `decrypt_api_key(encrypted: String) -> Result<String, String>` - 解密
- `migrate_plain_api_key() -> Result<(), String>` - 迁移明文 Key

### 数据库变更
- `settings.api_key` 字段存储加密后的 Base64 字符串

## 安全考虑
1. 加密密钥存储位置：应用数据目录下的 `.key` 文件
2. 密钥文件权限：仅当前用户可读 (chmod 600)
3. 重装应用需重新配置 API Key（密钥丢失）

## 依赖
- 无前置依赖