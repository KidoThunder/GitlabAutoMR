#!/bin/bash

# Auto Merge Request & Tag 使用示例脚本

echo "🚀 Auto Merge Request & Tag 工具使用示例"
echo "=========================================="

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

# 示例1：MR模式基本用法
echo "1️⃣ MR模式 - 基本用法:"
echo "./target/release/autoMR \\"
echo "  --path /path/to/your/repositories \\"
echo "  --source-branch feature-branch \\"
echo "  --target-branch main \\"
echo "  --gitlab-url https://gitlab.com/api/v4 \\"
echo "  --gitlab-token YOUR_GITLAB_TOKEN \\"
echo "  --mode mr"

echo ""
echo "2️⃣ Tag模式 - 创建Tag:"
echo "./target/release/autoMR \\"
echo "  --path /path/to/your/repositories \\"
echo "  --checkout-branch release-branch \\"
echo "  --tag-name v1.0.0 \\"
echo "  --tag-message \"Release version 1.0.0\" \\"
echo "  --mode tag"

echo ""
echo "3️⃣ MR模式 - 使用环境变量:"
echo "export GITLAB_TOKEN=\"your_gitlab_token_here\""
echo "./target/release/autoMR \\"
echo "  --path . \\"
echo "  --source-branch feature/new-feature \\"
echo "  --target-branch main \\"
echo "  --gitlab-url https://gitlab.com/api/v4 \\"
echo "  --mode mr"

echo ""
echo "4️⃣ MR模式 - 强制推送:"
echo "./target/release/autoMR \\"
echo "  --path /Users/username/projects \\"
echo "  --source-branch hotfix/critical-bug \\"
echo "  --target-branch develop \\"
echo "  --gitlab-url https://gitlab.company.com/api/v4 \\"
echo "  --gitlab-token glpat-xxxxxxxxxxxxxxxxxxxx \\"
echo "  --force \\"
echo "  --mode mr"

echo ""
echo "5️⃣ Tag模式 - 批量创建版本tag:"
echo "./target/release/autoMR \\"
echo "  --path /Users/username/projects \\"
echo "  --checkout-branch release \\"
echo "  --tag-name v2.1.0 \\"
echo "  --tag-message \"Release version 2.1.0 with new features\" \\"
echo "  --mode tag"

echo ""
echo "🔑 GitLab Token 获取步骤（仅MR模式需要）:"
echo "1. 登录到你的GitLab账户"
echo "2. 进入 Settings > Access Tokens"
echo "3. 创建一个新的Personal Access Token"
echo "4. 确保Token具有以下权限："
echo "   - api - 访问API"
echo "   - read_repository - 读取仓库信息"
echo "   - write_repository - 创建merge request"

echo ""
echo "⚠️  注意事项:"
echo "MR模式:"
echo "- 确保所有仓库都正确配置了Git远程仓库"
echo "- 确保源分支在本地存在，目标分支在远程存在"
echo "- 确保你有权限推送到远程仓库"
echo "- 脚本需要网络连接来访问GitLab API"
echo ""
echo "Tag模式:"
echo "- 确保所有仓库都正确配置了Git远程仓库"
echo "- 确保指定的分支在仓库中存在"
echo "- 确保你有权限推送到远程仓库"
echo "- 脚本会自动切换回原来的分支，不会影响你的工作环境" 