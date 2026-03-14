# API Reference

This document lists the currently implemented JSON-RPC methods.

## 1. upload

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `upload.init` | No | `upload_initialized` | Start chunked upload |
| `upload.chunk` | No | `upload_chunk_accepted` | Upload a chunk |
| `upload.complete` | No | `character_card_uploaded` | Finish `.chr` upload and persist the character |

## 2. llm_api

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `llm_api.create` | No | `llm_api` | Create an LLM API definition |
| `llm_api.get` | No | `llm_api` | Get one LLM API definition |
| `llm_api.list` | No | `llm_apis_listed` | List LLM API definitions |
| `llm_api.update` | No | `llm_api` | Update an LLM API definition |
| `llm_api.delete` | No | `llm_api_deleted` | Delete an LLM API definition |

The currently supported generation defaults on `llm_api` are:

- `temperature`
- `max_tokens`

Notes:

- Read APIs never return the raw `api_key`
- Delete returns `conflict` if the API is still referenced by global or session config
- `llm_api.create` may omit connection or model fields; missing values are filled from the current effective `default_llm_config`
- If no global config exists yet, creating the first `llm_api` automatically binds that `api_id` to every agent role

## 3. default_llm_config

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `default_llm_config.get` | No | `default_llm_config` | Get saved and effective default config |
| `default_llm_config.update` | No | `default_llm_config` | Replace the saved default config |

Notes:

- `saved` is the persistent default config stored in the backend
- `effective` is the runtime default config after applying env/file overrides
- env/file overrides do not overwrite the saved record

## 4. schema

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `schema.create` | No | `schema` | Create a schema resource |
| `schema.get` | No | `schema` | Get one schema |
| `schema.list` | No | `schemas_listed` | List schemas |
| `schema.update` | No | `schema` | Update a schema |
| `schema.delete` | No | `schema_deleted` | Delete a schema |

Notes:

- A schema has no fixed kind; classification is expressed through `tags`
- Delete returns `conflict` if the schema is still referenced by characters, resources, stories, or sessions

## 5. player_profile

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `player_profile.create` | No | `player_profile` | Create a player profile |
| `player_profile.get` | No | `player_profile` | Get one player profile |
| `player_profile.list` | No | `player_profiles_listed` | List player profiles |
| `player_profile.update` | No | `player_profile` | Update a player profile |
| `player_profile.delete` | No | `player_profile_deleted` | Delete a player profile |

Notes:

- A player profile contains `player_profile_id`, `display_name`, and `description`
- Delete returns `conflict` if any session still references it

## 6. character

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `character.create` | No | `character_created` | Create a character directly from request data |
| `character.get` | No | `character` | Get full character content |
| `character.update` | No | `character` | Update full character content |
| `character.list` | No | `characters_listed` | List character summaries |
| `character.delete` | No | `character_deleted` | Delete a character |
| `character.set_cover` | No | `character_cover_updated` | Set or replace the cover |
| `character.get_cover` | No | `character_cover` | Get cover as base64 |
| `character.export_chr` | No | `character_chr_export` | Export `.chr` |

Character content fields:

- `id`
- `name`
- `personality`
- `style`
- `tendencies`
- `schema_id`
- `system_prompt`

Notes:

- `schema_id` references the character-private state schema
- Cover upload is a separate update step
- `.chr` export requires the character to have a cover

## 7. story_resources

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `story_resources.create` | No | `story_resources` | Create story resources |
| `story_resources.get` | No | `story_resources` | Get one resource bundle |
| `story_resources.list` | No | `story_resources_listed` | List resource bundles |
| `story_resources.update` | No | `story_resources` | Update a resource bundle |
| `story_resources.delete` | No | `story_resources_deleted` | Delete a resource bundle |

Fields:

- `story_concept`
- `character_ids`
- `player_schema_id_seed`
- `world_schema_id_seed`
- `planned_story`

## 8. story

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `story.generate_plan` | No | `story_planned` | Run Planner and get editable script text |
| `story.generate` | No | `story_generated` | Compatibility wrapper: `story_draft.start -> continue* -> finalize` |
| `story.get` | No | `story` | Get story details |
| `story.update` | No | `story` | Update story metadata |
| `story.list` | No | `stories_listed` | List stories |
| `story.delete` | No | `story_deleted` | Delete a story |
| `story.start_session` | No | `session_started` | Create a new session from a story |

Important `story_generated` fields:

