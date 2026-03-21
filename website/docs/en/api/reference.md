# API Reference

This page summarizes the currently implemented JSON-RPC methods and transport-level HTTP endpoints.

## 1. binary HTTP

| Route | Body | Success response | Notes |
| --- | --- | --- | --- |
| `POST /upload/{resource_id}/{file_id}` | Raw bytes | `ResourceFilePayload` | Upload one logical resource file |
| `GET /download/{resource_id}/{file_id}` | None | Raw bytes | Download one logical resource file |

## 2. Configuration and Resource Families

| Family | Main methods | Notes |
| --- | --- | --- |
| `api.*` | `create` `get` `list` `list_models` `update` `delete` | Reusable API connections |
| `api_group.*` | `create` `get` `list` `update` `delete` | Per-agent API binding bundles |
| `preset.*` | `create` `get` `list` `update` `delete` | Generation settings and modular prompts |
| `preset_entry.*` | `create` `update` `delete` | Single prompt entry management |
| `preset_preview.*` | `template` `runtime` | Prompt preview endpoints |
| `schema.*` | `create` `get` `list` `update` `delete` | State schema resources |
| `lorebook.*` | `create` `get` `list` `update` `delete` | Lorebook resources |
| `lorebook_entry.*` | `create` `get` `list` `update` `delete` | Lorebook entry resources |
| `player_profile.*` | `create` `get` `list` `update` `delete` | Player setup resources |
| `character.*` | `create` `get` `list` `update` `delete` | Character card resources |

Notes:

- `api.delete` returns `conflict` if an `api_group` still references the API
- `preset.delete` returns `conflict` if a story draft or session still references the preset
- `schema.delete` returns `conflict` if characters, resources, stories, or sessions still reference it
- character covers and `.chr` archives are transferred through binary routes

## 3. Story Families

| Family | Main methods | Notes |
| --- | --- | --- |
| `story_resources.*` | `create` `get` `list` `update` `delete` | Pre-generation input bundle |
| `story.*` | `generate_plan` `create` `generate` `get` `update` `update_graph` `list` `delete` `start_session` | Story lifecycle |
| `story_draft.*` | `start` `get` `list` `update_graph` `continue` `finalize` `delete` | Chunked story generation |

Highlights:

- `story.generate` is the compatibility wrapper around the full draft flow
- `story_draft.start`, `story.generate`, and `story.generate_plan` all accept optional `api_group_id` and `preset_id`
- `story.update_graph` and `story_draft.update_graph` validate graph structure before saving

## 4. Session Families

| Family | Main methods | Notes |
| --- | --- | --- |
| `session.*` | `get` `update` `list` `delete` `run_turn` `suggest_replies` `set_player_profile` `update_player_description` `get_runtime_snapshot` `get_variables` `update_variables` `get_config` `update_config` | Core runtime APIs |
| `session_character.*` | `get` `list` `update` `delete` `enter_scene` `leave_scene` | Session-scoped temporary characters |
| `session_message.*` | `create` `get` `list` `update` `delete` | Transcript message CRUD |

Notes:

- `session.run_turn` is the primary streaming method
- `session.suggest_replies` returns 3 suggestions by default and accepts `2..=5`
- `session.update_variables` only accepts variable updates
- session-scoped temporary characters do not modify the underlying story graph

## 5. Global and Aggregate APIs

| Method | Result | Notes |
| --- | --- | --- |
| `config.get_global` | `global_config` | Get the fallback `api_group_id` / `preset_id` pair |
| `dashboard.get` | `dashboard` | Get aggregated dashboard data |

`dashboard` contains:

- `health`
- `counts`
- `global_config`
- `recent_stories`
- `recent_sessions`

## 6. Data Package APIs

| Method | Result | Notes |
| --- | --- | --- |
| `data_package.export_prepare` | `data_package_export_prepared` | Build a temporary ZIP export slot |
| `data_package.import_prepare` | `data_package_import_prepared` | Allocate a temporary ZIP import slot |
| `data_package.import_commit` | `data_package_import_committed` | Validate and atomically import the ZIP |

Supported resource types:

- `preset`
- `schema`
- `lorebook`
- `player_profile`
- `character`
- `story_resources`
- `story`

Import/export flow:

1. call `data_package.export_prepare`, then download through `GET /download/package_export:{export_id}/archive`
2. call `data_package.import_prepare`, then upload through `POST /upload/package_import:{import_id}/archive`
3. call `data_package.import_commit` to validate and apply the package atomically
