# CLAUDE.md

必须参考 @prompt.md

# CI/CD
- 纯文档更新（如 `.md`、`LICENSE` 等非代码文件）不应触发 CI/CD 构建
- commit message 中使用 `[skip ci]` 跳过，或确保 workflow 的 paths-ignore 已正确配置

# 版本发布
在创建 Git Tag 时自动通过GitHub Actions构建项目并发布到 GitHub Release

简单概括介绍当前MINOR版本和上个MINOR版本之间的变化