- `story_id`
- `resource_id`
- `graph`
- `world_schema_id`
- `player_schema_id`
- `introduction`

`story.start_session` input:

- `story_id`
- optional `display_name`
- optional `player_profile_id`
- `config_mode`
- optional `session_api_ids`

`story.update` input:

- `story_id`
- `display_name`

## 9. story_draft

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `story_draft.start` | No | `story_draft` | Start chunked Architect generation and create the first partial graph |
| `story_draft.get` | No | `story_draft` | Get draft details including the current partial graph |
| `story_draft.list` | No | `story_drafts_listed` | List draft summaries |
| `story_draft.continue` | No | `story_draft` | Generate the next outline section and merge it into the draft |
| `story_draft.finalize` | No | `story_generated` | Validate the completed draft and create the final story |
| `story_draft.delete` | No | `story_draft_deleted` | Delete a draft |

Notes:

- Draft generation is section-based, not fixed-node-count based.
- The server keeps the partial graph in a `story_draft` object. Clients do not need to send generated nodes back.
- `story.generate` remains available for clients that still want a one-shot call, but new clients should prefer `story_draft.*`.

## 10. session

| Method | session_id | Result | Streaming |
| --- | --- | --- | --- |
| `session.get` | Yes | `session` | No |
| `session.update` | Yes | `session` | No |
| `session.list` | No | `sessions_listed` | No |
| `session.delete` | Yes | `session_deleted` | No |
| `session.run_turn` | Yes | `turn_stream_accepted` / `turn_completed` | Yes |
| `session.suggest_replies` | Yes | `suggested_replies` | No |
| `session.set_player_profile` | Yes | `session` | No |
| `session.update_player_description` | Yes | `player_description_updated` | No |
| `session.get_runtime_snapshot` | Yes | `runtime_snapshot` | No |
| `session.get_config` | Yes | `session_config` | No |
| `session.update_config` | Yes | `session_config` | No |

Notes:

- `session.update` only updates the session `display_name`
- `session.suggest_replies` generates player reply suggestions on demand and does not write to `history`
- `session.suggest_replies` returns 3 suggestions by default and accepts `limit` values in `2..=5`
- `story.start_session` and `session.get` now return session details with:
  - `created_at_ms`
  - `updated_at_ms`
  - `history`
- `session.list` summaries now include:
  - `created_at_ms`
  - `updated_at_ms`
- `history` stores the visible session transcript in chronological order. It currently includes:
  - `player_input`
  - `narration`
  - `dialogue`
  - `action`
- `history` is now backed by standalone `session_message` records; `session.get` returns the aggregated ordered list
- `session.set_player_profile` switches the active player profile only; it does not switch `player_state`
- `session.update_player_description` clears `player_profile_id` and uses the manual description instead

## 11. session_message

| Method | session_id | Result | Streaming |
| --- | --- | --- | --- |
| `session_message.create` | Yes | `session_message` | No |
| `session_message.get` | Yes | `session_message` | No |
| `session_message.list` | Yes | `session_messages_listed` | No |
| `session_message.update` | Yes | `session_message` | No |
| `session_message.delete` | Yes | `session_message_deleted` | No |

Notes:

- message CRUD only changes transcript data
- editing or deleting a message does not replay or mutate the session snapshot
- manual `create` appends to the end of the current session transcript
- `session.get.history` and `session_message.list` use the same ordered message shape

## 12. config

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `config.get_global` | No | `global_config` | Get global agent API selections |
| `config.update_global` | No | `global_config` | Update global agent API selections |

The current agent selection fields in `global_config`, `session_config`, and request-level `api_overrides` are:

- `planner_api_id`
- `architect_api_id`
- `director_api_id`
- `actor_api_id`
- `narrator_api_id`
- `keeper_api_id`
- `replyer_api_id`

Notes:

- `config.get_global` succeeds even when nothing has been initialized yet; in that case `api_ids = null`
- This means the service is up, but no default executable agent bindings have been configured yet

## 13. dashboard

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `dashboard.get` | No | `dashboard` | Get aggregated dashboard data |

`dashboard` contains:

- `health`
- `counts`
- `global_config`
- `recent_stories`
- `recent_sessions`

Notes:

- `dashboard.global_config.api_ids` may also be `null` when the backend is still unconfigured
- In that state, browse/configuration APIs still work, but agent-running APIs return an “LLM config is not initialized” error
