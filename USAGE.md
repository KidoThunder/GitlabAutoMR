# Auto Merge Request 使用说明

## 快速开始

1. **编译程序**
   ```bash
   cargo build --release
   ```

2. **获取GitLab Token**
   - 登录GitLab → Settings → Access Tokens
   - 创建Personal Access Token，确保有 `api` 权限

3. **运行程序**
   ```bash
   ./target/release/playground \
     --path /path/to/repositories \
     --source-branch feature-branch \
     --target-branch main \
     --gitlab-url https://gitlab.com/api/v4 \
     --gitlab-token YOUR_TOKEN
   ```

## 参数说明

| 参数 | 短参数 | 必需 | 说明 |
|------|--------|------|------|
| `--path` | `-p` | ✅ | 要遍历的根路径 |
| `--source-branch` | `-s` | ✅ | 要推送的源分支名 |
| `--target-branch` | `-t` | ✅ | 目标合并分支名 |
| `--gitlab-url` | `-g` | ✅ | GitLab API URL |
| `--gitlab-token` | `-k` | ❌ | GitLab API Token (也可用环境变量) |
| `--force` | `-f` | ❌ | 强制推送 |

## 环境变量

```bash
export GITLAB_TOKEN="your_token_here"
./target/release/playground --path . --source-branch feature --target-branch main --gitlab-url https://gitlab.com/api/v4
```

## 功能特性

- 🔍 自动遍历指定路径下的所有Git仓库
- 🚀 推送指定分支到远程仓库
- 📝 自动创建GitLab merge request
- 🎯 MR标题格式：`{源分支} to {目标分支}`
- 📊 详细的执行结果报告
- ⚡ 支持强制推送选项

## 注意事项

- 确保所有仓库都配置了Git远程仓库
- 确保源分支在本地存在，目标分支在远程存在
- 确保有权限推送到远程仓库
- 需要网络连接访问GitLab API 