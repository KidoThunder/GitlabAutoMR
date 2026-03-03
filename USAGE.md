# Auto Merge Request & Tag 使用说明

## 快速开始

1. **编译程序**
   ```bash
   cargo build --release
   ```

2. **获取GitLab Token**（MR、list-mrs、approve-mrs 模式需要）
   - 登录GitLab → Settings → Access Tokens
   - 创建Personal Access Token，确保有 `api` 权限

3. **运行程序**

### MR模式（创建Merge Request）
   ```bash
   ./target/release/autoMR \
     --mode mr \
     --gitlab-url https://gitlab.com/api/v4 \
     --gitlab-token YOUR_TOKEN \
     --group-path your/group \
     --source-branch feature-branch \
     --target-branch main
   ```

### Tag模式（创建Tag）
   ```bash
   ./target/release/autoMR \
     --mode tag \
     --gitlab-url https://gitlab.com/api/v4 \
     --gitlab-token YOUR_TOKEN \
     --group-path your/group \
     --checkout-branch release-branch \
     --tag-name v1.0.0 \
     --tag-message "Release version 1.0.0"
   ```

### List-MRs模式（列出Merge Requests）
   ```bash
   # 列出所有打开的MR
   ./target/release/autoMR \
     --mode list-mrs \
     --gitlab-url https://gitlab.com/api/v4 \
     --gitlab-token YOUR_TOKEN

   # 列出所有已合并的MR
   ./target/release/autoMR \
     --mode list-mrs \
     --gitlab-url https://gitlab.com/api/v4 \
     --gitlab-token YOUR_TOKEN \
     --mr-state merged
   ```

### Approve-MRs模式（批量批准 Merge Requests）
对指定 GitLab Group 下的所有项目，查找「源分支 → 目标分支」的已打开 MR 并调用 Approve API 批准。默认源分支 `dev`，目标分支 `release`。

   ```bash
   # 批准 group server/lobby 下 dev → release 的 MR
   ./target/release/autoMR \
     --mode approve-mrs \
     --gitlab-url https://gitlab.com/api/v4 \
     --gitlab-token YOUR_TOKEN \
     --group-path server/lobby

   # 指定源分支与目标分支
   ./target/release/autoMR \
     --mode approve-mrs \
     --gitlab-url https://gitlab.com/api/v4 \
     --group-path server/lobby \
     --source-branch feature/xxx \
     --target-branch main
   ```

## 参数说明

### 通用
| 参数 | 短参数 | 说明 |
|------|--------|------|
| `--mode` | `-m` | mr / tag / list-mrs / approve-mrs [默认: mr] |

### MR 模式
| 参数 | 短参数 | 必需 | 说明 |
|------|--------|------|------|
| `--group-path` | - | ✅ | GitLab Group 路径 |
| `--source-branch` | `-s` | ✅ | 源分支 |
| `--target-branch` | `-t` | ✅ | 目标分支 |
| `--gitlab-url` | `-g` | ✅ | GitLab API URL |
| `--gitlab-token` | `-k` | ❌ | 或环境变量 GITLAB_TOKEN |

### Tag 模式
| 参数 | 短参数 | 必需 | 说明 |
|------|--------|------|------|
| `--group-path` | - | ✅ | GitLab Group 路径 |
| `--checkout-branch` | `-c` | ✅ | 打 tag 的分支或 commit |
| `--tag-name` | - | ✅ | tag 名 |
| `--tag-message` | - | ❌ | tag 注释 |
| `--gitlab-url` / `--gitlab-token` | - | 同上 |

### list-mrs 模式
| 参数 | 短参数 | 说明 |
|------|--------|------|
| `--gitlab-url` | `-g` | 必填 |
| `--gitlab-token` | `-k` | 或 GITLAB_TOKEN |
| `--mr-state` | - | opened/closed/locked/merged/all [默认: opened] |

### approve-mrs 模式
| 参数 | 说明 |
|------|------|
| `--group-path` | 必填 |
| `--source-branch` / `--target-branch` | 默认 dev → release |
| `--gitlab-url` / `--gitlab-token` | 同上 |

## 环境变量

- `GITLAB_TOKEN` 可替代 `--gitlab-token`。

## 使用示例

```bash
# MR
./target/release/autoMR --mode mr --gitlab-url https://gitlab.com/api/v4 --group-path server/lobby --source-branch dev --target-branch main

# Tag
./target/release/autoMR --mode tag --gitlab-url https://gitlab.com/api/v4 --group-path server/lobby --checkout-branch release --tag-name v1.0.0

# list-mrs
./target/release/autoMR --mode list-mrs --gitlab-url https://gitlab.com/api/v4 --gitlab-token TOKEN

# approve-mrs（默认 dev → release）
./target/release/autoMR --mode approve-mrs --gitlab-url https://gitlab.com/api/v4 --group-path server/lobby
```

## 注意事项

- MR / Tag / approve-mrs 均通过 GitLab API 按 Group 操作，无需本地仓库。
- Token 需具备相应 API 权限。失败时以退出码与 stderr 为准。

## MR状态说明

List-MRs模式支持以下状态筛选：
- `opened`: 已打开的MR（默认）
- `closed`: 已关闭的MR
- `locked`: 已锁定的MR
- `merged`: 已合并的MR
- `all`: 所有状态的MR 