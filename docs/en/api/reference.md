# API Reference

This document lists the currently implemented API surface in `ss-protocol`. Method names and field names match the current code.

## 1. Request Method Overview

| Method | `session_id` | Success result | Stream |
| --- | --- | --- | --- |
| `upload.init` | No | `upload_initialized` | No |
| `upload.chunk` | No | `upload_chunk_accepted` | No |
| `upload.complete` | No | `character_card_uploaded` | No |
| `character.create` | No | `character_created` | No |
| `character.get` | No | `character` | No |
| `character.set_cover` | No | `character_cover_updated` | No |
| `character.get_cover` | No | `character_cover` | No |
| `character.export_chr` | No | `character_chr_export` | No |
| `character.list` | No | `characters_listed` | No |
| `character.delete` | No | `character_deleted` | No |
| `story_resources.create` | No | `story_resources_created` | No |
| `story_resources.get` | No | `story_resources` | No |
| `story_resources.list` | No | `story_resources_listed` | No |
| `story_resources.update` | No | `story_resources_updated` | No |
| `story_resources.delete` | No | `story_resources_deleted` | No |
| `story.generate_plan` | No | `story_planned` | No |
| `story.generate` | No | `story_generated` | No |
| `story.get` | No | `story` | No |
| `story.list` | No | `stories_listed` | No |
| `story.delete` | No | `story_deleted` | No |
| `story.start_session` | No | `session_started` | No |
| `session.get` | Yes | `session` | No |
| `session.list` | No | `sessions_listed` | No |
| `session.delete` | Yes | `session_deleted` | No |
| `session.run_turn` | Yes | `turn_stream_accepted` | Yes |
| `session.update_player_description` | Yes | `player_description_updated` | No |
| `session.get_runtime_snapshot` | Yes | `runtime_snapshot` | No |
| `config.get_global` | No | `global_config` | No |
| `config.update_global` | No | `global_config` | No |
| `session.get_config` | Yes | `session_config` | No |
| `session.update_config` | Yes | `session_config` | No |

## 2. Upload API

### 2.1 `upload.init`

Params:

```json
{
  "target_kind": "character_card",
  "file_name": "merchant.chr",
  "content_type": "application/octet-stream",
  "total_size": 123456,
  "sha256": "..."
}
```

Result:

```json
{
  "type": "upload_initialized",
  "upload_id": "upload-0",
  "chunk_size_hint": 65536
}
```

### 2.2 `upload.chunk`

Params:

```json
{
  "upload_id": "upload-0",
  "chunk_index": 0,
  "offset": 0,
  "payload_base64": "...",
  "is_last": false
}
```

Result:

```json
{
  "type": "upload_chunk_accepted",
  "upload_id": "upload-0",
  "received_chunk_index": 0,
  "received_bytes": 65536
}
```

### 2.3 `upload.complete`

Params:

```json
{
  "upload_id": "upload-0"
}
```

Result:

```json
{
  "type": "character_card_uploaded",
  "character_id": "merchant",
  "character_summary": {
    "character_id": "merchant",
    "name": "Old Merchant",
    "personality": "greedy but friendly trader",
    "style": "talkative, casual, slightly cunning",
    "tendencies": [],
    "cover_file_name": "cover.png",
    "cover_mime_type": "image/png"
  }
}
```

## 3. Character API

### 3.1 `character.create`

Params:

```json
{
  "content": {
    "id": "merchant",
    "name": "Old Merchant",
    "personality": "greedy but friendly trader",
    "style": "talkative, casual, slightly cunning",
    "tendencies": [],
    "state_schema": {},
    "system_prompt": "Stay in character."
  }
}
```

Result: `character_created`

- `character_id`
- `character_summary`

This method creates character content only. A cover is not required at creation time, so `character_summary.cover_file_name` and `character_summary.cover_mime_type` may be `null`.

### 3.2 `character.get`

Params:

```json
{
  "character_id": "merchant"
}
```

Result: `character`

- `character_id`
- `content`
- `cover_file_name`
- `cover_mime_type`

`content` matches the `.chr` `content.json`.
If the character does not have a cover yet, `cover_file_name` and `cover_mime_type` are `null`.

### 3.3 `character.set_cover`

Params:

```json
{
  "character_id": "merchant",
  "cover_mime_type": "image/png",
  "cover_base64": "..."
}
```

Result: `character_cover_updated`

- `character_id`
- `cover_file_name`
- `cover_mime_type`

The server derives the cover file name from the mime type, for example `image/png -> cover.png`.

### 3.4 `character.get_cover`

Params:

```json
{
  "character_id": "merchant"
}
```

Result: `character_cover`

- `character_id`
- `cover_file_name`
- `cover_mime_type`
- `cover_base64`

`cover_base64` is the base64-encoded binary content of the character cover.
If the character does not have a cover yet, this method returns a `conflict` error.

### 3.5 `character.export_chr`

Params:

```json
{
  "character_id": "merchant"
}
```

Result: `character_chr_export`

- `character_id`
- `file_name`
- `content_type`
- `chr_base64`

`chr_base64` is the base64-encoded full `.chr` file content.
If the character does not have a cover yet, this method returns a `conflict` error.

### 3.6 `character.list`

Params: `{}`

Result: `characters_listed`

- `characters: CharacterCardSummaryPayload[]`

`cover_file_name` and `cover_mime_type` in each summary may be `null`.

### 3.7 `character.delete`

Params:

```json
{
  "character_id": "merchant"
}
```

Result: `character_deleted`

## 4. Story Resources API

### 4.1 `story_resources.create`

Params:

