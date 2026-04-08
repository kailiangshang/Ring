# Auth 中间件 API 参考

> 源码路径：`ring-server/src/middleware/auth.rs`

## AuthUser

### `struct AuthUser`
源文件：`middleware/auth.rs:7`

| 字段 | 类型 | 说明 |
|------|------|------|
| `user_id` | `String` | 用户 ID |

通过 Axum Extension 机制传递，handler 中通过 `Extension<AuthUser>` 提取。

---

## auth_middleware

### `async fn auth_middleware(request: Request, next: Next) -> Response`
源文件：`middleware/auth.rs:12`

**认证流程**：
1. 从请求头 `X-User-Id` 获取用户 ID
2. 如果存在：插入 `AuthUser` extension → 执行后续 handler
3. 如果不存在：返回 HTTP 401 Unauthorized，`{ "error": "missing X-User-Id header" }`

**使用方式**（在 `routes.rs` 中）：
```rust
.layer(middleware::from_fn(auth_middleware))
```
