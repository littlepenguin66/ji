# 笈 (ji)

*把 dotfiles 装进竹箱，背上就走。*

[![Rust](https://img.shields.io/badge/Rust-2024-orange?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/License-AGPL%20v3-blue?style=flat-square)](LICENSE)

> 昔者负笈游，今者云笈收。散珠归宝椟，代代可相酬。

---

## Overview

古人负笈游学——背上竹制书箱，装着亲手抄录的简牍，那便是他们全部的知识行囊。

笈是你数字时代的竹箱——把 dotfiles 加密打包成一个 `.ji` 文件，在设备间安全迁移。范式上类似 chezmoi 的轻量版：源文件 → 加密打包 → 单文件迁移。不依赖 Git，不依赖 symlink。

走到任何一台机器前，开笈即是故乡。

## Features

- **单文件迁移** — 所有 dotfiles 打包为一个自包含的 `.ji` 文件
- **age 加密** — 支持 SSH key 和 age keypair，多个 recipient 任一可解密
- **远程同步** — WebDAV 首发，SSH 通过 feature flag 支持（`--features ssh`）
- **PGP 可选** — `--features pgp` 启用 sequoia-pgp 后端，兼容 GnuPG
- **完整性校验** — HMAC-SHA256 保护文件索引，SHA-256 逐文件校验
- **原子写入** — tmp → fsync → rename，Ctrl+C 中断不留损坏文件
- **Unix 哲学** — 每个命令做一件事，`--json` 可被脚本消费，成功时静默
- **Shell 补全** — bash / zsh / fish，动态补全

## Installation

### 从 Crates.io 安装

```bash
cargo install ji
```

### 从源码构建

```bash
git clone https://github.com/littlepenguin66/ji.git
cd ji
cargo install --path .
```

预编译二进制和 Homebrew tap 即将推出。

## Usage

### 在新机器上设置

```bash
ji init                         # 生成 age keypair，创建 config
ji add .zshrc .gitconfig .config/nvim/
ji add **/* --exclude "*.zwc"   # 批量添加，支持过滤
ji list                         # 查看跟踪的文件列表（即 manifest）
ji status                       # 检查是否有未打包的变更
ji pack                         # → ~/.local/share/ji/<hostname>.ji
```

`ji list` 显示 manifest——即笈跟踪的文件集合。`ji pack` 以主机名（`uname -n`）命名输出文件，每台设备自然生成不同的 `.ji`。

### 在另一台机器上恢复

```bash
ji init --key ~/.ssh/id_ed25519.pub
ji unpack mbp.ji --dry-run      # 先预览
ji unpack mbp.ji --backup       # 恢复，已有文件备份为 .bak
ji status                       # 确认一切正常
```

### 多设备共用

打包一次，然后添加每台设备的 key，任意设备都能解密：

```bash
ji pack                         # 在台式机上
ji recipient add --key ~/.ssh/laptop.pub desktop.ji
ji recipient add --key ~/.ssh/server.pub desktop.ji
ji recipient list desktop.ji   # 确认所有 key 已加入
```

添加或移除 recipient 需要至少持有一把已有私钥。每台设备用自己的 SSH key——无需共享密钥。

### 通过 WebDAV 同步

```bash
ji remote add nas --type webdav --url https://nas.local/ji/ --user jrz
ji remote test nas
ji pack && ji push nas mbp.ji   # 上传

ji pull nas && ji unpack mbp.ji # 在另一台机器上下载
ji sync nas                     # 自动判断方向：pack+push 或 pull
```

### 排查问题

```bash
ji doctor                       # config + keys + manifest
ji doctor --full                # 加上 remote 连通性和 archive 扫描
ji doctor --json | jq .checks   # 脚本可读输出
```

## How It Works

笈遵循简单的流水线：

1. **Track** — `ji add` 把要管理的 dotfiles 记录到 manifest
2. **Pack** — tar 打包 → zstd 压缩 → age 加密 → 装入 `.ji` 容器
3. **Transfer** — 用任何方式传输 `.ji` 文件（USB、scp、WebDAV、云盘）
4. **Unpack** — 解密 → 解压 → 恢复，原子写入，冲突保护

`.ji` 文件 = `encrypt( zstd( tar(manifest + files) ))`，前面附加明文 HMAC 签名的索引，使 `ji list` 和 `ji check` 无需解密即可工作。

## Commands

| 类别 | 命令 |
|---|---|
| 本地 | `init`, `add`, `rm`, `list`, `status`, `diff` |
| 归档 | `pack`, `unpack`, `check` |
| 远程 | `remote add/remove/list/test/files/delete`, `push`, `pull`, `sync` |
| 安全 | `recipient list/add/remove` |
| 诊断 | `doctor` |
| 开发 | `completion` |

完整参考：[CLI Reference](docs/cli.md)

## Configuration

```toml
# ~/.config/ji/config.toml

[encryption]
type = "age"                          # age | pgp
recipients = ["ssh-ed25519 AAAAC3...", "age1..."]

[[remote]]
name = "nas"
type = "webdav"
url = "https://nas.local/ji/"
user = "jrz"
```

密码从不存储，WebDAV / SSH 每次交互输入。

## Ignoring Files

`.jiignore` 位于 `~/.config/ji/.jiignore`，格式同 `.gitignore`。默认排除：

| 规则 | 原因 |
|---|---|
| `.ssh/` | 私钥不应离开本机 |
| `.DS_Store` | macOS 元数据 |
| `node_modules/` | 依赖包，不是配置 |

## Troubleshooting

先运行 `ji doctor`——一个命令诊断 config、keys、manifest 和 remote 连通性。

| 问题 | 修复 |
|---|---|
| `no recipients configured` | `ji init --key ~/.ssh/id_ed25519.pub` |
| `no private key available` | 检查 `~/.local/share/ji/` 或 `~/.ssh/` |
| `HMAC verification failed` | 文件损坏，重新下载或 pack |
| `push failed: 401` | WebDAV 密码错误 |
| `connection refused` | 用 `ji remote test` 检查 URL |

完整指南：[Troubleshooting](docs/troubleshooting.md)

## Feature Flags

```bash
cargo build                   # 默认：age + WebDAV
cargo build --features pgp    # + PGP 加密（brew install nettle）
cargo build --features ssh    # + SSH 远程传输
```

## Requirements

- **Rust**: 1.80+（从源码构建时需要）
- **PGP 功能**: `nettle` 系统库（macOS: `brew install nettle`，Linux: `apt install libnettle-dev`）

## Docs

| 文档 | 内容 |
|---|---|
| [CLI Reference](docs/cli.md) | 每个命令、每个选项 |
| [Architecture](docs/architecture.md) | .ji 文件格式、分层架构、加密流程 |
| [Troubleshooting](docs/troubleshooting.md) | Doctor 导向的诊断和修复 |
| [Security](SECURITY.md) | 威胁模型和漏洞报告 |
| [Release Notes](docs/release/) | 每版更新日志 |

## FAQ

**和 chezmoi 有什么区别？** chezmoi 依赖 Git 做传输层，文件逐个加密。笈把所有东西打包成一个加密 `.ji` 文件——自包含，不依赖 Git，不泄露文件元数据。

**为什么不直接用 Git？** Git 泄露文件名、大小、变更频率。笈的 `.ji` 文件是加密后的不透明 blob。Git 适合版本历史，笈适合安全迁移。

**密码存哪？** 不存。每次 push/pull 交互输入。笈不碰你的凭据。

**支持 Windows 吗？** V1 仅支持 macOS 和 Linux。

## License

AGPL-3.0

---

若笈于你有用，欢迎 star。Bug 或想法，开 issue 聊。

[English](README.md)
