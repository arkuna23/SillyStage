# API Reference

This document lists the currently implemented JSON-RPC methods.

## 1. upload

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `upload.init` | No | `upload_initialized` | Start chunked upload |
| `upload.chunk` | No | `upload_chunk_accepted` | Upload a chunk |
| `upload.complete` | No | `character_card_uploaded` | Finish `.chr` upload and persist the character |

## 2. api

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `api.create` | No | `api` | Create one reusable API definition |
| `api.get` | No | `api` | Get one API definition |
| `api.list` | No | `apis_listed` | List API definitions |
| `api.update` | No | `api` | Update one API definition |
| `api.delete` | No | `api_deleted` | Delete one API definition |

An `api` stores one reusable connection definition:

- `provider`
- `base_url`
- `api_key`
- `model`

Notes:

- Read APIs never return the raw `api_key`
- `api.delete` returns `conflict` if the API is still referenced by an `api_group`

## 3. api_group

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `api_group.create` | No | `api_group` | Create an API group |
| `api_group.get` | No | `api_group` | Get one API group |
| `api_group.list` | No | `api_groups_listed` | List API groups |
| `api_group.update` | No | `api_group` | Update an API group |
| `api_group.delete` | No | `api_group_deleted` | Delete an API group |

An `api_group` stores per-agent `api_id` bindings:

- `planner_api_id`
- `architect_api_id`
- `director_api_id`
- `actor_api_id`
- `narrator_api_id`
- `keeper_api_id`
- `replyer_api_id`

Notes:

- `api_group.delete` returns `conflict` if the group is still referenced by a story draft or session

## 4. preset

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `preset.create` | No | `preset` | Create a preset |
| `preset.get` | No | `preset` | Get one preset |
| `preset.list` | No | `presets_listed` | List presets |
| `preset.update` | No | `preset` | Update a preset |
| `preset.delete` | No | `preset_deleted` | Delete a preset |

A `preset` stores per-agent generation parameters.

The currently implemented fields are:

- `temperature`
- `max_tokens`

Notes:

- `preset.delete` returns `conflict` if the preset is still referenced by a story draft or session
- The preset object is intended to grow over time; more per-agent generation fields can be added later without changing the binding model

## 5. schema

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

## 6. player_profile

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

## 7. character

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
| `story.update_graph` | No | `story` | Replace the full story graph, including node `on_enter_updates` |
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
- optional `api_group_id`
- optional `preset_id`

If `api_group_id` or `preset_id` is omitted and at least one resource exists, the backend sorts
the available ids and uses the first one.

`story.update` input:

- `story_id`
- `display_name`

`story.update_graph` input:

- `story_id`
- `graph`

## 9. story_draft

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `story_draft.start` | No | `story_draft` | Start chunked Architect generation and create the first partial graph |
| `story_draft.get` | No | `story_draft` | Get draft details including the current partial graph |
| `story_draft.list` | No | `story_drafts_listed` | List draft summaries |
| `story_draft.update_graph` | No | `story_draft` | Replace the current partial graph, including node `on_enter_updates` |
| `story_draft.continue` | No | `story_draft` | Generate the next outline section and merge it into the draft |
| `story_draft.finalize` | No | `story_generated` | Validate the completed draft and create the final story |
| `story_draft.delete` | No | `story_draft_deleted` | Delete a draft |

Notes:

- Draft generation is section-based, not fixed-node-count based.
- The server keeps the partial graph in a `story_draft` object. Clients do not need to send generated nodes back.
- `story_draft.update_graph` replaces `partial_graph` for a non-finalized draft, including node `on_enter_updates`.
- `story.generate` remains available for clients that still want a one-shot call, but new clients should prefer `story_draft.*`.
- `story.generate_plan`, `story.generate`, and `story_draft.start` all accept optional
  `api_group_id` and `preset_id`; if omitted, the backend auto-selects the first available pair.

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
| `session.get_variables` | Yes | `session_variables` | No |
| `session.update_variables` | Yes | `session_variables` | No |
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
- `session.get_variables` returns the current mutable conversation variables:
  - `custom`
  - `player_state`
  - `character_state`
- `session.update_variables` applies a `StateUpdate` to those same variable maps
- `session.update_variables` rejects non-variable ops such as:
  - `SetCurrentNode`
  - `SetActiveCharacters`
  - `AddActiveCharacter`
  - `RemoveActiveCharacter`
- `session.get_config` returns the session binding:
  - `api_group_id`
  - `preset_id`
- `session.update_config` updates that binding. Omitted fields keep the current value.

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
| `config.get_global` | No | `global_config` | Get the current fallback `api_group_id` / `preset_id` pair |

Notes:

- `config.get_global` succeeds even when nothing has been initialized yet
- in that case both `api_group_id` and `preset_id` are `null`
- otherwise it returns the current fallback pair, which is the first available `api_group` and first available `preset` after sorting by id

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

- `dashboard.global_config.api_group_id` and `dashboard.global_config.preset_id` may both be `null` when the backend is still unconfigured
- In that state, browse/configuration APIs still work, but agent-running APIs return an “LLM config is not initialized” error
