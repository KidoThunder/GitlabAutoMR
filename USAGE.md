# Auto Merge Request & Tag 使用说明

## 快速开始

1. **编译程序**
   ```bash
   cargo build --release
   ```

2. **获取GitLab Token**（仅MR模式需要）
   - 登录GitLab → Settings → Access Tokens
   - 创建Personal Access Token，确保有 `api` 权限

3. **运行程序**

### MR模式（创建Merge Request）
   ```bash
   ./target/release/autoMR \
     --path /path/to/repositories \
     --source-branch feature-branch \
     --target-branch main \
     --gitlab-url https://gitlab.com/api/v4 \
     --gitlab-token YOUR_TOKEN \
     --mode mr
   ```

### Tag模式（创建Tag）
   ```bash
   ./target/release/autoMR \
     --path /path/to/repositories \
     --checkout-branch release-branch \
     --tag-name v1.0.0 \
     --tag-message "Release version 1.0.0" \
     --mode tag
   ```

## 参数说明

### 通用参数
| 参数 | 短参数 | 必需 | 说明 |
|------|--------|------|------|
| `--path` | `-p` | ✅ | 要遍历的根路径 |
| `--mode` | `-m` | ❌ | 操作模式：mr（创建merge request）或 tag（创建tag）[默认: mr] |

### MR模式参数
| 参数 | 短参数 | 必需 | 说明 |
|------|--------|------|------|
| `--source-branch` | `-s` | ✅ | 要推送的源分支名 |
| `--target-branch` | `-t` | ✅ | 目标合并分支名 |
| `--gitlab-url` | `-g` | ✅ | GitLab API URL |
| `--gitlab-token` | `-k` | ❌ | GitLab API Token (也可用环境变量) |
| `--force` | `-f` | ❌ | 强制推送 |

### Tag模式参数
| 参数 | 短参数 | 必需 | 说明 |
|------|--------|------|------|
| `--checkout-branch` | `-c` | ✅ | 要切换到的分支名 |
| `--tag-name` | - | ✅ | 要创建的tag名 |
| `--tag-message` | - | ❌ | tag的注释信息 |

## 环境变量

```bash
export GITLAB_TOKEN="your_token_here"
./target/release/autoMR --path . --source-branch feature --target-branch main --gitlab-url https://gitlab.com/api/v4 --mode mr
```

## 功能特性

- 🔍 自动遍历指定路径下的所有Git仓库
- 🚀 支持两种操作模式：
  - **MR模式**: 推送指定分支到远程仓库并创建GitLab merge request
  - **Tag模式**: 切换到指定分支并创建Git tag
- 📝 自动创建GitLab merge request
- 🏷️ 自动创建和推送Git tag
- 🔄 自动拉取分支最新代码
- 🎯 MR标题格式：`{源分支} to {目标分支}`
- 📊 详细的执行结果报告
- ⚡ 支持强制推送选项
- 🔄 Tag模式会自动切换回原来的分支

## 使用示例

### 示例1：创建Merge Request
```bash
./target/release/autoMR \
  --path /Users/username/projects \
  --source-branch feature/new-feature \
  --target-branch main \
  --gitlab-url https://gitlab.com/api/v4 \
  --gitlab-token glpat-xxxxxxxxxxxxxxxxxxxx \
  --mode mr
```

### 示例2：创建Tag
```bash
./target/release/autoMR \
  --path /Users/username/projects \
  --checkout-branch release \
  --tag-name v2.1.0 \
  --tag-message "Release version 2.1.0 with new features" \
  --mode tag
```

### 示例3：强制推送并创建MR
```bash
./target/release/autoMR \
  --path /Users/username/projects \
  --source-branch hotfix/critical-bug \
  --target-branch develop \
  --gitlab-url https://gitlab.company.com/api/v4 \
  --gitlab-token glpat-xxxxxxxxxxxxxxxxxxxx \
  --force \
  --mode mr
```

## 注意事项

- 确保所有仓库都配置了Git远程仓库
- **MR模式**: 确保源分支在本地存在，目标分支在远程存在
- **Tag模式**: 确保指定的分支在仓库中存在
- 确保有权限推送到远程仓库
- **MR模式**: 需要网络连接访问GitLab API
- **Tag模式**: 脚本会自动拉取分支最新代码，确保基于最新版本创建tag
- **Tag模式**: 脚本会自动切换回原来的分支，不会影响你的工作环境 