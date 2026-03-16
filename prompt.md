# Claude Code 指令
项目请查看 @README.md 

之前有一些开发的计划、文档可以参考 @_bmad-output 目录里面的内容
# 注意事项
- prompt.md文件用于下达指令给Claude Code, 禁止AI修改prompt.md文件
- 隐私秘钥等文件禁止上传到GitHub仓库
- 授权Claude Code自动执行命令和计划, 无需批准确认
- 必须按照prompt.md文件中的开发流程完成新版本的开发
- 必须实现日志系统, 日志文件保存在用户目录项目命名的文件夹下, 方便出问题时提供日志文件以供分析定位


# 开发流程

每次任务的执行遵循新版本迭代开发全流程。

## **规划与准备 (Planning)**

在写代码之前，先明确新版本的目标。

- **创建或更新项目规划**：根据prompt.md文件中的要求, 在plan.md文件中创建或者更新当前项目短期、中期、长期功能规划, 确保可以长期迭代演进。
- **明确新版本需求清单**：需求清单从以下几个来源收集和确定：
  - plan.md文件中未实现功能的开发
  - plan.md文件中已实现功能的持续改进
  - 未关闭的GitHub Issues中的问题修复
  - 检查GitHub Actions中最新workflow的报错并进行修复
- **确定版本号**：遵循 语义化版本规范 (SemVer)（例如 v1.2.0）。
  - **MAJOR** (重大不兼容更新)
  - **MINOR** (新功能，向下兼容)
  - **PATCH** (Bug 修复，向下兼容)
- **存档新版本目标**：在plan.md文件中更新当前新版本的内容介绍。

## **开发与测试 (Development & Testing)**

根据plan.md文件中规划的当前版本需求清单, 完成开发与测试。

- **需求开发**：完成plan.md文件中规划的当前版本需求清单。
- **测试用例开发**：确保所有测试通过。
- **代码审查 (Code Review)**：Review 代码, 对架构、稳定性、易用性、可用性、可靠性、用户体验、性能、安全方面进行改进, 移除不需要的功能, 保持Clean Code。
- **提交代码**：建议每个小功能单独提交，保持 Commit 信息清晰, 提交时不要遗漏必要的文件。
- **通过CI流水线**：确保GitHub Actions workflow无报错。

## **版本发布**

在创建 Git Tag 时自动通过GitHub Actions构建项目并发布到 GitHub Release。

1. **构建矩阵 (Matrix)**：需要覆盖三个主要操作系统：
   - Windows x64 (windows-latest)
   - Linux x64 (ubuntu-latest)
   - macOS arm64 (macos-latest)
2. **文件命名与后缀规范**：生成的构建产物（Artifacts）必须遵循开源社区典型命名规范，包含操作系统和CPU架构信息，并使用标准后缀：
   - **Windows x64**: `仓库名称-tag版本号-操作系统-x64.exe`
   - **Linux x64**: `仓库名称-tag版本号-操作系统-x64.tar.gz`
   - **macOS arm64**: `仓库名称-tag版本号-操作系统-arm64.dmg`
3. **更新内容介绍**：更新内容需要介绍当前**MINOR**版本和上个**MINOR**版本之间的变化:
   - 亮点介绍
   - 新增功能
   - 优化改进
   - 问题修复

## **文档完善**

- 在plan.md文件中更新需求开发进展和状态
- 仓库的最新详细介绍更新到README.md文件

## **问题闭环**

- 关闭已解决的issue, 在issue里使用MarkDown格式回复问题是在哪个新版本解决的并提供新版本下载地址, 提醒用户进行验证.