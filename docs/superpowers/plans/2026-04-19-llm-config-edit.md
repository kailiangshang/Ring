# LLM Config Edit + Model Input + Test Connection

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add model name input to setup, editable LLM config in ConfigPanel, and a test connection button that verifies config against the actual LLM API.

**Architecture:** Backend adds a `POST /api/config/llm/test` endpoint that builds a temporary LlmClient and sends a minimal chat completion. Frontend adds model text input to StepLLM, edit mode toggle to ConfigPanel, and test buttons in both places.

**Tech Stack:** Rust + Axum (backend), React + TypeScript (frontend), async-openai (LLM client)

---

### Task 1: Add `TestLLMRequest` model + `test_connection` service

**Files:**
- Modify: `server/src/models/config.rs`
- Modify: `server/src/services/llm.rs`

- [ ] **Step 1: Add `TestLLMRequest` struct to `server/src/models/config.rs`**

Add after the `UpdateLLMConfig` struct (line 19):

```rust
#[derive(Debug, Deserialize)]
pub struct TestLLMRequest {
    pub provider: String,
    pub model: String,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}
```

- [ ] **Step 2: Add `test_connection` function to `server/src/services/llm.rs`**

Add at the end of the file, after the `LlmClient` impl block:

```rust
pub async fn test_connection(
    provider: &str,
    model: &str,
    api_key: Option<&str>,
    base_url: Option<&str>,
) -> crate::error::Result<(bool, String)> {
    let key = if provider == "ollama" {
        api_key.unwrap_or("ollama").to_string()
    } else {
        api_key
            .ok_or_else(|| crate::error::RingError::BadRequest("API key required".into()))?
            .to_string()
    };

    let mut config = OpenAIConfig::new().with_api_key(&key);
    if let Some(url) = base_url {
        if !url.is_empty() {
            config = config.with_api_base(url);
        }
    }
    if provider == "ollama" && base_url.is_none_or(|u| u.is_empty()) {
        config = config.with_api_base("http://localhost:11434/v1");
    }

    let client = Client::with_config(config);
    let messages = vec![
        ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
            content: ChatCompletionRequestSystemMessageContent::Text(
                "Respond with only the word OK.".into(),
            ),
            name: None,
        }),
        ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Text("test".into()),
            name: None,
        }),
    ];

    let request = CreateChatCompletionRequest {
        messages,
        model: model.to_string(),
        ..Default::default()
    };

    match tokio::time::timeout(
        std::time::Duration::from_secs(15),
        client.chat().create(request),
    )
    .await
    {
        Ok(Ok(_)) => Ok((true, "Connection successful".into())),
        Ok(Err(e)) => Ok((false, format!("{e}"))),
        Err(_) => Ok((false, "Connection timed out after 15s".into())),
    }
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check`
Expected: compiles without errors

- [ ] **Step 4: Commit**

```bash
git add server/src/models/config.rs server/src/services/llm.rs
git commit -m "Add TestLLMRequest model and test_connection service"
```

---

### Task 2: Add test endpoint + route registration

**Files:**
- Modify: `server/src/routes/config.rs`
- Modify: `server/src/routes/mod.rs`

- [ ] **Step 1: Add handler to `server/src/routes/config.rs`**

Add after the `update_llm_config` function (after line 25):

```rust
pub async fn test_llm_config(
    user: AuthUser,
    Json(body): Json<crate::models::config::TestLLMRequest>,
) -> Result<Json<serde_json::Value>> {
    let _ = &user;
    let (ok, message) = crate::services::llm::test_connection(
        &body.provider,
        &body.model,
        body.api_key.as_deref(),
        body.base_url.as_deref(),
    )
    .await?;
    Ok(Json(serde_json::json!({ "ok": ok, "message": message })))
}
```

- [ ] **Step 2: Register route in `server/src/routes/mod.rs`**

Add a new `.route()` after the existing `/config/llm` route (after line 53):

```rust
.route(
    "/config/llm/test",
    post(config::test_llm_config),
)
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check`
Expected: compiles without errors

- [ ] **Step 4: Commit**

```bash
git add server/src/routes/config.rs server/src/routes/mod.rs
git commit -m "Add POST /api/config/llm/test endpoint"
```

---

### Task 3: Add integration test for test endpoint

**Files:**
- Modify: `server/tests/integration.rs`

