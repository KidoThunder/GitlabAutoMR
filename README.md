# Auto Merge Request & Tag 脚本

这是一个用Rust编写的命令行工具，用于批量创建GitLab merge request或批量创建Git tag。

## 功能特性

- 🔍 自动遍历指定路径下的所有Git仓库
- 🚀 支持四种操作模式：
  - **MR模式**: 推送指定分支到远程仓库并创建GitLab merge request
  - **Tag模式**: 切换到指定分支并创建Git tag
  - **List-MRs模式**: 列出所有由你创建的merge requests，支持状态筛选
  - **Approve-MRs模式**: 批量批准指定 Group 下、从源分支到目标分支的 Merge Requests
- 📝 自动创建GitLab merge request
- 🏷️ 自动创建和推送Git tag
- 📋 查看和筛选你创建的merge requests
- ✅ 按 GitLab Group 批量批准指定分支的 Merge Requests
- 🔄 自动拉取分支最新代码
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

### MR模式（创建Merge Request）

```bash
# 编译后的可执行文件
./target/release/autoMR \
  --path /path/to/your/repositories \
  --source-branch feature-branch \
  --target-branch main \
  --gitlab-url https://gitlab.com/api/v4 \
  --gitlab-token YOUR_GITLAB_TOKEN \
  --mode mr
```

### Tag模式（创建Tag）

```bash
# 编译后的可执行文件
./target/release/autoMR \
  --path /path/to/your/repositories \
  --checkout-branch release-branch \
  --tag-name v1.0.0 \
  --tag-message "Release version 1.0.0" \
  --mode tag
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

#### 通用参数
- `--path` / `-p`: 要遍历的根路径（MR 和 Tag 模式必需；list-mrs、approve-mrs 不需要）
- `--mode` / `-m`: 操作模式（可选，默认为 `mr`，支持 `mr`、`tag`、`list-mrs`、`approve-mrs`）

#### MR模式参数
- `--source-branch` / `-s`: 要推送的源分支名（MR模式必需）
- `--target-branch` / `-t`: 目标合并分支名（MR模式必需）
- `--gitlab-url` / `-g`: GitLab API URL（MR模式必需）
- `--gitlab-token` / `-k`: GitLab API Token（MR模式可选，也可通过环境变量设置）
- `--force` / `-f`: 是否强制推送（可选，默认为false）

#### Tag模式参数
- `--checkout-branch` / `-c`: 要切换到的分支名（Tag模式必需）
- `--tag-name` / `-n`: 要创建的tag名（Tag模式必需）
- `--tag-message` / `-m`: tag的注释信息（Tag模式可选）

#### List-MRs模式参数
- `--gitlab-url` / `-g`: GitLab API URL（list-mrs 模式必需）
- `--gitlab-token` / `-k`: GitLab API Token（可选，也可通过环境变量 `GITLAB_TOKEN` 设置）
- `--mr-state`: MR 状态筛选（可选，默认 `opened`；支持: opened, closed, locked, merged, all）

#### Approve-MRs模式参数
- `--gitlab-url` / `-g`: GitLab API URL（必需）
- `--group-path`: GitLab Group 路径（必需），例如 `server/lobby`，会包含子 Group 下的项目
- `--source-branch` / `-s`: 待审批 MR 的源分支（可选，默认 `dev`）
- `--target-branch` / `-t`: 待审批 MR 的目标分支（可选，默认 `release`）
- `--gitlab-token` / `-k`: GitLab API Token（可选，也可通过环境变量 `GITLAB_TOKEN` 设置）

### 环境变量

你也可以通过环境变量设置GitLab Token：

```bash
export GITLAB_TOKEN="your_gitlab_token_here"
./target/release/autoMR --path . --source-branch feature --target-branch main --gitlab-url https://gitlab.com/api/v4 --mode mr
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
  --gitlab-token glpat-xxxxxxxxxxxxxxxxxxxx \
  --mode mr
```

### 示例2：为所有仓库在release分支上创建tag

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

### 示例4：使用环境变量创建MR

```bash
export GITLAB_TOKEN="glpat-xxxxxxxxxxxxxxxxxxxx"
./target/release/autoMR \
  --path /path/to/repos \
  --source-branch release/v1.0.0 \
  --target-branch main \
  --gitlab-url https://gitlab.com/api/v4 \
  --mode mr
