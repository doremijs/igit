---
title: config schema
description: 了解 yaml 配置文件的 schema
---

- [schema.json](https://igit.erguotou.me/schema/0.0.2/schema.json)

  只需要在`.config/igit.yaml` 文件中添加 `# yaml-language-server: $schema=https://igit.erguotou.me/schema/0.0.2/schema.json` 注释，就可以使用 VSCode 的 yaml 语言服务器来检查配置文件的语法。
  
  我们在生成的配置文件中已自动添加了该注释，所以你不需要手动添加。