- [ ] **Step 1: Add test at the end of `server/tests/integration.rs`**

```rust
#[tokio::test]
async fn test_llm_test_endpoint_missing_key() {
    let state = setup_app().await;
    let app = build_router(state);
    let token = do_setup(&app).await;

    let body = r#"{"provider":"openai","model":"gpt-4o","api_key":null,"base_url":null}"#;
    let resp = app
        .clone()
        .oneshot(make_request(
            "POST",
            "/api/config/llm/test",
            Some(body),
            Some(&token),
        ))
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
}
```

- [ ] **Step 2: Run the test**

Run: `cargo test test_llm_test_endpoint_missing_key`
Expected: PASS

- [ ] **Step 3: Run all tests to verify nothing broke**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 4: Commit**

```bash
git add server/tests/integration.rs
git commit -m "Add integration test for LLM test endpoint"
```

---

### Task 4: Add `testLLMConfig` to frontend API

**Files:**
- Modify: `ui/src/services/api.ts`

- [ ] **Step 1: Add function at the end of `ui/src/services/api.ts`**

```typescript
export async function testLLMConfig(input: { provider: string; model: string; api_key?: string; base_url?: string }): Promise<{ ok: boolean; message: string }> {
  return api.post('/config/llm/test', input)
}
```

- [ ] **Step 2: Commit**

```bash
git add ui/src/services/api.ts
git commit -m "Add testLLMConfig API function"
```

---

### Task 5: Add model name input to StepLLM

**Files:**
- Modify: `ui/src/components/setup/StepLLM.tsx`

- [ ] **Step 1: Rewrite `StepLLM.tsx`**

The full replacement — adds model text input between provider buttons and API key, plus a TEST CONNECTION button:

