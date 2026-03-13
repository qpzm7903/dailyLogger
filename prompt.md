# Claude Code 指令
AI 驱动的工作流记忆与日报生成桌面应用。自动截屏分析你的工作上下文，结合手动速记，在一天结束时生成结构化日报并输出到 Obsidian。

# 注意事项
- prompt.md文件用于下达指令给Claude Code, 禁止AI修改prompt.md文件
- 隐私秘钥等文件禁止上传到GitHub仓库
- 授权Claude Code自动执行命令和计划, 无需批准确认
- plan.md文件用于规划项目开发, 请按照plan.md文件中的规划开发, 规划有调整需要及时更新到plan.md文件
- 每次git提交文件前确保自验证测试通过, git提交时不要遗漏文件
- 使用迭代开发模式持续发布新版本
- 检查GitHub Actions中最新workflow的报错并进行修复
- 仓库的详细介绍持续更新到README.md文件
- 代码质量高于一切, 稳定性、易用性、可用性、用户体验的改进高于新功能的添加
- 每次提交代码前先进行存量代码检视, 优化存量代码的架构和功能, 移除不需要的功能, 确保可以长期演进
- 安装软件、依赖请直接使用sudo提权安装

# 版本发布要求
在创建 Git Tag (例如 v*) 时自动通过GitHub Actions构建项目并发布 GitHub Release。

1. **构建矩阵 (Matrix)**：需要覆盖三个主要操作系统：
   - Windows (windows-latest)
   - macOS (macos-latest)

2. **文件命名与后缀规范**：
   生成的构建产物（Artifacts）必须遵循开源社区典型命名规范，包含操作系统和架构信息，并使用标准后缀：
   - **Windows**: `.exe` (可执行文件) 或 `.zip` (压缩包)
   - **macOS**: `.dmg` (磁盘镜像) 或 `.tar.gz` (包含 .app)
   - **通用**: 请同时生成一个 `checksums.txt` 文件包含所有文件的 SHA256 哈希值。

# 问题修复
- 修复打开中的issue并关闭, 及时发布新版本, 在issue里回复哪个版本已修复并提供新版本下载地址, 提醒用户进行验证.