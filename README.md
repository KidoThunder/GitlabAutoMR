# Auto Merge Request 脚本

这是一个用Rust编写的命令行工具，用于批量创建GitLab merge request。

## 功能特性

- 🔍 自动遍历指定路径下的所有Git仓库
- 🚀 推送指定分支到远程仓库
- 📝 自动创建GitLab merge request
- 🎯 支持自定义源分支和目标分支
- 🔧 支持强制推送选项
- 📊 详细的执行结果报告

## 安装

确保你的系统已安装Rust和Cargo。

```bash
# 克隆或下载项目后，在项目目录中运行
cargo build --release
```

## 使用方法

### 基本用法

```bash
# 编译后的可执行文件
./target/release/autoMR \
  --path /path/to/your/repositories \
  --source-branch feature-branch \
  --target-branch main \
  --gitlab-url https://gitlab.com/api/v4 \
  --gitlab-token YOUR_GITLAB_TOKEN
```

### 参数说明

- `--path` / `-p`: 要遍历的根路径（必需）
- `--source-branch` / `-s`: 要推送的源分支名（必需）
- `--target-branch` / `-t`: 目标合并分支名（必需）
- `--gitlab-url` / `-g`: GitLab API URL（必需）
- `--gitlab-token` / `-k`: GitLab API Token（可选，也可通过环境变量设置）
- `--force` / `-f`: 是否强制推送（可选，默认为false）

### 环境变量

你也可以通过环境变量设置GitLab Token：

```bash
export GITLAB_TOKEN="your_gitlab_token_here"
./target/release/autoMR --path . --source-branch feature --target-branch main --gitlab-url https://gitlab.com/api/v4
```

## GitLab Token 获取

1. 登录到你的GitLab账户
2. 进入 **Settings** > **Access Tokens**
3. 创建一个新的Personal Access Token
4. 确保Token具有以下权限：
   - `api` - 访问API
   - `read_repository` - 读取仓库信息
   - `write_repository` - 创建merge request

## 示例

### 示例1：为当前目录下的所有仓库创建MR

```bash
./target/release/autoMR \
  --path . \
  --source-branch feature/new-feature \
  --target-branch main \
  --gitlab-url https://gitlab.com/api/v4 \
  --gitlab-token glpat-xxxxxxxxxxxxxxxxxxxx
```

### 示例2：强制推送并创建MR

```bash
./target/release/autoMR \
  --path /Users/username/projects \
  --source-branch hotfix/critical-bug \
  --target-branch develop \
  --gitlab-url https://gitlab.company.com/api/v4 \
  --gitlab-token glpat-xxxxxxxxxxxxxxxxxxxx \
  --force
```

### 示例3：使用环境变量

```bash
export GITLAB_TOKEN="glpat-xxxxxxxxxxxxxxxxxxxx"
./target/release/autoMR \
  --path /path/to/repos \
  --source-branch release/v1.0.0 \
  --target-branch main \
  --gitlab-url https://gitlab.com/api/v4
```

## 输出示例

```
🚀 开始批量创建Merge Request...
📁 搜索路径: /Users/username/projects
🌿 源分支: feature/new-feature
🎯 目标分支: main
📦 找到 3 个Git仓库

🔍 处理仓库: /Users/username/projects/project1
✅ 成功: project1: https://gitlab.com/group/project1/-/merge_requests/123

🔍 处理仓库: /Users/username/projects/project2
✅ 成功: project2: https://gitlab.com/group/project2/-/merge_requests/456

🔍 处理仓库: /Users/username/projects/project3
❌ 失败: Failed to push branch feature/new-feature in /Users/username/projects/project3

📊 处理完成!
✅ 成功创建 2 个Merge Request

📋 创建的Merge Request:
  - project1: https://gitlab.com/group/project1/-/merge_requests/123
  - project2: https://gitlab.com/group/project2/-/merge_requests/456
```

## 注意事项

1. **GitLab Token**: 确保你的GitLab Token有足够的权限来创建merge request
2. **网络连接**: 脚本需要网络连接来访问GitLab API
3. **Git配置**: 确保所有仓库都正确配置了Git远程仓库
4. **分支存在**: 确保源分支在本地存在，目标分支在远程存在
5. **权限**: 确保你有权限推送到远程仓库

## 错误处理

脚本会处理以下常见错误：
- Git仓库未找到
- 远程URL格式不支持
- GitLab API认证失败
- 分支推送失败
- Merge request创建失败

每个错误都会显示详细的错误信息，帮助你快速定位问题。

## 支持的Git URL格式

- SSH格式: `git@gitlab.com:group/project.git`
- HTTPS格式: `https://gitlab.com/group/project.git`

## 开发

如果你想修改或扩展这个脚本：

```bash
# 开发模式运行
cargo run -- --path . --source-branch test --target-branch main --gitlab-url https://gitlab.com/api/v4

# 运行测试
cargo test

# 检查代码
cargo check
``` 