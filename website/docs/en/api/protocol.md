# API Structure

This page describes the current `ss-protocol` shape, with emphasis on envelopes, resource models, and session semantics.

## 1. Transport Model

The backend currently uses:

- JSON-RPC 2.0 on `POST /rpc`
- SSE for streaming responses
- binary file transfer on `/upload/{resource_id}/{file_id}` and `/download/{resource_id}/{file_id}`

### 1.1 Request

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "session_id": "session-1",
  "method": "story.generate",
  "params": {}
}
```

### 1.2 Unary Response

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "session_id": null,
  "result": {
    "type": "story_generated"
  }
}
```

### 1.3 Stream Response

`session.run_turn` returns a unary `ack` first, then sends `message` events.

Frame types:

- `started`
- `event`
- `completed`
- `failed`

### 1.4 Binary File Transfer

- upload and download routes do not use JSON-RPC
- public file identity is `resource_id + file_id`
- built-in resource files include:
  - `character:{character_id}/cover`
  - `character:{character_id}/archive`
  - `package_import:{import_id}/archive`
  - `package_export:{export_id}/archive`

## 2. Core Resource Model

### 2.1 Connection and Prompt Configuration

- `api`: one reusable connection definition
- `api_group`: per-agent `api_id` bindings
- `preset`: generation parameters plus modular prompt configuration

The runtime binding model uses `api_group_id + preset_id`.

### 2.2 State and Supporting Resources

- `schema`: standalone schema resource
- `lorebook`: reusable lore bundle
- `player_profile`: standalone player setup
- `resource_file`: transport-neutral binary file identity

### 2.3 Creative Resources

- `character`: character card content plus cover metadata
- `story_resources`: editable pre-generation input bundle
- `story`: final graph plus schema bindings
- `session`: story runtime snapshot and transcript

## 3. Method Families

Current protocol families:

- binary file routes
- `api.*`
- `api_group.*`
- `preset.*`
- `schema.*`
- `lorebook.*`
- `lorebook_entry.*`
- `player_profile.*`
- `character.*`
- `story_resources.*`
- `story.*`
- `story_draft.*`
- `session.*`
- `session_character.*`
- `session_message.*`
- `config.*`
- `dashboard.get`
- `data_package.*`

## 4. Session Semantics

### 4.1 Starting a Session

`story.start_session` accepts:

- `story_id`
- optional `display_name`
- optional `player_profile_id`
- optional `api_group_id`
- optional `preset_id`

If either binding id is omitted and resources exist, the backend uses the first available sorted id.

### 4.2 Draft Story Generation

`story_draft.*` is the preferred path for larger stories:

- `story_draft.start`
- `story_draft.update_graph`
- `story_draft.continue`
- `story_draft.finalize`

The server keeps `partial_graph` inside the draft object, so clients do not need to resend generated nodes.

### 4.3 Variable Panels

`story.common_variables` can be combined with `session.get_variables` or the runtime snapshot to render pinned variable panels.

### 4.4 Session Variable Updates

`session.update_variables` accepts variable ops only:

- `SetState`
- `RemoveState`
- `SetPlayerState`
- `RemovePlayerState`
- `SetCharacterState`
- `RemoveCharacterState`

Scene-control ops such as `SetCurrentNode` are rejected.
