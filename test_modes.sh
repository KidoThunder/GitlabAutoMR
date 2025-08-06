#!/bin/bash

echo "🧪 测试 AutoMR 工具的不同模式"
echo "================================"

# 编译项目
echo "📦 编译项目..."
cargo build --release

echo ""
echo "✅ 编译完成！"
echo ""

# 测试帮助信息
echo "📖 测试帮助信息:"
echo "----------------------------------------"
./target/release/autoMR --help

echo ""
echo "🔍 测试参数验证:"
echo "----------------------------------------"

# 测试MR模式缺少必需参数
echo "1️⃣ 测试MR模式缺少必需参数:"
./target/release/autoMR --path . --mode mr 2>&1 | head -5

echo ""
echo "2️⃣ 测试Tag模式缺少必需参数:"
./target/release/autoMR --path . --mode tag 2>&1 | head -5

echo ""
echo "3️⃣ 测试Tag模式完整参数（包含自动拉取）:"
./target/release/autoMR --path . --checkout-branch main --tag-name test-tag-v2 --tag-message "测试自动拉取功能" --mode tag

echo ""
echo "4️⃣ 测试无效模式:"
./target/release/autoMR --path . --mode invalid 2>&1 | head -3

echo ""
echo "🎉 测试完成！"
echo ""
echo "💡 使用提示:"
echo "- MR模式需要GitLab token和API URL"
echo "- Tag模式只需要本地Git操作，不需要网络连接"
echo "- 两种模式都会自动处理多个仓库" 