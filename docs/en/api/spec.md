# API Structure

This document describes the current `ss-protocol` wire structure. It focuses on message envelopes and resource relationships. For the method list, see [reference.md](./reference.md).

## 1. Transport Model

The backend uses JSON-RPC 2.0 for protocol request/response envelopes, a separate server-event
envelope for streaming output, and dedicated binary HTTP routes for file transfer.

### 1.1 Request

Every request is a JSON-RPC request object:

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "session_id": "session-1",
  "method": "story.generate",
  "params": {}
}
```

- `id`: client-generated request id.
- `session_id`: required only for session-bound methods.
- `method`: protocol method name.
- `params`: method-specific parameter object.

### 1.2 Unary Response

Unary responses follow the JSON-RPC response shape:

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "session_id": null,
  "result": {
    "type": "story_generated",
    "...": "..."
  }
}
```

Error response:

```json
{
  "jsonrpc": "2.0",
  "id": "req-1",
  "session_id": null,
  "error": {
    "code": "conflict",
    "message": "schema 'schema-player-default' is still referenced",
    "details": null,
    "retryable": false
  }
}
```

### 1.3 Stream Response

Streaming methods such as `session.run_turn` return a unary `ack` first, then emit server events:

```json
{
  "message_type": "stream",
  "request_id": "req-turn",
  "session_id": "session-1",
  "sequence": 3,
  "frame": {
    "type": "event",
    "event": {
      "type": "actor_dialogue_delta",
      "...": "..."
    }
  }
}
```

Stream frame types:

- `started`
- `event`
- `completed`
- `failed`

`completed` includes the final aggregate turn result, so the frontend does not need to rebuild it from deltas.

Session-bound APIs can also return ordinary unary JSON-RPC results. The current example is:

- `session.suggest_replies`
  - requires top-level `session_id`
  - `params`:
    - `limit?: number`
  - returns:
    - `type = "suggested_replies"`
    - `replies: [{ reply_id, text }]`
  - notes:
    - it only generates suggestions and does not write them into the session transcript
    - it returns 3 suggestions by default and accepts `2..=5`

### 1.4 Binary File Transfer

Routes under `/upload/{resource_id}/{file_id}` and `/download/{resource_id}/{file_id}` do not use
JSON-RPC envelopes.

- Upload request bodies are raw bytes.
- `resource_id + file_id` is the protocol-level file identity.
- `x-file-name` is an optional upload request header.
- Download responses return raw bytes with HTTP `Content-Type`.
- Binary route failures return plain `ErrorPayload` JSON bodies with transport HTTP status codes.
- Current built-in resource files:
  - `character:{character_id}/cover`
  - `character:{character_id}/archive`

## 2. Resource Model

### 2.1 `api`

`api` is the persistent reusable connection definition.

Fields:

- `api_id`
- `display_name`
- `provider`
- `base_url`
- `api_key`
- `model`

Read APIs never return the raw `api_key`. They return:

- `has_api_key`
- `api_key_masked`

Helper method:

- `api.list_models` accepts `provider`, `base_url`, and `api_key`
- It returns `provider`, normalized `base_url`, and `models: string[]`
- It does not create or update a stored `api`

### 2.2 `api_group`

`api_group` is the persistent per-agent API binding bundle.

Fields:

- `api_group_id`
- `display_name`
- `bindings`

`bindings` contains one `api_id` per runtime agent:

- `planner_api_id`
- `architect_api_id`
- `director_api_id`
- `actor_api_id`
- `narrator_api_id`
- `keeper_api_id`
- `replyer_api_id`

### 2.3 `preset`

`preset` is the persistent per-agent generation-parameter bundle.

Fields:

- `preset_id`
- `display_name`
- `agents`

Each agent entry currently supports:

- `temperature`
- `max_tokens`
- optional `extra`

The runtime binding model now uses `api_group_id + preset_id`.

If a request omits one of those ids and at least one resource exists, the backend sorts ids and
uses the first available value.

### 2.4 `schema`

`schema` is now an independent resource and is no longer embedded into characters, resources, or stories.

Fields:

- `schema_id`
- `display_name`
- `tags: string[]`
- `fields`

Notes:

- A schema does not have a built-in `kind`.
- `tags` are user-facing labels such as `player`, `world`, or `character`.
- `fields` follow the `StateFieldSchema` structure.
- `StateFieldSchema` supports `value_type`, optional `default`, optional `description`, and optional `enum_values`.
- `enum_values` is currently only valid for scalar types: `bool`, `int`, `float`, `string`.
- If `enum_values` is present, `default` must also match the field type and be one of the allowed values.

