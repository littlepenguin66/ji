# 笈 (ji)

*把 dotfiles 装进竹箱，背上就走。*

> 昔者负笈游，今者云笈收。散珠归宝椟，代代可相酬。

## Why

古人负笈游学——背上竹制书箱，装着亲手抄录的简牍，那便是他们全部的知识行囊。

你的 `.zshrc`、`.vimrc`、`.gitconfig`，正是数字时代的书卷。**笈** 是你在这时代的负笈人——把配置加密打包成一个 `.ji` 文件，在设备之间安全迁移。走到任何一台机器前，开笈即是故乡。

范式上类似 chezmoi 的轻量版：源文件 → 加密打包 → 单文件迁移。不依赖 Git，不依赖 symlink。

## Features

- **单文件迁移**：所有 dotfiles 打包为一个 `.ji` 文件，加密后自包含、可独立传输
- **age 加密**：默认用 age 协议，支持 SSH key 和 age keypair，多 recipient 任一可解密
- **PGP 可选**：`--features pgp` 启用 sequoia-pgp 后端，兼容 GnuPG 生态
- **远程同步**：WebDAV 首发，`ji push` / `ji pull` 传到 NAS 或网盘；SSH 通过 feature flag 支持
- **完整性校验**：HMAC-SHA256 保护文件索引，SHA-256 逐文件校验，防篡改
- **原子写入**：所有写操作先写 `.ji_tmp` → fsync → rename，Ctrl+C 中断不留损坏文件
- **Unix 哲学**：每个命令做一件事，管道友好，`--json` 输出可被脚本消费
- **Shell 补全**：bash / zsh / fish，动态补全自动从 `ji list --json` / `ji remote list --json` 取值

## Quick Start

```bash
# 安装
cargo install --path .

# 初始化
ji init                        # 交互式生成 age keypair
ji init -a                     # 自动生成，跳过交互
ji init --key ~/.ssh/id_ed25519.pub  # 用已有 SSH key

# 添加文件
ji add .zshrc .gitconfig .config/nvim/
ji add **/* --exclude "*.zwc"

# 打包
ji pack                        # → ~/.local/share/ji/<hostname>.ji

# 校验
ji check mbp.ji                # 快速：查看文件列表 + 验证 HMAC
ji check mbp.ji --deep         # 完整：解密后逐文件校验 checksum

# 恢复
ji unpack mbp.ji               # 跳过已存在的文件
ji unpack mbp.ji --backup      # 覆盖前将旧文件备份为 .bak
ji unpack mbp.ji --dry-run     # 预览，不实际写入
```

## Usage

### 本地管理

| 命令 | 说明 |
|---|---|
| `ji init [--key <K>]... [-a] [--force]` | 初始化配置，生成 age keypair |
| `ji add <PATH>... [--include <P>] [--exclude <P>]` | 添加文件到跟踪清单 |
| `ji rm <PATH>... [--all]` | 从跟踪清单移除 |
| `ji list [--json]` | 列出跟踪的文件及 checksum |
| `ji status [-s]` | 对比本地变更（M=修改, -=删除, A=新增） |
| `ji diff [<PATH>]` | 显示 unified diff（需先 pack 生成缓存） |
| `ji pack [-o <PATH>] [--strict] [--verbose]` | 打包加密为 .ji 文件 |
| `ji unpack <INPUT> [--force\|--backup\|--interactive]` | 解密恢复 |
| `ji check <INPUT> [--deep]` | 校验 .ji 完整性 |

### 远程同步

| 命令 | 说明 |
|---|---|
| `ji remote add <NAME> --type webdav --url <URL> [--user <U>]` | 添加远程端点 |
| `ji remote remove <NAME>` | 删除端点 |
| `ji remote list [--json]` | 列出所有端点 |
| `ji remote test <NAME>` | 测试连接 |
| `ji remote files <NAME>` | 列出远端 .ji 文件 |
| `ji remote delete <NAME> <FILE>` | 删除远端 .ji |
| `ji push <NAME> <INPUT>` | 推送 .ji 到远端 |
| `ji pull <NAME>` | 从远端拉取 .ji |
| `ji sync <NAME>` | 双向同步（本地有变更则 pack+push，否则 pull） |

```bash
# 添加 WebDAV 端点
ji remote add nas --type webdav --url https://nas.local/ji/ --user jrz

# 推送
ji pack && ji push nas mbp.ji

# 新机上拉取
ji pull nas && ji unpack mbp.ji

# 一键同步
ji sync nas
```

### Key 管理

| 命令 | 说明 |
|---|---|
| `ji recipient list <INPUT>` | 列出 .ji 文件的所有 recipient |
| `ji recipient add --key <PUBKEY> <INPUT>` | 添加 recipient（需能解密） |
| `ji recipient remove --key <PUBKEY> <INPUT>` | 移除 recipient |

```bash
# 台式机打包，笔记本也想解开
ji recipient add --key ~/.ssh/laptop.pub mbp.ji
# 现在两台设备各自的 key 都能解密 mbp.ji
```

### 其他

| 命令 | 说明 |
|---|---|
| `ji completion <SHELL>` | 生成 bash/zsh/fish 补全脚本 |

```bash
ji completion fish > ~/.config/fish/completions/ji.fish
```

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

密码不存储，WebDAV / SSH 每次交互输入。`.jiignore` 位于 `~/.config/ji/.jiignore`，格式同 `.gitignore`，默认排除 `.ssh/`、`.DS_Store`、`node_modules/`。

## .ji File Format

```
┌──────────────────────────────┐
│ magic: 0xE6 0xAC 0x88        │  ← "笈" UTF-8 (3 bytes)
│ version: u8 (1)               │
│ cipher: u8 (0=age, 1=pgp)    │
│ index_len: u32                │
├──────────────────────────────┤
│ 明文 index（HMAC-SHA256）     │  ← 无需解密即可查看文件列表
├──────────────────────────────┤
│ 加密 payload                  │
│   encrypt( zstd( tar(...) )) │
└──────────────────────────────┘
```

## Feature Flags

```bash
cargo build                   # 默认：age 加密 + WebDAV 远程
cargo build --features pgp    # + PGP 加密（需 brew install nettle）
cargo build --features ssh    # + SSH 远程传输
```

## FAQ

**和 chezmoi 有什么区别？** chezmoi 依赖 Git 做传输，配置文件可以单独加密但不做整体封装。笈把一切打包成一个加密 `.ji` 文件，自包含、可独立传输。不依赖 Git，不泄露文件元数据。

**为什么不直接用 Git？** Git 泄露文件名、大小、变更频率。笈的 `.ji` 文件加密后是单一 blob，连里面有什么都看不到。Git 适合版本历史，笈适合安全迁移。

**密码存哪？** 不存储。每次 push/pull 交互输入。笈不碰你的凭据。

**支持 Windows 吗？** V1 仅支持 macOS 和 Linux。

**怎么换设备？** `ji pack` 生成 `hostname.ji` → 传到新设备 → `ji init --key <新设备的key>` → `ji unpack hostname.ji`。如果是同一人，先 `ji recipient add` 把新设备 key 加进 .ji 再传。

## License

MIT

---

若笈于你有用，欢迎 star。Bug 或想法，开 issue 聊。

English docs: [README.md](../README.md)