```tsx
import { useState } from 'react'
import type { LLMProvider } from '../../types/config'
import type { SetupData } from './SetupWizard'
import { testLLMConfig } from '../../services/api'

interface StepProps {
  data: SetupData
  onChange: (partial: Partial<SetupData>) => void
  onNext: () => void
  onBack: () => void
}

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: 'var(--bg-input)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '8px 12px',
  color: 'var(--text-primary)',
  fontSize: 13,
  fontFamily: 'inherit',
  outline: 'none',
  marginBottom: 12,
  marginTop: 4,
}

const navButtonStyle: React.CSSProperties = {
  background: 'var(--bg-hover)',
  color: 'var(--text-primary)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '8px 20px',
  fontSize: 12,
  cursor: 'pointer',
  fontFamily: 'inherit',
}

const defaultModel = (p: string) => {
  if (p === 'anthropic') return 'claude-sonnet-4-20250514'
  if (p === 'ollama') return 'qwen2.5'
  return 'gpt-4o'
}

export function StepLLM({ data, onChange, onNext, onBack }: StepProps) {
  const provider = data.llm_provider as LLMProvider
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<{ ok: boolean; message: string } | null>(null)

  const handleProviderChange = (p: string) => {
    onChange({ llm_provider: p, llm_model: defaultModel(p) })
    setTestResult(null)
  }

  const handleTest = async () => {
    setTesting(true)
    setTestResult(null)
    try {
      const result = await testLLMConfig({
        provider: data.llm_provider,
        model: data.llm_model,
        api_key: provider !== 'ollama' ? data.llm_api_key || undefined : undefined,
        base_url: data.llm_base_url || undefined,
      })
      setTestResult(result)
    } catch (e: unknown) {
      setTestResult({ ok: false, message: e instanceof Error ? e.message : 'Test failed' })
    } finally {
      setTesting(false)
    }
  }

  return (
    <div style={{ padding: '20px', maxWidth: 400, margin: '0 auto' }}>
      <h2 style={{ fontSize: 16, color: 'var(--accent-ice)', marginBottom: 16 }}>
        Step 2: LLM Config
      </h2>

      <div style={{ display: 'flex', gap: 4, marginBottom: 16 }}>
        {(['openai', 'anthropic', 'ollama'] as const).map((p) => (
          <button
            key={p}
            onClick={() => handleProviderChange(p)}
            style={{
              background: provider === p ? 'var(--accent-cyan)' : 'var(--bg-hover)',
              color: provider === p ? 'var(--bg-base)' : 'var(--text-secondary)',
              border: '1px solid var(--border)',
              borderRadius: 4,
              padding: '6px 14px',
              fontSize: 12,
              cursor: 'pointer',
              fontWeight: provider === p ? 700 : 400,
            }}
          >
            {p}
          </button>
        ))}
      </div>

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>Model</label>
      <input
        value={data.llm_model}
        onChange={(e) => { onChange({ llm_model: e.target.value }); setTestResult(null) }}
        placeholder={defaultModel(provider)}
        style={inputStyle}
      />

      {provider !== 'ollama' && (
        <>
          <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>API Key</label>
          <input
            type="password"
            value={data.llm_api_key}
            onChange={(e) => { onChange({ llm_api_key: e.target.value }); setTestResult(null) }}
            placeholder={`sk-${provider === 'openai' ? 'xxx' : 'ant-xxx'}`}
            style={inputStyle}
          />
        </>
      )}

      <label style={{ fontSize: 11, color: 'var(--text-dim)' }}>
        Base URL {provider === 'ollama' ? '(e.g. http://localhost:11434)' : '(optional)'}
      </label>
      <input
        value={data.llm_base_url}
        onChange={(e) => { onChange({ llm_base_url: e.target.value }); setTestResult(null) }}
        placeholder={provider === 'ollama' ? 'http://localhost:11434' : ''}
        style={inputStyle}
      />

      <button
        onClick={handleTest}
        disabled={testing}
        style={{
          ...navButtonStyle,
          width: '100%',
          marginBottom: 8,
          opacity: testing ? 0.5 : 1,
          border: '1px solid var(--accent-cyan)',
          color: 'var(--accent-cyan)',
          background: 'transparent',
        }}
      >
        {testing ? 'TESTING...' : 'TEST CONNECTION'}
      </button>

      {testResult && (
        <div style={{
          fontSize: 11,
          padding: '6px 8px',
          borderRadius: 3,
          marginBottom: 12,
          background: testResult.ok ? 'rgba(34,197,94,0.1)' : 'rgba(239,68,68,0.1)',
          color: testResult.ok ? 'var(--accent-green)' : '#ef4444',
          border: `1px solid ${testResult.ok ? 'var(--accent-green)' : '#ef4444'}`,
        }}>
          {testResult.message}
        </div>
      )}

      <div style={{ display: 'flex', gap: 8, marginTop: 12 }}>
        <button onClick={onBack} style={navButtonStyle}>Back</button>
        <button
          onClick={onNext}
          disabled={provider !== 'ollama' && !data.llm_api_key.trim()}
          style={{
            ...navButtonStyle,
            opacity: provider !== 'ollama' && !data.llm_api_key.trim() ? 0.4 : 1,
            marginLeft: 'auto',
          }}
        >
          Next
        </button>
      </div>
    </div>
  )
}
```

- [ ] **Step 2: Verify TypeScript compiles**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Commit**

```bash
git add ui/src/components/setup/StepLLM.tsx
git commit -m "Add model name input and test connection to StepLLM"
```

---

### Task 6: Add edit mode + test to ConfigPanel

**Files:**
- Modify: `ui/src/components/panels/ConfigPanel.tsx`

- [ ] **Step 1: Rewrite ConfigPanel with edit mode**

Full replacement — adds EDIT/SAVE/CANCEL toggle, edit form with model input, and TEST CONNECTION:

