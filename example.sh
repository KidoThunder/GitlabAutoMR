#!/bin/bash

# Auto Merge Request 使用示例脚本

echo "🚀 Auto Merge Request 工具使用示例"
echo "=================================="

# 检查是否已编译
if [ ! -f "./target/release/autoMR" ]; then
    echo "📦 正在编译项目..."
    cargo build --release
fi

# 显示帮助信息
echo ""
echo "📖 显示帮助信息:"
echo "----------------------------------------"
./target/release/autoMR --help

echo ""
echo "💡 使用示例:"
echo "----------------------------------------"

# 示例1：基本用法
echo "1️⃣ 基本用法:"
echo "./target/release/autoMR \\"
echo "  --path /path/to/your/repositories \\"
echo "  --source-branch feature-branch \\"
echo "  --target-branch main \\"
echo "  --gitlab-url https://gitlab.com/api/v4 \\"
echo "  --gitlab-token YOUR_GITLAB_TOKEN"

echo ""
echo "2️⃣ 使用环境变量:"
echo "export GITLAB_TOKEN=\"your_gitlab_token_here\""
echo "./target/release/autoMR \\"
echo "  --path . \\"
echo "  --source-branch feature/new-feature \\"
echo "  --target-branch main \\"
echo "  --gitlab-url https://gitlab.com/api/v4"

echo ""
echo "3️⃣ 强制推送:"
echo "./target/release/autoMR \\"
echo "  --path /Users/username/projects \\"
echo "  --source-branch hotfix/critical-bug \\"
echo "  --target-branch develop \\"
echo "  --gitlab-url https://gitlab.company.com/api/v4 \\"
echo "  --gitlab-token glpat-xxxxxxxxxxxxxxxxxxxx \\"
echo "  --force"

echo ""
echo "🔑 GitLab Token 获取步骤:"
echo "1. 登录到你的GitLab账户"
echo "2. 进入 Settings > Access Tokens"
echo "3. 创建一个新的Personal Access Token"
echo "4. 确保Token具有以下权限："
echo "   - api - 访问API"
echo "   - read_repository - 读取仓库信息"
echo "   - write_repository - 创建merge request"

echo ""
echo "⚠️  注意事项:"
echo "- 确保所有仓库都正确配置了Git远程仓库"
echo "- 确保源分支在本地存在，目标分支在远程存在"
echo "- 确保你有权限推送到远程仓库"
echo "- 脚本需要网络连接来访问GitLab API" 