### 2.5 `lorebook`

`lorebook` is a persistent reusable lore bundle.

Fields:

- `lorebook_id`
- `display_name`
- `entries`

Each `entry` contains:

- `entry_id`
- `title`
- `content`
- `keywords: string[]`
- `enabled`
- `always_include`

Notes:

- `lorebook.update` updates base metadata such as `display_name`
- `lorebook_entry.*` methods operate on entries inside one `lorebook`
- `keywords` defaults to `[]`
- `enabled` defaults to `true`
- `always_include` defaults to `false`

### 2.6 `player_profile`

`player_profile` is an independent switchable player setup resource.

Fields:

- `player_profile_id`
- `display_name`
- `description`

Notes:

- A story can work with multiple player profiles.
- A session activates at most one `player_profile_id` at a time.
- Switching player profiles does not switch `player_state`.

### 2.7 `resource_file`

`resource_file` is the transport-neutral public identity for binary transfer.

Fields:

- `resource_id`
- `file_id`
- `file_name`
- `content_type`
- `size_bytes`

Notes:

- Public APIs identify files through `resource_id + file_id`, not through internal blob ids.
- `POST /upload/{resource_id}/{file_id}` returns `ResourceFilePayload`.
- `GET /download/{resource_id}/{file_id}` returns the raw bytes for that logical file.
- Current built-in resource files:
  - `character:{character_id}/cover`
  - `character:{character_id}/archive`

### 2.8 `character`

Character content is represented by `CharacterCardContent`:

- `id`
- `name`
- `personality`
- `style`
- `schema_id`
- `system_prompt`

Character read payloads add optional cover metadata around that content:

- `cover_file_name`
- `cover_mime_type`

Notes:

- Characters no longer embed `state_schema`.
- Character-private schema is referenced through `schema_id`.
- Cover bytes are not embedded in JSON-RPC payloads.
- Fetch cover bytes through `GET /download/character:{character_id}/cover`.
- Import and export `.chr` files through:
  - `POST /upload/character:{character_id}/archive`
  - `GET /download/character:{character_id}/archive`

### 2.9 `story_resources`

`story_resources` is the editable input bundle used before story generation.

Fields:

- `resource_id`
- `story_concept`
- `character_ids`
- `player_schema_id_seed`
- `world_schema_id_seed`
- `lorebook_ids`
- `planned_story`

Notes:

- `character_ids` only reference existing character objects.
- Both schema seeds are optional ids.
- `lorebook_ids` only reference existing lorebooks and may be empty.
- `planned_story` is optional planner output text.

### 2.10 `story`

A generated `story` record contains:

- `story_id`
- `display_name`
- `resource_id`
- `graph`
- `world_schema_id`
- `player_schema_id`
- `introduction`

Notes:

- After `Architect` generates world/player schema content, the engine manager stores them as `schema` resources first.
- The story itself stores only schema ids.

### 2.11 `session`

A session binds a story to a runtime snapshot.

Fields:

- `session_id`
- `story_id`
- `display_name`
- `player_profile_id`
- `player_schema_id`
- `snapshot`
- `history`
- `created_at_ms`
- `updated_at_ms`
- `config`

Notes:

- `player_profile_id` may be `null`; that means the session currently uses a manually overridden player description.
- `player_schema_id` points to the player-state schema used by the session.
- `snapshot` stores dynamic runtime state, including `world_state`, `turn_index`, and the effective `player_description` text.
- `history` stores the visible session transcript in chronological order:
  - player input
  - narration
  - visible actor actions
  - actor dialogue
- `history` is assembled from standalone `session_message` records
- `created_at_ms` / `updated_at_ms` are Unix timestamps in milliseconds.
- session-scoped temporary characters are stored separately from the story graph and exist only within the session runtime.

## 3. Character Card Archive `.chr`

`.chr` is a ZIP archive with fixed entries:

- `manifest.json`
- `content.json`
- `cover.<ext>`

`content.json` uses `CharacterCardContent`.

Import and export happen through:

- `POST /upload/character:{character_id}/archive`
- `GET /download/character:{character_id}/archive`

For details, see:

- [../character.md](../character.md)

## 4. Method Families

Current protocol families:

- binary file routes under `/upload/{resource_id}/{file_id}` and `/download/{resource_id}/{file_id}`
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