```

### 示例5：列出所有打开的Merge Requests

```bash
./target/release/autoMR \
  --mode list-mrs \
  --gitlab-url https://gitlab.com/api/v4 \
  --gitlab-token glpat-xxxxxxxxxxxxxxxxxxxx \
  --mr-state opened
```

### 示例6：列出所有已合并的Merge Requests

```bash
export GITLAB_TOKEN="glpat-xxxxxxxxxxxxxxxxxxxx"
./target/release/autoMR \
  --mode list-mrs \
  --gitlab-url https://gitlab.com/api/v4 \
  --mr-state merged
```

### 示例7：批量批准 Group 下 dev → release 的 MR

```bash
./target/release/autoMR \
  --mode approve-mrs \
  --gitlab-url https://gitlab.com/api/v4 \
  --gitlab-token glpat-xxxxxxxxxxxxxxxxxxxx \
  --group-path server/lobby
```

### 示例8：批量批准指定分支的 MR

```bash
./target/release/autoMR \
  --mode approve-mrs \
  --gitlab-url https://gitlab.com/api/v4 \
  --group-path my-group/subgroup \
  --source-branch feature/next \
  --target-branch main
```

## 输出示例

### MR模式输出

```
🚀 开始批量操作...
📁 搜索路径: /Users/username/projects
🔧 操作模式: mr
🌿 源分支: feature/new-feature
🎯 目标分支: main
📦 找到 3 个Git仓库

🔍 处理仓库: /Users/username/projects/project1
✅ 成功: project1: https://gitlab.com/group/project1/-/merge_requests/123

🔍 处理仓库: /Users/username/projects/project2
✅ 成功: project2: https://gitlab.com/group/project2/-/merge_requests/456

📊 处理完成!
✅ 成功创建 2 个Merge Request

📋 创建的Merge Request:
  - project1: https://gitlab.com/group/project1/-/merge_requests/123
  - project2: https://gitlab.com/group/project2/-/merge_requests/456
```

### Tag模式输出

```
🚀 开始批量操作...
📁 搜索路径: /Users/username/projects
🔧 操作模式: tag
🌿 切换分支: release
🏷️ 创建tag: v2.1.0
📝 Tag消息: Release version 2.1.0 with new features
📦 找到 3 个Git仓库

🔍 处理仓库: /Users/username/projects/project1
📍 当前分支: main
✅ 成功切换到分支: release
✅ 成功拉取分支 release 的最新代码
✅ 成功创建tag: v2.1.0
✅ 成功推送tag: v2.1.0
✅ 成功切换到分支: main
✅ 成功拉取分支 main 的最新代码
✅ 成功: /Users/username/projects/project1: 成功创建并推送tag v2.1.0

🔍 处理仓库: /Users/username/projects/project2
📍 当前分支: develop
✅ 成功切换到分支: release
✅ 成功拉取分支 release 的最新代码
✅ 成功创建tag: v2.1.0
✅ 成功推送tag: v2.1.0
✅ 成功切换到分支: develop
✅ 成功拉取分支 develop 的最新代码
✅ 成功: /Users/username/projects/project2: 成功创建并推送tag v2.1.0

📊 处理完成!
✅ 成功创建 2 个Tag

📋 创建的Tag:
  - /Users/username/projects/project1: 成功创建并推送tag v2.1.0
  - /Users/username/projects/project2: 成功创建并推送tag v2.1.0
```

### List-MRs模式输出

```
🚀 开始批量操作...
📁 搜索路径: 
🔧 操作模式: list-mrs
📋 MR状态筛选: opened

📊 找到 3 个Merge Request

📋 Merge Request列表:
====================================================================================================

🔹 MR #123 - Feature: Add user authentication
   状态: opened
   作者: John Doe (@johndoe)
   分支: feature/auth → main
   创建时间: 2024-12-01T10:30:00.000Z
   更新时间: 2024-12-05T15:45:00.000Z
   链接: https://gitlab.com/group/project1/-/merge_requests/123

