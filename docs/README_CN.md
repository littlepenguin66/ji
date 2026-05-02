# 笈 (ji)

*把 dotfiles 装进竹箱，背上就走。*

> 昔者负笈游，今者云笈收。散珠归宝椟，代代可相酬。

## Why

古人负笈游学——背上竹制书箱，装着亲手抄录的简牍，那便是他们全部的知识行囊。

你的 `.zshrc`、`.vimrc`、`.gitconfig`，正是数字时代的书卷。**笈** 是你在这时代的负笈人——把配置加密打包成一个 `.ji` 文件，在设备之间安全迁移。走到任何一台机器前，开笈即是故乡。

范式上类似 chezmoi 的轻量版：源文件 → 加密打包 → 单文件迁移。不依赖 Git，不依赖 symlink。

## Features

- **单文件迁移** — 所有 dotfiles 打包为一个自包含的 `.ji` 文件
- **age 加密** — 支持 SSH key 和 age keypair，多个 recipient 任一可解密
- **PGP 可选** — `--features pgp` 启用 sequoia-pgp 后端，兼容 GnuPG
- **远程同步** — WebDAV 首发，SSH 通过 feature flag 支持
- **完整性校验** — HMAC-SHA256 保护文件索引，SHA-256 逐文件校验
- **原子写入** — tmp → fsync → rename，Ctrl+C 中断不留损坏文件
- **Unix 哲学** — 每个命令做一件事，`--json` 输出可被脚本消费
- **Shell 补全** — bash / zsh / fish，动态补全

## Quick Start

```bash
cargo install --path .
```

### 在新机器上设置

```bash
ji init                         # 生成 age keypair，创建 config
ji add .zshrc .gitconfig .config/nvim/
ji add **/* --exclude "*.zwc"   # 批量添加，支持过滤
ji list                         # 查看跟踪的文件
ji status                       # 检查是否有未打包的变更
ji pack                         # → ~/.local/share/ji/mbp.ji
```

### 在另一台机器上恢复

```bash
ji init --key ~/.ssh/id_ed25519.pub
ji unpack mbp.ji --dry-run      # 先预览
ji unpack mbp.ji --backup       # 恢复，已有文件备份为 .bak
ji status                       # 确认一切正常
```

### 多设备共用

```bash
ji recipient add --key ~/.ssh/laptop.pub mbp.ji
ji recipient add --key ~/.ssh/desktop.pub mbp.ji
ji recipient list mbp.ji
```

每台设备用自己的 SSH key，任何一个都能解密同一份 `.ji` 文件。

### 通过 WebDAV 同步

```bash
ji remote add nas --type webdav --url https://nas.local/ji/ --user jrz
ji remote test nas
ji pack && ji push nas mbp.ji   # 上传

ji pull nas && ji unpack mbp.ji # 在另一台机器上下载
ji sync nas                     # 自动判断方向，pack+push 或 pull
```

### 排查问题

```bash
ji doctor                       # config + keys + manifest
ji doctor --full                # 加上 remote 连通性和 archive 扫描
ji doctor --json | jq .checks   # 脚本可读输出
```

## Docs

| 文档 | 内容 |
|---|---|
| [CLI Reference](docs/cli.md) | 每个命令、每个选项 |
| [Architecture](docs/architecture.md) | .ji 文件格式、分层架构、加密流程 |
| [Troubleshooting](docs/troubleshooting.md) | Doctor 导向的诊断和修复 |

## Config

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

密码从不存储，WebDAV / SSH 每次交互输入。`.jiignore` 位于 `~/.config/ji/.jiignore`，格式同 `.gitignore`，默认排除 `.ssh/`、`.DS_Store`、`node_modules/`。

## .ji File Format

`.ji` 文件 = `encrypt( zstd( tar(manifest + files) ))`，前面附加明文 HMAC 签名的索引，使 `ji list` 和 `ji check` 无需解密即可工作。完整格式见 [Architecture](docs/architecture.md)。

## Feature Flags

```bash
cargo build                   # 默认：age + WebDAV
cargo build --features pgp    # + PGP 加密（需 brew install nettle）
cargo build --features ssh    # + SSH 远程传输
```

## FAQ

**和 chezmoi 有什么区别？** chezmoi 依赖 Git 做传输层，文件逐个加密。笈把所有东西打包成一个加密 `.ji` 文件——自包含，不依赖 Git，不泄露文件元数据。

**为什么不直接用 Git？** Git 泄露文件名、大小、变更频率。笈的 `.ji` 文件是加密后的不透明 blob。Git 适合版本历史，笈适合安全迁移。

**密码存哪？** 不存。每次 push/pull 交互输入。笈不碰你的凭据。

**支持 Windows 吗？** V1 仅支持 macOS 和 Linux。

**怎么换设备？** 旧机器上 `ji pack` → 传输 `.ji` 文件 → 新机器上 `ji init --key <新 key>` → `ji unpack`。如果是同一人，先 `ji recipient add --key ~/.ssh/new.pub old.ji` 把新 key 加进去再传。

## License

MIT

---

若笈于你有用，欢迎 star。Bug 或想法，开 issue 聊。

[English](README.md)
