# LLM Config: Model Name Input + Edit Mode + Test Connection

Date: 2026-04-19

## Problem

1. Setup StepLLM has no model name input — hardcoded to `gpt-4o`
2. ConfigPanel shows LLM config read-only — no way to edit after setup
3. No way to verify LLM config works before/after saving

## Solution

Three changes, all minimal:

### 1. StepLLM: Add Model Name Input

Add a plain text input between Provider buttons and API Key field.

- Default value changes with provider selection:
  - `openai` → `gpt-4o`
  - `anthropic` → `claude-sonnet-4-20250514`
  - `ollama` → `qwen2.5`
- User can override freely
- Add `[TEST]` button below the form fields

### 2. ConfigPanel: Read/Edit Toggle

- LLM Config section header gets `[EDIT]` button (right-aligned)
- Click EDIT → text display becomes form inputs:
  - Provider: button group (openai/anthropic/ollama)
  - Model: plain text input
  - API Key: password input (placeholder "Leave blank to keep current")
  - Base URL: text input
  - `[TEST CONNECTION]` button
- `[SAVE]` calls `PUT /api/config/llm`, `[CANCEL]` restores read-only
- Save/test errors shown in red text below buttons

### 3. Backend: Test Connection Endpoint

`POST /api/config/llm/test`

Request body:
```json
{
  "provider": "openai",
  "model": "gpt-4o",
  "api_key": "sk-xxx",
  "base_url": null
}
```

Response (success):
```json
{ "ok": true, "message": "Connection successful" }
```

Response (failure — 200 status, body indicates error):
```json
{ "ok": false, "message": "Invalid API key provided" }
```

Implementation: build a temporary `LlmClient` from the provided params, send a minimal chat completion request (system: "Respond with OK", user: "test"), timeout 15s. Catch and return the error message.

## Files Changed

| File | Change |
|------|--------|
| `server/src/models/config.rs` | Add `TestLLMRequest` struct |
| `server/src/services/llm.rs` | Add `test_connection()` method |
| `server/src/routes/config.rs` | Add `test_llm_config` handler |
| `server/src/routes/mod.rs` | Register `POST /api/config/llm/test` |
| `server/tests/integration.rs` | Add test for test endpoint |
| `ui/src/services/api.ts` | Add `testLLMConfig()` function |
| `ui/src/components/setup/StepLLM.tsx` | Add model input + test button |
| `ui/src/components/panels/ConfigPanel.tsx` | Add edit mode + test button |

## What Does NOT Change

- No DB migration needed
- No new stores
- No routing changes
- Existing `PUT /api/config/llm` and `GET /api/config/llm` unchanged