🔹 MR #456 - Fix: Resolve login bug
   状态: opened
   作者: John Doe (@johndoe)
   分支: bugfix/login → develop
   创建时间: 2024-12-03T14:20:00.000Z
   更新时间: 2024-12-04T09:15:00.000Z
   链接: https://gitlab.com/group/project2/-/merge_requests/456

🔹 MR #789 - Update: Refactor API endpoints
   状态: opened
   作者: John Doe (@johndoe)
   分支: refactor/api → main
   创建时间: 2024-12-05T08:00:00.000Z
   更新时间: 2024-12-05T08:00:00.000Z
   链接: https://gitlab.com/group/project3/-/merge_requests/789

====================================================================================================
```

### Approve-MRs模式输出

```
🚀 开始批量操作...
🔧 操作模式: approve-mrs
📂 GitLab Group: server/lobby
🌿 源分支(待审批): dev
🎯 目标分支: release
📦 在 group `server/lobby` 下找到 5 个项目

🔍 处理项目: project-a (id: 101)
📋 在项目 project-a 中找到 1 个待审批的 MR (dev -> release)
  🔹 MR #12 - dev to release (dev -> release)
     ✅ 已批准 MR #12: https://gitlab.com/group/project-a/-/merge_requests/12
✅ project-a: 成功批准 1/1 个 MR (dev -> release)

🔍 处理项目: project-b (id: 102)
✅ project-b: 无待审批的 MR (dev -> release)

📊 处理完成!
✅ 已处理 5 个项目的 Merge Request 审批
```

## 注意事项

1. **GitLab Token**: 在 MR、list-mrs、approve-mrs 模式下，确保你的 GitLab Token 有足够权限
   - MR 模式需要: `api`, `read_repository`, `write_repository`
   - list-mrs 模式需要: `api`, `read_api`
   - approve-mrs 模式需要: `api`, `read_api`, 以及批准 MR 的权限
2. **网络连接**: 脚本需要网络连接来访问 GitLab API（MR、list-mrs、approve-mrs 模式）
3. **Git配置**: 确保所有仓库都正确配置了Git远程仓库（MR和Tag模式）
4. **分支存在**: 
   - MR模式：确保源分支在本地存在，目标分支在远程存在
   - Tag模式：确保指定的分支在仓库中存在
5. **权限**: 确保你有权限推送到远程仓库（MR和Tag模式）
6. **Tag模式**: 脚本会自动切换回原来的分支，不会影响你的工作环境
7. **list-mrs 模式**: 不需要指定 `--path`，会列出你在 GitLab 上创建的所有 MR
8. **approve-mrs 模式**: 不需要指定 `--path`；按 Group 路径列出项目（含子 Group），仅对「已打开」且源/目标分支匹配的 MR 调用 Approve API

## 错误处理

脚本会处理以下常见错误：
- Git仓库未找到（MR和Tag模式）
- 远程URL格式不支持（MR模式）
- GitLab API认证失败（MR和List-MRs模式）
- 分支推送失败（MR模式）
- Merge request创建失败（MR模式）
- Merge request 列表获取失败（list-mrs 模式）
- Group 项目列表获取失败（approve-mrs 模式）
- MR 批准失败（approve-mrs 模式）
- 分支切换失败（Tag 模式）
- Tag创建失败（Tag模式）
- Tag推送失败（Tag模式）

每个错误都会显示详细的错误信息，帮助你快速定位问题。

## 支持的Git URL格式

- SSH格式: `git@gitlab.com:group/project.git`
- HTTPS格式: `https://gitlab.com/group/project.git`

## 开发

如果你想修改或扩展这个脚本：

```bash
# 开发模式运行（MR模式）
cargo run -- --path . --source-branch test --target-branch main --gitlab-url https://gitlab.com/api/v4 --mode mr

# 开发模式运行（Tag模式）
cargo run -- --path . --checkout-branch release --tag-name v1.0.0 --mode tag

# 开发模式运行（List-MRs模式）
cargo run -- --mode list-mrs --gitlab-url https://gitlab.com/api/v4 --mr-state opened

# 开发模式运行（Approve-MRs模式）
cargo run -- --mode approve-mrs --gitlab-url https://gitlab.com/api/v4 --group-path server/lobby

# 运行测试
cargo test

# 检查代码
cargo check
``` 