```json
{
  "story_concept": "A tense negotiation in a flooded city.",
  "character_ids": ["merchant", "guard"],
  "player_state_schema_seed": {},
  "world_state_schema_seed": null,
  "planned_story": null
}
```

Result: `story_resources_created`

Fields:

- `resource_id`
- `story_concept`
- `character_ids`
- `player_state_schema_seed`
- `world_state_schema_seed`
- `planned_story`

### 4.2 `story_resources.get`

Params: `{ "resource_id": "resource-0" }`

Result: `story_resources`

### 4.3 `story_resources.list`

Params: `{}`

Result: `story_resources_listed`

- `resources: StoryResourcesPayload[]`

### 4.4 `story_resources.update`

All update fields are optional:

- `story_concept`
- `character_ids`
- `player_state_schema_seed`
- `world_state_schema_seed`
- `planned_story`

Result: `story_resources_updated`

### 4.5 `story_resources.delete`

Params: `{ "resource_id": "resource-0" }`

Result: `story_resources_deleted`

## 5. Story API

### 5.1 `story.generate_plan`

Params:

```json
{
  "resource_id": "resource-0",
  "planner_api_id": "planner-fast"
}
```

`planner_api_id` is optional.

Result: `story_planned`

- `resource_id`
- `story_script`

### 5.2 `story.generate`

Params:

```json
{
  "resource_id": "resource-0",
  "display_name": "Flood Market",
  "architect_api_id": "architect-main"
}
```

Result: `story_generated`

- `resource_id`
- `story_id`
- `display_name`
- `graph`
- `world_state_schema`
- `player_state_schema`
- `introduction`

### 5.3 `story.get`

Params: `{ "story_id": "story-0" }`

Result: `story`

- `story_id`
- `display_name`
- `resource_id`
- `graph`
- `world_state_schema`
- `player_state_schema`
- `introduction`

### 5.4 `story.list`

Params: `{}`

Result: `stories_listed`

Each `StorySummaryPayload` contains:

- `story_id`
- `display_name`
- `resource_id`
- `introduction`

### 5.5 `story.delete`

Params: `{ "story_id": "story-0" }`

Result: `story_deleted`

### 5.6 `story.start_session`

Params:

```json
{
  "story_id": "story-0",
  "display_name": "Negotiation Run",
  "player_description": "A careful trader with little money.",
  "config_mode": "use_global",
  "session_api_ids": null
}
```

Field notes:

- `display_name`: optional session display name.
- `player_description`: required.
- `config_mode`: `use_global` or `use_session`, default is `use_global`.
- `session_api_ids`: only meaningful when `config_mode` is `use_session`.

Result: `session_started`

- `story_id`
- `display_name`
- `snapshot`
- `character_summaries`
- `config`

## 6. Session API

### 6.1 `session.get`

Requires top-level `session_id`. Params are an empty object.

Result: `session`

- `session_id`
- `story_id`
- `display_name`
- `snapshot`
- `config`

### 6.2 `session.list`

Params: `{}`

Result: `sessions_listed`

Each item contains:

- `session_id`
- `story_id`
- `display_name`
- `turn_index`

### 6.3 `session.delete`

Requires top-level `session_id`. Params are an empty object.

Result: `session_deleted`

### 6.4 `session.run_turn`

Requires top-level `session_id`.

Request params:

```json
{
  "player_input": "I ask the merchant about the flood gate.",
  "api_overrides": {
    "director_api_id": "director-main"
  }
}
```

Unary ack result:

```json
{
  "type": "turn_stream_accepted"
}
```

The server then starts sending stream events. The final completed response is:

```json
{
  "type": "turn_completed",
  "result": {}
}
```

### 6.5 `session.update_player_description`

Requires top-level `session_id`.

Params:

```json
{
  "player_description": "Now the player is suspicious and impatient."
}
```

Result: `player_description_updated`

- `snapshot`

### 6.6 `session.get_runtime_snapshot`

Requires top-level `session_id`. Params are an empty object.

Result: `runtime_snapshot`

- `snapshot`

### 6.7 `session.get_config`

Requires top-level `session_id`. Params are an empty object.

Result: `session_config`

- `mode`
- `session_api_ids`
- `effective_api_ids`

### 6.8 `session.update_config`

Requires top-level `session_id`.

Params:

```json
{
  "mode": "use_session",
  "session_api_ids": {
    "planner_api_id": "default",
    "architect_api_id": "default",
    "director_api_id": "default",
    "actor_api_id": "default",
    "narrator_api_id": "default",
    "keeper_api_id": "default"
  },
  "api_overrides": {
    "actor_api_id": "actor-large"
  }
}
```

Result: `session_config`

## 7. Config API

### 7.1 `config.get_global`

Params: `{}`

Result: `global_config`

- `api_ids`

### 7.2 `config.update_global`

Params:

```json
{
  "api_overrides": {
    "architect_api_id": "architect-large"
  }
}
```

Result: `global_config`

## 8. Stream Events

`session.run_turn` currently emits these `event.body.type` values:

- `turn_started`
- `player_input_recorded`
- `keeper_applied`
- `director_completed`
- `narrator_started`
- `narrator_text_delta`
- `narrator_completed`
- `actor_started`
- `actor_thought_delta`
- `actor_action_complete`
- `actor_dialogue_delta`
- `actor_completed`

End-of-stream variants:

- `completed`, where `response.type = "turn_completed"`
- `failed`, where `error` is the standard error payload

## 9. Character Archive Reference

Character files uploaded through `upload.*` must be `.chr` ZIP archives containing:

- `manifest.json`
- `content.json`
- `cover.<ext>`

Current `content.json` fields:

- `id`
- `name`
- `personality`
- `style`
- `tendencies`
- `state_schema`
- `system_prompt`
