# API Structure

This document describes the current `ss-protocol` wire structure. It focuses on message envelopes and resource relationships. For the method list, see [reference.md](./reference.md).

## 1. Transport Model

The backend protocol uses JSON-RPC 2.0 for request/response envelopes and a separate server-event envelope for streaming output.

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

## 2. Resource Model

### 2.1 `llm_api`

`llm_api` is the persistent LLM API definition object.

Fields:

- `api_id`
- `provider`
- `base_url`
- `api_key`
- `model`

Read APIs never return the raw `api_key`. They return:

- `has_api_key`
- `api_key_masked`

Global config and session config only reference `api_id`.

### 2.2 `schema`

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

### 2.3 `player_profile`

`player_profile` is an independent switchable player setup resource.

Fields:

- `player_profile_id`
- `display_name`
- `description`

Notes:

- A story can work with multiple player profiles.
- A session activates at most one `player_profile_id` at a time.
- Switching player profiles does not switch `player_state`.

### 2.4 `character`

Character content is represented by `CharacterCardContent`:

- `id`
- `name`
- `personality`
- `style`
- `tendencies`
- `schema_id`
- `system_prompt`

Notes:

- Characters no longer embed `state_schema`.
- Character-private schema is referenced through `schema_id`.
- Cover retrieval and `.chr` export remain separate APIs.

### 2.5 `story_resources`

`story_resources` is the editable input bundle used before story generation.

Fields:

- `resource_id`
- `story_concept`
- `character_ids`
- `player_schema_id_seed`
- `world_schema_id_seed`
- `planned_story`

Notes:

- `character_ids` only reference existing character objects.
- Both schema seeds are optional ids.
- `planned_story` is optional planner output text.

### 2.6 `story`

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

### 2.7 `session`

A session binds a story to a runtime snapshot.

Fields:

- `session_id`
- `story_id`
- `display_name`
- `player_profile_id`
- `player_schema_id`
- `snapshot`
- `config`

Notes:

- `player_profile_id` may be `null`; that means the session currently uses a manually overridden player description.
- `player_schema_id` points to the player-state schema used by the session.
- `snapshot` stores dynamic runtime state, including `world_state`, `turn_index`, and the effective `player_description` text.

## 3. Character Card Archive `.chr`

`.chr` is a ZIP archive with fixed entries:

- `manifest.json`
- `content.json`
- `cover.<ext>`

`content.json` uses `CharacterCardContent`.

For details, see:

- [../character.md](../character.md)

## 4. Method Families

Current protocol families:

- `upload.*`
- `llm_api.*`
- `schema.*`
- `player_profile.*`
- `character.*`
- `story_resources.*`
- `story.*`
- `session.*`
- `config.*`
- `dashboard.get`

## 5. Session Semantics

### 5.1 Starting a Session

`story.start_session` accepts:

- `story_id`
- optional `display_name`
- optional `player_profile_id`
- `config_mode`
- optional `session_api_ids`

### 5.2 Switching Player Profile

`session.set_player_profile` only switches the active `player_profile_id` and the effective description. It does not switch `player_state`.

### 5.3 Manual Player Description Override

`session.update_player_description` directly overwrites the session description text and clears `player_profile_id`.

## 6. Delete Constraints

- `schema.delete`: returns `conflict` if the schema is still referenced by characters, resources, stories, or sessions
- `player_profile.delete`: returns `conflict` if any session still references it
- `character.delete`: returns `conflict` if any `story_resources` record still references it
- `story.delete`: returns `conflict` if any session still references it

