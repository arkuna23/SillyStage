# API Reference

This document lists the currently implemented JSON-RPC methods and transport-level binary HTTP endpoints.

## 1. binary HTTP

| Route | Body | Success response | Notes |
| --- | --- | --- | --- |
| `POST /upload/{resource_id}/{file_id}` | Raw bytes | `ResourceFilePayload` JSON (`200 OK`) | Optional `x-file-name`; transport adapter for logical resource files |
| `GET /download/{resource_id}/{file_id}` | None | Raw bytes (`200 OK`) | Downloads one logical resource file; uses attachment disposition when a file name is known |

Current built-in resource files:

- `character:{character_id}/cover`
- `character:{character_id}/archive`
- `package_import:{import_id}/archive`
- `package_export:{export_id}/archive`

## 2. api

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `api.create` | No | `api` | Create one reusable API definition |
| `api.get` | No | `api` | Get one API definition |
| `api.list` | No | `apis_listed` | List API definitions |
| `api.list_models` | No | `api_models_listed` | Probe one provider endpoint and list models |
| `api.update` | No | `api` | Update one API definition |
| `api.delete` | No | `api_deleted` | Delete one API definition |

An `api` stores one reusable connection definition:

- `provider`
- `base_url`
- `api_key`
- `model`

Notes:

- Read APIs never return the raw `api_key`
- `api.list_models` takes `provider`, `base_url`, and `api_key`; it does not persist an `api`
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
| `preset_entry.create` | No | `preset_entry` | Add one custom prompt entry under one agent module |
| `preset_entry.update` | No | `preset_entry` | Update one prompt entry inside one module |
| `preset_entry.delete` | No | `preset_entry_deleted` | Delete one custom prompt entry inside one module |

A `preset` stores per-agent generation parameters and modular prompt configuration.

The currently implemented fields are:

- `temperature`
- `max_tokens`
- optional `extra`
- `modules`

Each `module` contains:

- `module_id`
- `entries`

The current `module_id` values are:

- `role`
- `task`
- `static_context`
- `dynamic_context`
- `output`

Each `entries` item contains:

- `entry_id`
- `display_name`
- `kind`
- `enabled`
- `order`
- `required`
- optional `text`
- optional `context_key`

The current `kind` values are:

- `built_in_text`
- `built_in_context_ref`
- `custom_text`

Behavior notes:

- `preset.create`, `preset.get`, and `preset.update` use the full module shape
- `preset.list` returns summaries; each agent reports `module_count`, `entry_count`, and module
  metadata without `text/context_key`
- Clients may submit only the modules or entries they want to override; the backend normalizes
  the result against built-in agent templates and fills in missing built-in items
- `built_in_text` and `built_in_context_ref` come from backend defaults; they cannot be created
  through `preset_entry.create` and cannot be removed through `preset_entry.delete`
- `preset_entry.create` creates `custom_text` entries under one `agent + module_id`
- `preset_entry.update` can change `display_name`, `text`, `enabled`, and `order` for
  `custom_text`; for built-in entries it only allows `enabled` and `order`
- Enabled entries are compiled by module into the final prompt: `role/task/output` become part of
  the system prompt, while `static_context/dynamic_context` become stable and dynamic user-prompt
  segments
- `context_key` is only used by `built_in_context_ref`; `custom_text` uses `text`

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
- Schema fields follow `StateFieldSchema`:
  - `value_type`
  - optional `default`
  - optional `description`
  - optional `enum_values` for scalar fields (`bool`, `int`, `float`, `string`)
- Delete returns `conflict` if the schema is still referenced by characters, resources, stories, or sessions

## 6. lorebook

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `lorebook.create` | No | `lorebook` | Create a lorebook |
| `lorebook.get` | No | `lorebook` | Get one lorebook |
| `lorebook.list` | No | `lorebooks_listed` | List lorebooks |
| `lorebook.update` | No | `lorebook` | Update lorebook base metadata |
| `lorebook.delete` | No | `lorebook_deleted` | Delete a lorebook |

A `lorebook` stores:

- `lorebook_id`
- `display_name`
- `entries`

Notes:

- `lorebook.create` can include initial `entries`
- `lorebook.update` currently updates base metadata only, such as `display_name`
- `lorebook.delete` returns `conflict` if the lorebook is still referenced by `story_resources`

## 7. lorebook_entry

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `lorebook_entry.create` | No | `lorebook_entry` | Create one lorebook entry |
| `lorebook_entry.get` | No | `lorebook_entry` | Get one lorebook entry |
| `lorebook_entry.list` | No | `lorebook_entries_listed` | List one lorebook's entries |
| `lorebook_entry.update` | No | `lorebook_entry` | Update one lorebook entry |
| `lorebook_entry.delete` | No | `lorebook_entry_deleted` | Delete one lorebook entry |

Entry fields:

- `entry_id`
- `title`
- `content`
- `keywords`
- `enabled`
- `always_include`

Notes:

- All `lorebook_entry.*` methods are scoped by `lorebook_id`
- `enabled` defaults to `true` on create

## 8. player_profile

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

## 9. character

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `character.create` | No | `character_created` | Create a character directly from request data |
| `character.get` | No | `character` | Get full character content |
| `character.update` | No | `character` | Update full character content |
| `character.list` | No | `characters_listed` | List character summaries |
| `character.delete` | No | `character_deleted` | Delete a character |

Character content fields:

