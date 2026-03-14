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

Session-bound APIs can also return ordinary unary JSON-RPC results. The current example is:

- `session.suggest_replies`
  - requires top-level `session_id`
  - `params`:
    - `limit?: number`
    - `api_overrides?: AgentApiIdOverrides`
  - returns:
    - `type = "suggested_replies"`
    - `replies: [{ reply_id, text }]`
  - notes:
    - it only generates suggestions and does not write them into the session transcript
    - it returns 3 suggestions by default and accepts `2..=5`

## 2. Resource Model

### 2.1 `llm_api`

`llm_api` is the persistent LLM API definition object.

Fields:

- `api_id`
- `provider`
- `base_url`
- `api_key`
- `model`
- `temperature`
- `max_tokens`

Read APIs never return the raw `api_key`. They return:

- `has_api_key`
- `api_key_masked`

Global config and session config only reference `api_id`.

`llm_api.create` may omit `provider`, `base_url`, `api_key`, `model`, `temperature`, or
`max_tokens`. Missing values are filled from the current effective `default_llm_config`.

If no global config exists yet, successfully creating the first `llm_api` automatically binds
that `api_id` to every agent role.

The currently supported generation defaults on an `llm_api` object are:

- `temperature`
- `max_tokens`

Current `AgentApiIds` / `AgentApiIdOverrides` fields:

- `planner_api_id`
- `architect_api_id`
- `director_api_id`
- `actor_api_id`
- `narrator_api_id`
- `keeper_api_id`
- `replyer_api_id`

Notes:

- global config is now allowed to be absent
- when it is absent, read APIs return `api_ids = null`
- only agent-executing APIs require global config to be initialized

### 2.1.1 `default_llm_config`

`default_llm_config` is a singleton default template for new `llm_api` records.

Fields:

- `provider`
- `base_url`
- `api_key`
- `model`
- `temperature`
- `max_tokens`

API shape:

- `default_llm_config.get` returns:
  - `saved`: the persisted default config in store, optional
  - `effective`: the runtime default config after env/file overrides, optional
- `default_llm_config.update` replaces the `saved` config only

Notes:

- env/file overrides take precedence over the saved config
- env/file overrides do not write back into store

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
- `story_draft.*`
- `session.*`
- `session_message.*`
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

`story.update` only updates story metadata. For now it supports:

- `story_id`
- `display_name`

### 5.1.1 Draft Story Generation

`story_draft.*` is the preferred generation flow when a story is large enough that a single Architect call would become too expensive.

- `story_draft.start` creates a server-side draft and generates the first section
- `story_draft.continue` appends one more outline section to the partial graph
- `story_draft.finalize` validates the merged graph and creates the final `story`
- `story.generate` remains available as a compatibility wrapper around the full draft flow

### 5.2 Switching Player Profile

`session.set_player_profile` only switches the active `player_profile_id` and the effective description. It does not switch `player_state`.

### 5.3 Manual Player Description Override

`session.update_player_description` directly overwrites the session description text and clears `player_profile_id`.

### 5.4 Updating Session Metadata

`session.update` only updates session metadata. For now it supports changing `display_name`.

### 5.5 Editing Session Transcript Messages

`session_message.*` manages standalone transcript messages for a session.

- `session_message.create` appends a visible message to the end of the transcript
- `session_message.get` and `session_message.list` return transcript message resources
- `session_message.update` edits transcript data in place
- `session_message.delete` removes a message from the transcript

Important:

- transcript edits do not replay history
- transcript edits do not mutate `snapshot` or `world_state`

## 6. Delete Constraints

- `schema.delete`: returns `conflict` if the schema is still referenced by characters, resources, stories, or sessions
- `player_profile.delete`: returns `conflict` if any session still references it
- `character.delete`: returns `conflict` if any `story_resources` record still references it
- `story.delete`: returns `conflict` if any session still references it
