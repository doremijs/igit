# igit

Git 工作流工具

## 简介

本项目旨在使用 Rust 语言整合前端工程化中的 Git 工作流工具，如 husky、lint-staged、commitlint 和 devmoji。通过一个配置文件来管理这些功能模块，并使用 napi 将其封装并发布为 npm 包。

## 功能

- `hooks`: 在钩子中执行脚本
- `staged-hooks`: 在提交前对暂存区的文件进行 lint 检查
- `commitlint`: 检查提交信息的格式是否符合规范并使用表情符号

## 安装

```bash
npm install -D @doremojs/igit
```

## 使用

在项目根目录下或者`.config/`目录下创建 `.igit.yaml` 文件，并添加以下内容

```yaml
hooks:
  enabled: true
  hooks:
    applypatch-msg: echo "applypatch-msg"
    pre-applypatch: echo "pre-applypatch"
    pre-commit: echo "pre-commit"
    pre-merge-commit: echo "pre-merge-commit"
    prepare-commit-msg: echo "prepare-commit-msg"
    commit-msg: echo "commit-msg"
    post-commit: echo "post-commit"
    pre-rebase: echo "pre-rebase"
    post-checkout: echo "post-checkout"
    post-merge: echo "post-merge"
    pre-push: echo "pre-push"
    pre-receive: echo "pre-receive"
    update: echo "update"
    proc-receive: echo "proc-receive"
    post-receive: echo "post-receive"
    post-update: echo "post-update"
    reference-transaction: echo "reference-transaction"
    push-to-checkout: echo "push-to-checkout"
    pre-auto-gc: echo "pre-auto-gc"
    post-rewrite: echo "post-rewrite"
    # ...more
staged-hooks:
  enabled: true
  rules:
    '*.{js,jsx,ts,tsx}': 'biome check --write'
    '*.{json,yaml,yml}': 'prettier --write'
    '*.{css,scss,less,styl,stylus}': 'stylelint --fix'
commitlint:
  enabled: true
  prependEmoji: true
```