## 5. Session Semantics

### 5.1 Starting a Session

`story.start_session` accepts:

- `story_id`
- optional `display_name`
- optional `player_profile_id`
- optional `api_group_id`
- optional `preset_id`

If either binding id is omitted and the backend has at least one `api_group` and one `preset`,
it uses the first available id from each list after sorting.

`story.generate` accepts the same creation inputs as draft start:

- `resource_id`
- optional `display_name`
- optional `api_group_id`
- optional `preset_id`
- optional `common_variables`

`story.update` only updates story metadata. For now it supports:

- `story_id`
- optional `display_name`
- optional `common_variables`

Each `common_variables` entry contains:

- `scope`: `world | player | character`
- `key`
- `display_name`
- optional `character_id`
- optional `pinned` (defaults to `true`)

Validation rules:

- `world` and `player` entries must not set `character_id`
- `character` entries must set `character_id`
- `character_id` must belong to one of the story's bound characters
- `key` must exist in the schema bound to that scope
- duplicate entries for the same bound variable are rejected

Clients can combine `story.common_variables` with `session.get_variables` or the runtime snapshot to
render a pinned variable panel without hardcoding schema keys.

`story.update_graph` replaces the full `graph` field of an existing story, including each node's
`on_enter_updates`.
The backend validates the graph before saving and returns `invalid_request` if:

- `start_node` does not exist
- any transition points to a missing node
- duplicate node ids are present

### 5.1.1 Draft Story Generation

`story_draft.*` is the preferred generation flow when a story is large enough that a single Architect call would become too expensive.

- `story_draft.start` creates a server-side draft and generates the first section
- `story_draft.start` may also persist caller-supplied `common_variables`
- `story_draft.update_graph` replaces the draft's current `partial_graph`, including node
  `on_enter_updates`
- `story_draft.continue` appends one more outline section to the partial graph
- `story_draft.finalize` validates the merged graph and creates the final `story`
- `story.generate` remains available as a compatibility wrapper around the full draft flow
- `story_draft.start` persists the chosen `api_group_id` and `preset_id` in the draft
- draft detail responses also include the persisted `common_variables`
- `story_draft.finalize` copies draft `common_variables` into the final `story`

`story_draft.update_graph` uses the same graph validation as `story.update_graph`, and rejects finalized drafts.

### 5.2 Switching Player Profile

`session.set_player_profile` only switches the active `player_profile_id` and the effective description. It does not switch `player_state`.

### 5.3 Manual Player Description Override

`session.update_player_description` directly overwrites the session description text and clears `player_profile_id`.

### 5.4 Updating Session Metadata

`session.update` only updates session metadata. For now it supports changing `display_name`.

### 5.5 Reading and Editing Session Variables

`session.get_variables` returns the current mutable conversation variables from the session snapshot:

- `custom`
- `player_state`
- `character_state`

`session.update_variables` applies a `StateUpdate` to those same variable maps.

Only variable ops are allowed:

- `SetState`
- `RemoveState`
- `SetPlayerState`
- `RemovePlayerState`
- `SetCharacterState`
- `RemoveCharacterState`

Scene-control ops such as `SetCurrentNode` and `SetActiveCharacters` are rejected.

### 5.6 Managing Session Characters

`session_character.*` manages temporary runtime-only roles inside a session.

- the default creation path is `Director` `role_actions.create_and_enter` during `session.run_turn`
- created session characters can enter the current scene immediately and participate in the same turn
- `session_character.enter_scene` and `session_character.leave_scene` only change current-scene presence
- session characters do not modify `story`, `story_draft`, or `character`

### 5.7 Editing Session Transcript Messages

`session_message.*` manages standalone transcript messages for a session.

- `session_message.create` appends a visible message to the end of the transcript
- `session_message.get` and `session_message.list` return transcript message resources
- `session_message.update` edits transcript data in place
- `session_message.delete` removes a message from the transcript
- `session.suggest_replies` uses the most recent 8 transcript messages as its history window

Important:

- transcript edits do not replay history
- transcript edits do not mutate `snapshot` or `world_state`

## 6. Delete Constraints

- `schema.delete`: returns `conflict` if the schema is still referenced by characters, resources, stories, or sessions
- `player_profile.delete`: returns `conflict` if any session still references it
- `character.delete`: returns `conflict` if any `story_resources` record still references it
- `story.delete`: returns `conflict` if any session still references it
