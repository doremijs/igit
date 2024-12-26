# iGit 🚀

[![NPM Version](https://img.shields.io/npm/v/@doremijs/igit-cli.svg)](https://www.npmjs.com/package/@doremijs/igit-cli)
[![License](https://img.shields.io/npm/l/@doremijs/igit-cli.svg)](https://github.com/erguotou/igit/blob/main/LICENSE)

一个强大的 Git Hooks 管理工具，让你的 Git 工作流更加智能和高效。✨

## 特性 🌟

- 🔧 简单易用的配置系统
- 🎯 支持自定义 Git Hooks 脚本
- ✨ 支持 lint-staged 文件校验
- 😊 支持为 commit message 自动添加 emoji
- 🛠️ 灵活的配置选项
- 📦 零配置，开箱即用

## 安装 📥

```bash
npm install -g @doremijs/igit-cli
# 或者使用 yarn
yarn global add @doremijs/igit-cli
# 或者使用 pnpm
pnpm add -g @doremijs/igit-cli
```

## 快速开始 🚀

1. 初始化配置文件：

```bash
igit init
```

2. 安装 Git Hooks：

```bash
igit install
```

就是这么简单！现在你的项目已经配置好了 Git Hooks。

## 配置说明 ⚙️

初始化后会在项目根目录生成 `.config/igit.yaml` 配置文件，你可以根据需要自定义配置：

```yaml
# yaml-language-server: $schema=https://igit.erguotou.me/schema/0.0.1/schema.json
hooks:
  enabled: true
  hooks:
    - pre-commit: echo "pre-commit"
staged_hooks:
  enabled: true
  rules:
    '**/*.{js,jsx,ts,tsx}': biome check --write
commit_msg:
  enabled: true
  # 启用追加 emoji 功能
  prependEmoji: true
```

## 文档 📚

详细使用说明和配置选项请访问我们的官方文档：

📖 [https://igit.erguotou.me](https://igit.erguotou.me)

## 贡献 🤝

欢迎提交 Issue 和 Pull Request！

## 许可证 📄

LGPL-3.0-or-later