- `id`
- `name`
- `personality`
- `style`
- `schema_id`
- `system_prompt`
- `tags`
- `folder`

Notes:

- `schema_id` references the character-private state schema
- `tags` is a user-facing label list for the character card
- `folder` is the character card folder grouping; an empty string means unfiled
- character summaries and detail payloads also include:
  - `tags`
  - `folder`
  - `cover_file_name`
  - `cover_mime_type`
- cover bytes are fetched through `GET /download/character:{character_id}/cover`
- `.chr` import and export use:
  - `POST /upload/character:{character_id}/archive`
  - `GET /download/character:{character_id}/archive`
- cover upload uses `POST /upload/character:{character_id}/cover`

## 10. story_resources

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `story_resources.create` | No | `story_resources_created` | Create story resources |
| `story_resources.get` | No | `story_resources` | Get one resource bundle |
| `story_resources.list` | No | `story_resources_listed` | List resource bundles |
| `story_resources.update` | No | `story_resources_updated` | Update a resource bundle |
| `story_resources.delete` | No | `story_resources_deleted` | Delete a resource bundle |

Fields:

- `story_concept`
- `character_ids`
- `player_schema_id_seed`
- `world_schema_id_seed`
- `lorebook_ids`
- `planned_story`

Notes:

- `lorebook_ids` references lorebooks used during generation and may be empty

## 11. story

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
- `common_variables`

`story.generate` input:

- `resource_id`
- optional `display_name`
- optional `api_group_id`
- optional `preset_id`
- optional `common_variables`

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
- optional `display_name`
- optional `common_variables`

Each `common_variables` item contains:

- `scope`
- `key`
- `display_name`
- optional `character_id`
- optional `pinned` (defaults to `true`)

`story`, `stories_listed`, and `story_generated` responses all include `common_variables`.

`story.update_graph` input:

- `story_id`
- `graph`

## 12. story_draft

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
- `story_draft.start` accepts optional `common_variables`.
- `story_draft` detail responses include `common_variables`.
- `story_draft.finalize` copies the draft's `common_variables` into the final `story`.
- `story_draft.update_graph` replaces `partial_graph` for a non-finalized draft, including node `on_enter_updates`.
- `story.generate` remains available for clients that still want a one-shot call, but new clients should prefer `story_draft.*`.
- `story.generate_plan`, `story.generate`, and `story_draft.start` all accept optional
  `api_group_id` and `preset_id`; if omitted, the backend auto-selects the first available pair.

## 13. session

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
- `session.suggest_replies` currently uses the most recent 8 transcript messages as reply context
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
- `Director` may create session-scoped temporary characters during turn planning through `role_actions`
- those temporary characters are created and entered before beat execution, so they can act in the same turn
- session-scoped temporary characters do not modify the underlying story graph and do not persist outside the current session

## 14. session_character

| Method | session_id | Result | Streaming |
| --- | --- | --- | --- |
| `session_character.get` | Yes | `session_character` | No |
| `session_character.list` | Yes | `session_characters_listed` | No |
| `session_character.update` | Yes | `session_character` | No |
| `session_character.delete` | Yes | `session_character_deleted` | No |
| `session_character.enter_scene` | Yes | `session_character` | No |
| `session_character.leave_scene` | Yes | `session_character` | No |

Notes:

- session characters are temporary runtime-only roles scoped to a single session
- the primary creation path is `Director` `role_actions.create_and_enter` during `session.run_turn`
- `session_character.enter_scene` and `session_character.leave_scene` only change whether the character is in the current active cast
- session characters are not part of `story`, `story_draft`, or `character`

## 15. session_message

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

## 16. config

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `config.get_global` | No | `global_config` | Get the current fallback `api_group_id` / `preset_id` pair |

Notes:

- `config.get_global` succeeds even when nothing has been initialized yet
- in that case both `api_group_id` and `preset_id` are `null`
- otherwise it returns the current fallback pair, which is the first available `api_group` and first available `preset` after sorting by id

## 17. dashboard

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

## 18. data_package

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `data_package.export_prepare` | No | `data_package_export_prepared` | Build a temporary ZIP export slot and return a downloadable archive ref |
| `data_package.import_prepare` | No | `data_package_import_prepared` | Allocate a temporary ZIP import slot and return an upload archive ref |
| `data_package.import_commit` | No | `data_package_import_committed` | Validate and atomically import the uploaded ZIP archive |

Supported package resource types:

- `preset`
- `schema`
- `lorebook`
- `player_profile`
- `character` with optional cover bytes
- `story_resources`
- `story`

`data_package.export_prepare` input:

- optional `preset_ids`
- optional `schema_ids`
- optional `lorebook_ids`
- optional `player_profile_ids`
- optional `character_ids`
- optional `story_resource_ids`
- optional `story_ids`
- optional `include_dependencies`, defaults to `true`

Export behavior notes:

- at least one selected id is required
- when `include_dependencies = true`, stories automatically pull in their referenced `story_resources`, story/player/world schemas, character schemas, characters, and lorebooks
- exported characters include cover bytes when a cover exists
- download the prepared ZIP through `GET /download/package_export:{export_id}/archive`

Import behavior notes:

- call `data_package.import_prepare` first, then upload bytes through `POST /upload/package_import:{import_id}/archive`, then call `data_package.import_commit`
- import is all-or-nothing
- the current conflict policy is strict: any imported id that already exists returns `conflict`
- import does not overwrite or remap ids
