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

Notes:

- Read APIs never return the raw `api_key`
- Delete returns `conflict` if the API is still referenced by global or session config

## 3. schema

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

## 4. player_profile

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

## 5. character

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

## 6. story_resources

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

## 7. story

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `story.generate_plan` | No | `story_planned` | Run Planner and get editable script text |
| `story.generate` | No | `story_generated` | Generate graph and schema references |
| `story.get` | No | `story` | Get story details |
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

## 8. session

| Method | session_id | Result | Streaming |
| --- | --- | --- | --- |
| `session.get` | Yes | `session` | No |
| `session.list` | No | `sessions_listed` | No |
| `session.delete` | Yes | `session_deleted` | No |
| `session.run_turn` | Yes | `turn_stream_accepted` / `turn_completed` | Yes |
| `session.set_player_profile` | Yes | `session` | No |
| `session.update_player_description` | Yes | `player_description_updated` | No |
| `session.get_runtime_snapshot` | Yes | `runtime_snapshot` | No |
| `session.get_config` | Yes | `session_config` | No |
| `session.update_config` | Yes | `session_config` | No |

Notes:

- `session.set_player_profile` switches the active player profile only; it does not switch `player_state`
- `session.update_player_description` clears `player_profile_id` and uses the manual description instead

## 9. config

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `config.get_global` | No | `global_config` | Get global agent API selections |
| `config.update_global` | No | `global_config` | Update global agent API selections |

## 10. dashboard

| Method | session_id | Result | Notes |
| --- | --- | --- | --- |
| `dashboard.get` | No | `dashboard` | Get aggregated dashboard data |

`dashboard` contains:

- `health`
- `counts`
- `global_config`
- `recent_stories`
- `recent_sessions`

