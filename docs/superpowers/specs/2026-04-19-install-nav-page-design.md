# 安装导航页 设计

> **Affects**: `server/src/routes/join_page.rs`, `server/src/routes/mod.rs`
> **Depends on**: `verify_join_token`（已实现）, `invite_tokens` 表
> **Last verified**: 2026-04-19

## 1. 概述

安装导航页是邀请链接的目标页面，由创建者 ring-server 托管。被邀请人点击邀请链接后看到此页面，包含 Ring 信息、平台下载链接和三步安装引导。

实现方式：**内联 HTML**（不使用 React），Rust handler 动态生成 HTML 响应。

### 1.1 架构

```
被邀请人浏览器                创建者 ring-server
     │                            │
     │ GET /ring/join?token=xxx   │
     │───────────────────────────>│
     │                            │ verify_join_token()
     │                            │ 检测 User-Agent
     │                            │ 渲染 HTML
     │  HTML page                 │
     │<───────────────────────────│
     │                            │
     │ （点击"继续加入"按钮）       │
     │ GET localhost:7420/ring/join
     │ ?token=xxx&creator_ip=...  │
     │ ──> 加入者本地 ring-server  │
```

### 1.2 设计决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 路由位置 | `/ring/join`（在 `/api` 嵌套之外） | 非 API 端点，是 HTML 页面 |
| 实现方式 | 内联 HTML 模板 | 单二进制分发，无需额外前端资源 |
| 下载链接 | GitHub Releases | PRD 定义：统一从 GitHub 下载 |
| OS 检测 | User-Agent 解析 | 服务端判断，最可靠 |
| "继续加入"按钮 | 跳转 `localhost:7420/ring/join?token=xxx&creator_ip={server_ip}` | PRD 定义的 P2P 流程 |
| Token 无效时 | 仍然返回 HTML 页面，显示错误信息 | 用户体验优于 JSON 错误 |
| `creator_ip` 来源 | `Host` header 中的 IP 部分 | 创建者 ring-server 知道自己的地址 |

## 2. 路由

### 2.1 GET /ring/join?token=xxx

公开端点，无需认证。在 `/api` 嵌套之外，在 `fallback_service` 之前注册。

**查询参数**：
- `token` — invite token（必需）
- `creator_ip` — 可选，如果带此参数说明是"继续加入"按钮的跳转目标

**响应**：`Content-Type: text/html; charset=utf-8`

### 2.2 路由注册位置

```
Router::new()
    .nest("/api", api)
    .route("/ring/join", get(join_page::join_page_handler))   // ← 新增
    .fallback_service(ServeDir::new(...))                       // 现有
    .layer(cors)
    .layer(TraceLayer::new_for_http())
```

必须在 `fallback_service` 之前，否则会被前端静态资源捕获。

## 3. 页面内容

### 3.1 有效 Token

页面包含以下区域：

**标题区**：
- Ring 图标 + "Ring" 品牌名
- 邀请类型标签：`open link` 或 `audit link`

**信息区**：
- Ring 名称（大字）
- 成员数量
- 角色（member / admin）

**三步引导区**（未安装用户）：
1. "下载 Ring" — 根据检测到的 OS 高亮对应按钮
2. "解压并运行 ring-server" — 终端命令 `./ring-server`
3. "点击下方按钮加入" — "继续加入"按钮

**下载按钮区**：
- Windows: `ring-server-windows-amd64.zip`
- macOS (Apple Silicon): `ring-server-macos-arm64.tar.gz`
- macOS (Intel): `ring-server-macos-amd64.tar.gz`
- Linux: `ring-server-linux-amd64.tar.gz`

检测到的 OS 对应按钮高亮，其他按钮灰色。

**"继续加入"按钮**：
- 链接到 `http://localhost:7420/ring/join?token=xxx&creator_ip={host_ip}`
- 按钮文案：`继续加入 "{ring_name}"`
- 底部说明："Ring 需在本地运行后才能加入"

### 3.2 无效 / 过期 / 已撤销 Token

显示错误页面：
- 标题："邀请链接无效"
- 错误原因（token 过期 / 已撤销 / 不存在）
- 提示："请联系 Ring 创建者获取新的邀请链接"

### 3.3 缺少 Token 参数

显示错误页面：
- 标题："缺少邀请令牌"
- 提示："请使用有效的邀请链接"

## 4. OS 检测逻辑

从 `User-Agent` header 解析：

```rust
fn detect_os(user_agent: &str) -> &'static str {
    let ua = user_agent.to_lowercase();
    if ua.contains("windows") {
        "windows"
    } else if ua.contains("mac os x") || ua.contains("macos") {
        if ua.contains("arm") || ua.contains("apple") {
            "macos-arm64"
        } else {
            "macos-amd64"
        }
    } else if ua.contains("linux") {
        "linux"
    } else {
        "unknown"
    }
}
```

macOS Apple Silicon 检测不精确时默认推荐 arm64（当前 Mac 主流）。

## 5. HTML 模板

使用 `format!()` 宏生成 HTML，嵌入 CSS 内联样式。

**视觉风格**：深色背景（IceChat 主题变体），但保持简洁，不引入额外 CSS 框架。

**关键样式**：
- 背景：`#0d1117`（深色）
- 文字：`#e6edf3`
- 主色按钮：`#1f6feb`
- 高亮下载按钮：`#238636`（绿色）
- 非高亮下载按钮：`#21262d`（灰色）
- 字体：`-apple-system, BlinkMacSystemFont, 'Segoe UI', Helvetica, Arial, sans-serif`
- 最大宽度：`480px`，居中

**HTML 结构**（简化）：
```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>加入 {ring_name} - Ring</title>
  <style>/* 内联 CSS */</style>
</head>
<body>
  <div class="container">
    <!-- 错误页面 或 邀请页面 -->
  </div>
</body>
</html>
```

## 6. `creator_ip` 提取

从请求的 `Host` header 提取 IP：

```rust
fn extract_creator_ip(host: Option<&str>) -> Option<String> {
    host.and_then(|h| h.split(':').next().map(|s| s.to_string()))
}
```

移除端口号（`:7420`），只保留 IP 部分。

## 7. GitHub Releases URL 模板

```
https://github.com/{owner}/ring/releases/latest/download/{filename}
```

`owner` 使用占位符 `RING_GITHUB_OWNER`，默认 `ring-hub`。
`filename` 取值：
- `ring-server-windows-amd64.zip`
- `ring-server-macos-arm64.tar.gz`
- `ring-server-macos-amd64.tar.gz`
- `ring-server-linux-amd64.tar.gz`

## 8. 错误处理

| 场景 | 页面行为 |
|------|---------|
| 缺少 `token` 参数 | 显示"缺少邀请令牌"错误页 |
| Token 不存在 | 显示"邀请链接无效"错误页 |
| Token 过期 | 显示"邀请链接已过期"错误页 |
| Token 已撤销 | 显示"邀请链接已被撤销"错误页 |
| `verify_join_token` 内部错误 | 显示"服务器错误"错误页 |

所有错误情况返回 HTTP 200 + HTML 错误页面（非 JSON 错误），确保浏览器友好。

## 9. 修改文件清单

| 文件 | 改动 | 说明 |
|------|------|------|
| `server/src/routes/join_page.rs` | 新建 | join page handler + HTML 渲染 |
| `server/src/routes/mod.rs` | 修改 | 注册 `/ring/join` 路由 |
| `server/tests/integration.rs` | 修改 | 新增 join page 测试 |