```tsx
import { useEffect, useState } from 'react'
import type { Member } from '../../types/ring'
import type { LLMConfig, LLMProvider } from '../../types/config'
import { api, testLLMConfig } from '../../services/api'
import { useRingStore } from '../../stores/ring-store'
import { useInviteStore } from '../../stores/invite-store'

const inputStyle: React.CSSProperties = {
  width: '100%',
  background: 'var(--bg-input)',
  border: '1px solid var(--border)',
  borderRadius: 4,
  padding: '6px 10px',
  color: 'var(--text-primary)',
  fontSize: 12,
  fontFamily: 'inherit',
  outline: 'none',
  marginBottom: 8,
  marginTop: 2,
}

const smallBtn: React.CSSProperties = {
  background: 'var(--bg-hover)',
  color: 'var(--text-primary)',
  border: '1px solid var(--border)',
  borderRadius: 3,
  padding: '4px 10px',
  fontSize: 10,
  cursor: 'pointer',
  fontFamily: 'inherit',
}

export function ConfigPanel() {
  const active_ring_id = useRingStore((s) => s.active_ring_id)
  const rings = useRingStore((s) => s.rings)
  const [members, setMembers] = useState<Member[]>([])
  const [llmConfig, setLlmConfig] = useState<LLMConfig | null>(null)
  const [editing, setEditing] = useState(false)
  const [editProvider, setEditProvider] = useState<string>('openai')
  const [editModel, setEditModel] = useState('')
  const [editApiKey, setEditApiKey] = useState('')
  const [editBaseUrl, setEditBaseUrl] = useState('')
  const [testing, setTesting] = useState(false)
  const [testResult, setTestResult] = useState<{ ok: boolean; message: string } | null>(null)
  const [saveError, setSaveError] = useState<string | null>(null)

  const tokens = useInviteStore((s) => s.tokens)
  const join_requests = useInviteStore((s) => s.join_requests)
  const fetch_tokens = useInviteStore((s) => s.fetch_tokens)
  const revoke_token = useInviteStore((s) => s.revoke_token)
  const fetch_requests = useInviteStore((s) => s.fetch_requests)
  const approve_request = useInviteStore((s) => s.approve_request)
  const reject_request = useInviteStore((s) => s.reject_request)
  const open_modal = useInviteStore((s) => s.open_modal)

  const active_ring = rings.find((r) => r.id === active_ring_id)
  const is_admin = active_ring?.role === 'creator' || active_ring?.role === 'admin'

  const loadLlm = () => {
    api.get<{ provider: string; model: string; api_key_set: boolean; base_url: string | null }>('/config/llm')
      .then((res) => {
        setLlmConfig({ ...res, provider: res.provider as LLMProvider })
        if (!editing) {
          setEditProvider(res.provider)
          setEditModel(res.model)
          setEditBaseUrl(res.base_url || '')
        }
      })
      .catch(() => {})
  }

  useEffect(() => { loadLlm() }, [])

  useEffect(() => {
    if (!active_ring_id) return
    api.get<{ members: Member[] }>(`/rings/${active_ring_id}/members`)
      .then((res) => setMembers(res.members))
      .catch(() => {})
    if (is_admin) {
      fetch_tokens(active_ring_id)
      fetch_requests(active_ring_id)
    }
  }, [active_ring_id, is_admin])

  const startEdit = () => {
    if (!llmConfig) return
    setEditProvider(llmConfig.provider)
    setEditModel(llmConfig.model)
    setEditBaseUrl(llmConfig.base_url || '')
    setEditApiKey('')
    setTestResult(null)
    setSaveError(null)
    setEditing(true)
  }

  const cancelEdit = () => {
    setEditing(false)
    setTestResult(null)
    setSaveError(null)
  }

  const handleTest = async () => {
    setTesting(true)
    setTestResult(null)
    try {
      const result = await testLLMConfig({
        provider: editProvider,
        model: editModel,
        api_key: editApiKey || undefined,
        base_url: editBaseUrl || undefined,
      })
      setTestResult(result)
    } catch (e: unknown) {
      setTestResult({ ok: false, message: e instanceof Error ? e.message : 'Test failed' })
    } finally {
      setTesting(false)
    }
  }

  const handleSave = async () => {
    setSaveError(null)
    try {
      const body: Record<string, string> = {
        provider: editProvider,
        model: editModel,
      }
      if (editApiKey) body.api_key = editApiKey
      if (editBaseUrl) body.base_url = editBaseUrl
      await api.put<LLMConfig>('/config/llm', body)
      setEditing(false)
      loadLlm()
    } catch (e: unknown) {
      setSaveError(e instanceof Error ? e.message : 'Save failed')
    }
  }

  const time_remaining = (expires_at: string) => {
    const diff = new Date(expires_at).getTime() - Date.now()
    if (diff <= 0) return 'expired'
    const hours = Math.floor(diff / 3600000)
    if (hours > 24) return `${Math.floor(hours / 24)}d left`
    return `${hours}h left`
  }

  return (
    <div style={{ fontSize: 12 }}>
      <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between', marginBottom: 8 }}>
        <span style={{ color: 'var(--text-secondary)', fontWeight: 700 }}>LLM Config</span>
        {!editing && (
          <span style={{ ...smallBtn, color: 'var(--accent-cyan)', borderColor: 'var(--accent-cyan)' }} onClick={startEdit}>EDIT</span>
        )}
      </div>

      {!editing && llmConfig && (
        <div style={{ marginBottom: 16, color: 'var(--text-primary)', lineHeight: 1.8 }}>
          <div>Provider: <span style={{ color: 'var(--accent-ice)' }}>{llmConfig.provider}</span></div>
          <div>Model: <span style={{ color: 'var(--accent-ice)' }}>{llmConfig.model}</span></div>
          <div>API Key: {llmConfig.api_key_set ? '✓' : '✗'}</div>
        </div>
      )}

      {editing && (
        <div style={{ marginBottom: 16 }}>
          <div style={{ display: 'flex', gap: 4, marginBottom: 10 }}>
            {(['openai', 'anthropic', 'ollama'] as const).map((p) => (
              <button
                key={p}
                onClick={() => setEditProvider(p)}
                style={{
                  background: editProvider === p ? 'var(--accent-cyan)' : 'var(--bg-hover)',
                  color: editProvider === p ? 'var(--bg-base)' : 'var(--text-secondary)',
                  border: '1px solid var(--border)',
                  borderRadius: 3,
                  padding: '4px 10px',
                  fontSize: 10,
                  cursor: 'pointer',
                  fontWeight: editProvider === p ? 700 : 400,
                }}
              >
                {p}
              </button>
            ))}
          </div>

          <label style={{ fontSize: 10, color: 'var(--text-dim)' }}>Model</label>
          <input value={editModel} onChange={(e) => setEditModel(e.target.value)} style={inputStyle} />

          <label style={{ fontSize: 10, color: 'var(--text-dim)' }}>API Key</label>
          <input
            type="password"
            value={editApiKey}
            onChange={(e) => setEditApiKey(e.target.value)}
            placeholder="Leave blank to keep current"
            style={inputStyle}
          />

          <label style={{ fontSize: 10, color: 'var(--text-dim)' }}>Base URL</label>
          <input value={editBaseUrl} onChange={(e) => setEditBaseUrl(e.target.value)} style={inputStyle} />

          <button
            onClick={handleTest}
            disabled={testing}
            style={{
              ...smallBtn,
              width: '100%',
              marginBottom: 8,
              marginTop: 4,
              border: '1px solid var(--accent-cyan)',
              color: 'var(--accent-cyan)',
              background: 'transparent',
              opacity: testing ? 0.5 : 1,
            }}
          >
            {testing ? 'TESTING...' : 'TEST CONNECTION'}
          </button>

          {testResult && (
            <div style={{
              fontSize: 10,
              padding: '4px 8px',
              borderRadius: 3,
              marginBottom: 8,
              background: testResult.ok ? 'rgba(34,197,94,0.1)' : 'rgba(239,68,68,0.1)',
              color: testResult.ok ? 'var(--accent-green)' : '#ef4444',
              border: `1px solid ${testResult.ok ? 'var(--accent-green)' : '#ef4444'}`,
            }}>
              {testResult.message}
            </div>
          )}

          {saveError && (
            <div style={{
              fontSize: 10,
              padding: '4px 8px',
              borderRadius: 3,
              marginBottom: 8,
              background: 'rgba(239,68,68,0.1)',
              color: '#ef4444',
              border: '1px solid #ef4444',
            }}>
              {saveError}
            </div>
          )}

          <div style={{ display: 'flex', gap: 6 }}>
            <span style={{ ...smallBtn, background: 'var(--accent-cyan)', color: 'var(--bg-base)', borderColor: 'var(--accent-cyan)' }} onClick={handleSave}>SAVE</span>
            <span style={smallBtn} onClick={cancelEdit}>CANCEL</span>
          </div>
        </div>
      )}

      <p style={{ marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
        Members
      </p>
      {members.map((m) => (
        <div key={m.token_id} style={{ display: 'flex', alignItems: 'center', gap: 8, padding: '4px 0', color: 'var(--text-primary)' }}>
          <span>{m.display_name}</span>
          <span style={{ color: 'var(--text-dim)', fontSize: 11 }}>({m.role})</span>
          {m.online && <span style={{ color: 'var(--accent-green)', fontSize: 10 }}>●</span>}
        </div>
      ))}
      {members.length === 0 && <p style={{ color: 'var(--text-dim)' }}>No members</p>}

      {is_admin && (
        <div
          style={{ marginTop: 8, padding: '5px 8px', border: '1px solid var(--accent-cyan)', borderRadius: 3, textAlign: 'center', color: 'var(--accent-cyan)', cursor: 'pointer', fontSize: 10 }}
          onClick={open_modal}
        >
          + invite member
        </div>
      )}

      {is_admin && tokens.length > 0 && (
        <>
          <p style={{ marginTop: 16, marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
            Active Invites · {tokens.length}
          </p>
          {tokens.map((t) => (
            <div key={t.token} style={{ padding: '6px 8px', border: '1px solid var(--border)', borderRadius: 3, marginBottom: 3, display: 'flex', alignItems: 'center', gap: 6, fontSize: 10 }}>
              <span style={{ color: t.type === 'open' ? 'var(--accent-cyan)' : 'var(--accent-amber)', fontSize: 9 }}>{t.type}</span>
              <span style={{ flex: 1, color: 'var(--text-muted)', fontSize: 9 }}>{t.use_count}/{t.max_uses} uses · {time_remaining(t.expires_at)}</span>
              <span style={{ color: 'var(--text-dim)', fontSize: 9, cursor: 'pointer' }} onClick={() => revoke_token(active_ring_id!, t.token)}>revoke</span>
            </div>
          ))}
        </>
      )}

      {is_admin && join_requests.length > 0 && (
        <>
          <p style={{ marginTop: 16, marginBottom: 8, color: 'var(--text-secondary)', fontWeight: 700 }}>
            Pending Requests · {join_requests.length}
          </p>
          {join_requests.map((req) => (
            <div key={req.id} style={{ padding: 8, border: '1px solid var(--accent-amber)', borderRadius: 3, marginBottom: 3 }}>
              <div style={{ display: 'flex', alignItems: 'center', gap: 6, marginBottom: 4, fontSize: 10 }}>
                <span style={{ color: 'var(--text-primary)', fontWeight: 500 }}>{req.display_name}</span>
                <span style={{ color: 'var(--accent-amber)', fontSize: 8 }}>audit</span>
              </div>
              {req.message && <div style={{ color: 'var(--text-muted)', fontSize: 9, marginBottom: 6 }}>"{req.message}"</div>}
              <div style={{ display: 'flex', gap: 6 }}>
                <div style={{ flex: 1, padding: 4, background: 'var(--accent-green)', color: 'var(--bg-base)', borderRadius: 2, textAlign: 'center', fontSize: 9, fontWeight: 700, cursor: 'pointer' }} onClick={() => approve_request(active_ring_id!, req.id)}>APPROVE</div>
                <div style={{ flex: 1, padding: 4, border: '1px solid var(--border)', borderRadius: 2, textAlign: 'center', fontSize: 9, color: 'var(--text-secondary)', cursor: 'pointer' }} onClick={() => { const note = window.prompt('Rejection reason (optional):'); reject_request(active_ring_id!, req.id, note || undefined) }}>REJECT</div>
              </div>
            </div>
          ))}
        </>
      )}
    </div>
  )
}
```

- [ ] **Step 2: Verify TypeScript compiles**

Run: `cd ui && npx tsc --noEmit`
Expected: no errors

- [ ] **Step 3: Verify frontend build**

Run: `cd ui && npm run build`
Expected: build succeeds

- [ ] **Step 4: Commit**

```bash
git add ui/src/components/panels/ConfigPanel.tsx
git commit -m "Add edit mode and test connection to ConfigPanel"
```

---

### Task 7: Run full test suite + final verification

- [ ] **Step 1: Run backend tests**

Run: `cargo test`
Expected: all tests pass

- [ ] **Step 2: Run frontend tests**

Run: `cd ui && npm test`
Expected: all tests pass (1 pre-existing failure in command-parser is acceptable)

- [ ] **Step 3: Run cargo clippy**

Run: `cargo clippy -- -D warnings`
Expected: no warnings

- [ ] **Step 4: Run cargo fmt check**

Run: `cargo fmt --check`
Expected: no formatting issues

- [ ] **Step 5: Run frontend lint**

Run: `cd ui && npx eslint src/`
Expected: no errors
