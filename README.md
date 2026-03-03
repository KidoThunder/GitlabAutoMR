# Auto Merge Request & Tag 脚本

这是一个用Rust编写的命令行工具，用于在 Gitlab 上批量处理的工具。

## 功能特性

- 按 **GitLab Group** 批量操作（MR / Tag / approve-mrs 均指定 `--group-path`，含子 Group）
- 四种模式：**mr**（创建 Merge Request）、**tag**（创建 Tag）、**list-mrs**（列出 MR）、**approve-mrs**（批量批准 MR）
- MR / Tag 通过 GitLab API 完成，无需本地 git 仓库

## 安装

确保你的系统已安装Rust和Cargo。

```bash
# 克隆或下载项目后，在项目目录中运行
cargo build --release
```

## 使用方法

### MR模式（创建Merge Request）

指定 Group 下所有项目，为「源分支 → 目标分支」创建 MR：

```bash
./target/release/autoMR \
  --mode mr \
  --gitlab-url https://gitlab.com/api/v4 \
  --gitlab-token YOUR_GITLAB_TOKEN \
  --group-path your/group \
  --source-branch feature-branch \
  --target-branch main
```

### Tag模式（创建Tag）

指定 Group 下所有项目，在给定分支/ref 上创建 Tag（走 GitLab API）：

```bash
./target/release/autoMR \
  --mode tag \
  --gitlab-url https://gitlab.com/api/v4 \
  --gitlab-token YOUR_GITLAB_TOKEN \
  --group-path your/group \
  --checkout-branch release-branch \
  --tag-name v1.0.0 \
  --tag-message "Release version 1.0.0"
```

### List-MRs模式（列出Merge Requests）

```bash
# 列出所有已打开的MR
./target/release/autoMR \
  --mode list-mrs \
  --gitlab-url https://gitlab.com/api/v4 \
  --gitlab-token YOUR_GITLAB_TOKEN

# 列出所有已合并的MR
./target/release/autoMR \
  --mode list-mrs \
  --gitlab-url https://gitlab.com/api/v4 \
  --gitlab-token YOUR_GITLAB_TOKEN \
  --mr-state merged

# 列出所有状态的MR
./target/release/autoMR \
  --mode list-mrs \
  --gitlab-url https://gitlab.com/api/v4 \
  --gitlab-token YOUR_GITLAB_TOKEN \
  --mr-state all
```

### Approve-MRs模式（批量批准 Merge Requests）

对指定 GitLab Group 下的所有项目，查找「源分支 → 目标分支」的已打开 MR，并调用 GitLab Approve API 进行批准。默认源分支为 `dev`，目标分支为 `release`，可通过参数覆盖。

```bash
# 批准 group server/lobby 下所有项目里 dev → release 的 MR
./target/release/autoMR \
  --mode approve-mrs \
  --gitlab-url https://gitlab.com/api/v4 \
  --gitlab-token YOUR_GITLAB_TOKEN \
  --group-path server/lobby

# 指定源分支与目标分支
./target/release/autoMR \
  --mode approve-mrs \
  --gitlab-url https://gitlab.com/api/v4 \
  --group-path server/lobby \
  --source-branch feature/xxx \
  --target-branch main
```

### 参数说明

#### 通用
- `--mode` / `-m`: 模式，默认 `mr`，可选 `mr`、`tag`、`list-mrs`、`approve-mrs`

#### MR 模式
- `--group-path`: GitLab Group 路径（必填，如 `server/lobby`）
- `--source-branch` / `-s`: 源分支
- `--target-branch` / `-t`: 目标分支
- `--gitlab-url` / `-g`: GitLab API URL
- `--gitlab-token` / `-k`: Token，或环境变量 `GITLAB_TOKEN`

#### Tag 模式
- `--group-path`: GitLab Group 路径（必填）
- `--checkout-branch` / `-c`: 打 tag 的分支名或 commit SHA
- `--tag-name`: tag 名
- `--tag-message`: tag 注释（可选）
- `--gitlab-url` / `-g`、`--gitlab-token` / `-k`: 同上

#### list-mrs 模式
- `--gitlab-url` / `-g`、`--gitlab-token` / `-k`: 同上
- `--mr-state`: 状态筛选，默认 `opened`，可选 opened / closed / locked / merged / all

#### approve-mrs 模式
- `--group-path`: Group 路径（必填）
- `--source-branch` / `-s`: 待审批 MR 源分支（默认 `dev`）
- `--target-branch` / `-t`: 目标分支（默认 `release`）
- `--gitlab-url` / `-g`、`--gitlab-token` / `-k`: 同上

### 环境变量

- `GITLAB_TOKEN`: 可替代 `--gitlab-token`

## GitLab Token 获取

1. 登录到你的GitLab账户
2. 进入 **Settings** > **Access Tokens**
3. 创建一个新的Personal Access Token
4. 确保Token具有以下权限：
   - `api` - 访问API
   - `read_repository` - 读取仓库信息
   - `write_repository` - 创建merge request

## 示例

```bash
# MR：指定 Group，创建 source → target 的 MR
./target/release/autoMR --mode mr --gitlab-url https://gitlab.com/api/v4 --group-path server/lobby --source-branch dev --target-branch main

# Tag：指定 Group，在分支上打 tag
./target/release/autoMR --mode tag --gitlab-url https://gitlab.com/api/v4 --group-path server/lobby --checkout-branch release --tag-name v1.0.0

# 列出当前用户 MR（可加 --mr-state merged 等）
./target/release/autoMR --mode list-mrs --gitlab-url https://gitlab.com/api/v4 --gitlab-token TOKEN

# 批准 Group 下 dev → release 的 MR（默认）
./target/release/autoMR --mode approve-mrs --gitlab-url https://gitlab.com/api/v4 --group-path server/lobby
```

## 注意事项

- Token 需具备相应 API 权限（如 `api`、`read_repository`、`write_repository` 等，视操作而定）。
- MR / Tag / approve-mrs 均通过 GitLab API 完成，无需本地 clone；list-mrs 仅查询当前用户 MR。
- 失败时以非零退出码及 stderr 报错信息为准。

## 开发

```bash
cargo build --release
cargo check
cargo test
``` 