#!/bin/bash

# 检查上游 rustfs/rustfs 版本脚本
# 用于本地测试版本同步逻辑

set -e

UPSTREAM_REPO="rustfs/rustfs"
VERSION_API="https://version.rustfs.com/latest.json"

echo "🔍 检查上游仓库版本..."
echo "仓库: ${UPSTREAM_REPO}"
echo ""

echo "📡 获取上游最新版本..."
RAW_RESPONSE="$(curl -fsSL "$VERSION_API")"
UPSTREAM_VERSION="$(printf '%s' "$RAW_RESPONSE" | jq -r '.tag // .version // empty')"

if [ -z "$UPSTREAM_VERSION" ] || [ "$UPSTREAM_VERSION" = "null" ]; then
    echo "❌ 无法获取上游版本"
    echo "原始响应："
    echo "$RAW_RESPONSE"
    echo "请检查："
    echo "  1. 网络连接"
    echo "  2. latest.json 是否可访问"
    echo "  3. jq 是否已安装"
    exit 1
fi

echo "✅ 上游最新版本: $UPSTREAM_VERSION"
echo ""

# 获取当前仓库版本
echo "📦 检查当前仓库版本..."
CURRENT_VERSION=$(git describe --tags --abbrev=0 2>/dev/null || echo "v0.0.0")
echo "当前版本: $CURRENT_VERSION"
echo ""

# 比较版本
echo "🔄 版本比较..."
UPSTREAM_NORMALIZED="${UPSTREAM_VERSION#v}"
CURRENT_NORMALIZED="${CURRENT_VERSION#v}"

if [ "$UPSTREAM_NORMALIZED" != "$CURRENT_NORMALIZED" ]; then
    echo "🆕 发现新版本！"
    echo "  上游: $UPSTREAM_VERSION"
    echo "  当前: $CURRENT_VERSION"
    echo ""
    echo "建议操作："
    echo "  1. 运行 'git tag $UPSTREAM_VERSION' 创建新标签"
    echo "  2. 运行 'git push origin $UPSTREAM_VERSION' 推送标签"
    echo "  3. 或等待自动同步工作流执行（每小时一次）"
    exit 0
else
    echo "✅ 已是最新版本"
    echo "无需更新"
    exit 0
fi
