# End-to-End Flow

This document describes the current flow from resource import to interactive play.

## 1. Prepare Base Resources

### 1.1 Configure LLM APIs

Create one or more `llm_api` resources first:

- `llm_api.create`
- `llm_api.list`
- `llm_api.update`
- `llm_api.delete`

These objects describe reusable model endpoints such as OpenAI-compatible APIs.  
Global config and session config only reference `api_id`.

### 1.2 Create Schema Resources

Then create reusable `schema` resources:

- character-private state schemas
- player state schemas
- world state schema seeds

Through:

- `schema.create`
- `schema.list`
- `schema.get`

Each schema stores:

- `schema_id`
- `display_name`
- `tags`
- `fields`

## 2. Prepare Player Profiles

Player setup is now a standalone `player_profile` resource, and multiple profiles can coexist.

Fields:

- `player_profile_id`
- `display_name`
- `description`

APIs:

- `player_profile.create`
- `player_profile.list`
- `player_profile.get`
- `player_profile.update`
- `player_profile.delete`

A session activates at most one player profile at a time, but the system can store many profiles for later switching.

## 3. Import Character Cards

Characters can be created in two ways.

### 3.1 Upload `.chr`

Flow:

1. `upload.init`
2. `upload.chunk`
3. `upload.complete`

After completion, the backend parses the archive and stores a `character` resource.

### 3.2 Create Character Directly

Flow:

1. `character.create`
2. optional `character.set_cover`

Character content now stores only:

- `id`
- `name`
- `personality`
- `style`
- `tendencies`
- `schema_id`
- `system_prompt`

That means a character references its private schema by `schema_id` instead of embedding the schema body.

## 4. Create Story Resources

Once characters and schemas exist, create `story_resources`:

- `story_resources.create`

Main fields:

- `story_concept`
- `character_ids`
- `player_schema_id_seed`
- `world_schema_id_seed`
- `planned_story`

All of these are ids or plain text, not embedded resource objects.

## 5. Optional: Run Planner First

If you want an editable planning draft first:

- `story.generate_plan`

This reads the story concept and character set from `story_resources` and returns an editable script-like draft.  
If the user edits that draft, call:

- `story_resources.update`

to store the new `planned_story`.

## 6. Generate the Story

Call:

- `story.generate`

This stage:

1. reads `story_resources`
2. uses `Architect` to generate:
   - `graph`
   - world schema content
   - player schema content
   - `introduction`
3. stores the generated schema bodies as standalone `schema` resources
4. creates the final `story`

The resulting `story` stores:

- `story_id`
- `resource_id`
- `graph`
- `world_schema_id`
- `player_schema_id`
- `introduction`

## 7. Start a Session

Call:

- `story.start_session`

Input may include:

- `story_id`
- optional `display_name`
- optional `player_profile_id`
- `config_mode`
- optional `session_api_ids`

This creates a new `session`.

## 8. Session State

A session now stores:

- `player_profile_id`
- `player_schema_id`
- `snapshot`

Where:

- `player_profile_id` selects the active player setup
- `player_schema_id` selects which player-state schema is in use
- `snapshot` stores dynamic runtime state, including:
  - `world_state`
  - `turn_index`
  - the currently effective `player_description`

Important:

- a session has a single `player_state`
- switching `player_profile_id` does not switch `player_state`

## 9. Run Interactive Turns

Each player turn uses:

- `session.run_turn`

Execution order is fixed:

1. user input
2. `Keeper` (after player input)
3. `Director`
4. `Narrator` / `Actor`
5. `Keeper` (after turn outputs)

The result is streamed as:

- unary `ack`
- `started`
- multiple `event` frames
- `completed` or `failed`

## 10. Switch Player Profile

To switch the current session to another player profile:

- `session.set_player_profile`

This updates:

- the active `player_profile_id`
- the effective description text

and keeps the existing `player_state`.

## 11. Manually Override Player Description

If the session should stop using a stored player profile and use ad hoc text instead:

- `session.update_player_description`

This:

- directly overwrites the current description text
- clears `player_profile_id`

## 12. Save, Restore, and Switch

The store persists:

- `llm_api`
- `schema`
- `player_profile`
- `character`
- `story_resources`
- `story`
- `session`

That enables:

- browsing multiple stories with `story.list`
- browsing multiple conversations with `session.list`
- switching to another conversation by sending the target `session_id` on later requests

