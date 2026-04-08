# Tool Engine API 参考

> 源码路径：`ring-server/src/services/tool_engine/`

## Tool Trait

### `trait Tool`
源文件：`services/tool_engine/mod.rs:13`

| 方法 | 签名 | 说明 |
|------|------|------|
| `definition` | `() -> ToolDefinition` | 返回工具定义（名称、描述、参数 Schema） |
| `execute` | `(input) -> Result<serde_json::Value>` | 执行工具，返回 JSON 结果 |

---

## ToolRegistry

### `struct ToolRegistry`
源文件：`services/tool_engine/registry.rs:7`

| 字段 | 类型 | 说明 |
|------|------|------|
| `tools` | `HashMap<String, Arc<dyn Tool>>` | 工具注册表 |

### `impl ToolRegistry`
源文件：`services/tool_engine/registry.rs:11`

- `fn new() -> Self` — 构造函数
- `fn register(tool: Arc<dyn Tool>)` — 注册工具（按 definition().name 索引）
- `fn get(name) -> Option<Arc<dyn Tool>>` — 获取工具
- `fn definitions() -> Vec<ToolDefinition>` — 获取所有工具定义

---

## ToolDispatcher

### `struct ToolDispatcher`
源文件：`services/tool_engine/dispatcher.rs:6`

| 字段 | 类型 | 说明 |
|------|------|------|
| `registry` | `Arc<ToolRegistry>` | 工具注册表 |

### `impl ToolDispatcher`
源文件：`services/tool_engine/dispatcher.rs:10`

- `fn new(registry) -> Self` — 构造函数
- `fn definitions() -> Vec<ToolDefinition>` — 获取所有工具定义
- `async fn dispatch(call) -> ToolResultRecord` — 调度工具执行
  - 查找工具并调用 `execute(input)`
  - 成功：`success: true` + `output`
  - 失败：`success: false` + 错误 JSON
  - 工具不存在：返回 unknown tool 错误

---

## 工具实现

### SearchTool
源文件：`services/tool_engine/tools/search_tool.rs`

| 字段 | 类型 | 说明 |
|------|------|------|
| `repo` | `Arc<dyn Repository>` | 数据库 |

- **工具名**：`search`
- **描述**：Full-text search knowledge graph nodes
- **参数**：`{ query: string, graph_ids?: string[], limit?: integer }`
- **返回**：`{ results: SearchResult[] }`

---

### TextCleanTool
源文件：`services/tool_engine/tools/text_clean_tool.rs`

- **工具名**：`text_clean`
- **描述**：Clean and normalize text by stripping extra whitespace and normalizing unicode
- **参数**：`{ text: string }`
- **返回**：`{ cleaned_text: string }`

---

### WebScrapeTool
源文件：`services/tool_engine/tools/web_scrape_tool.rs`

| 字段 | 类型 | 说明 |
|------|------|------|
| `client` | `Client` | HTTP 客户端 |

- **工具名**：`web_scrape`
- **描述**：Fetch a web page and extract its title and text content
- **参数**：`{ url: string }`
- **返回**：`{ title: string, text: string }`（提取 p、h1-h6、li、td 标签内容）

---

### MarkdownGenTool
源文件：`services/tool_engine/tools/markdown_gen_tool.rs`

- **工具名**：`markdown_gen`
- **描述**：Generate formatted markdown from a title and sections
- **参数**：
  ```json
  {
    "title": "string",
    "sections": [{ "heading": "string", "body": "string" }]
  }
  ```
- **返回**：`{ markdown: string }`（格式：`# {title}\n\n## {heading}\n\n{body}`）

---

### PrivacyFilterTool
源文件：`services/tool_engine/tools/privacy_filter_tool.rs`

| 字段 | 类型 | 说明 |
|------|------|------|
| `email_re` | `Regex` | 邮箱正则 |
| `phone_re` | `Regex` | 手机号正则（中国 1[3-9] 开头） |
| `id_card_re` | `Regex` | 身份证号正则（18位） |

- **工具名**：`privacy_filter`
- **描述**：Redact PII (email, phone, ID card) from text
- **参数**：`{ text: string }`
- **返回**：`{ filtered_text: string, redactions_count: number }`

---

## 预注册工具（main.rs 中）

源文件：`main.rs:70-76`

在 `AppState` 构建时注册 5 个工具：

```rust
registry.register(Arc::new(SearchTool::new(db.clone())));
registry.register(Arc::new(TextCleanTool::new()));
registry.register(Arc::new(WebScrapeTool::new()));
registry.register(Arc::new(MarkdownGenTool::new()));
registry.register(Arc::new(PrivacyFilterTool::new()));
```
