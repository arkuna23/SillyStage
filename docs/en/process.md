# End-to-End Flow

This document describes the current flow from resource import to interactive play.

## 1. Prepare Base Resources

### 1.1 Configure APIs, API Groups, and Presets

Create one or more reusable `api` resources:

- `api.create`
- `api.list`
- `api.update`
- `api.delete`

An `api` stores one connection definition:

- `provider`
- `base_url`
- `api_key`
- `model`

Create one or more `api_group` resources:

- `api_group.create`
- `api_group.list`
- `api_group.update`
- `api_group.delete`

An `api_group` stores per-agent `api_id` bindings:

- `planner_api_id`
- `architect_api_id`
- `director_api_id`
- `actor_api_id`
- `narrator_api_id`
- `keeper_api_id`
- `replyer_api_id`

Then create one or more `preset` resources:

- `preset.create`
- `preset.list`
- `preset.update`
- `preset.delete`

A `preset` stores per-agent generation parameters. The current fields are:

- `temperature`
- `max_tokens`

The service is allowed to start with no `api_group` or `preset` resources. In that state,
browse/configuration APIs still work, but agent-executing APIs return an “LLM config is not initialized” error.

If a request omits `api_group_id` or `preset_id` and at least one resource exists, the backend
sorts ids and uses the first available `api_group` and `preset`.

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

`fields` can optionally constrain scalar values with `enum_values`, which helps both validation and
LLM-side state updates stay inside a known set of values.

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

### 3.1 Import `.chr`

Flow:

1. `POST /upload/character:{character_id}/archive`

The request body is raw `.chr` bytes, not JSON-RPC and not base64. After completion, the backend
parses the archive, stores the cover internally, and stores a `character` resource.

### 3.2 Create Character Directly

Flow:

1. `character.create`
2. optional `POST /upload/character:{character_id}/cover`

Character content now stores only:

- `id`
- `name`
- `personality`
- `style`
- `schema_id`
- `system_prompt`

That means a character references its private schema by `schema_id` instead of embedding the schema body.
Character payloads now expose `cover_file_name` and `cover_mime_type`; fetch the actual cover
bytes through `GET /download/character:{character_id}/cover`.

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

Recommended flow:

- `story_draft.start`
- `story_draft.continue`
- `story_draft.finalize`

Compatibility flow:

- `story.generate`

The recommended draft flow works like this:

1. `story_draft.start` reads `story_resources`
2. if `planned_story` is missing, the server first runs `story.generate_plan`
3. `Architect` generates only the first outline section plus the initial schemas and introduction
4. the server stores the partial graph, progress, and any caller-supplied `common_variables` in a `story_draft`
5. each `story_draft.continue` call generates one more section and merges it into the same draft
6. `story_draft.finalize` validates the merged graph and creates the final `story`, carrying over the draft's `common_variables`

`story.generate` still exists as a wrapper that internally runs the full draft flow for clients that want a one-shot call. It accepts the same optional `common_variables` input and writes it into the created story.

During generation, the server:

1. reads `story_resources`
2. keeps the already generated nodes in the server-side `story_draft`
3. sends Architect a compact graph summary plus the current outline section, instead of replaying the full graph every time
4. stores generated:
   - `graph`
   - world schema content
   - player schema content
   - `introduction`
5. stores the generated schema bodies as standalone `schema` resources
6. creates the final `story`

The resulting `story` stores:

- `story_id`
- `resource_id`
- `graph`
- `world_schema_id`
- `player_schema_id`
- `introduction`
- `common_variables`

## 7. Start a Session

Call:

- `story.start_session`

Input may include:

- `story_id`
- optional `display_name`
- optional `player_profile_id`
- optional `api_group_id`
- optional `preset_id`

This creates a new `session`.

The returned `session` detail includes `history`, but that transcript is now backed by standalone `session_message` records.

If the frontend wants a few clickable next-line suggestions for the player, it can call:

- `session.suggest_replies`

These suggestions are not written into the transcript; only real inputs sent to `session.run_turn` become history.

If the frontend wants to inspect or patch mutable conversation variables without fetching the
entire snapshot, it can call:

- `session.get_variables`
- `session.update_variables`

If the frontend also wants a stable list of variables to keep visible, it should read
`story.common_variables` and then map those definitions onto the live values returned by
`session.get_variables` or the runtime snapshot.

## 8. Session State

A session now stores:

- `player_profile_id`
- `player_schema_id`
- `api_group_id`
- `preset_id`
- `snapshot`

Where:

- `player_profile_id` selects the active player setup
- `player_schema_id` selects which player-state schema is in use
- `api_group_id` selects which per-agent `api_id` binding group is used
- `preset_id` selects which generation-parameter bundle is used
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
- `session.suggest_replies`

Execution order is fixed:

1. user input
2. `Keeper` (after player input)
3. `Director`
4. apply Director `role_actions`
5. `Narrator` / `Actor`
6. `Keeper` (after turn outputs)

The result is streamed as:

- unary `ack`
- `started`
- multiple `event` frames
- `completed` or `failed`

After turns have been recorded, transcript editing is done through:

- `session_message.create`
- `session_message.get`
- `session_message.list`
- `session_message.update`
- `session_message.delete`

These operations only change transcript data. They do not replay or mutate the session snapshot.

`session.suggest_replies` uses the most recent 8 transcript messages as its history window.

Director may also create temporary session-scoped characters during step 4. Those characters are
added to the current active cast before beat execution, so they can participate in the same turn.
They remain session-local runtime objects and do not modify the story graph.

## 10. Manage Session Characters

Session-scoped temporary characters can be inspected and managed through:

- `session_character.get`
- `session_character.list`
- `session_character.update`
- `session_character.delete`
- `session_character.enter_scene`
- `session_character.leave_scene`

The primary creation path is still `Director` `role_actions.create_and_enter` during
`session.run_turn`.

## 11. Switch Player Profile

To switch the current session to another player profile:

- `session.set_player_profile`

This updates:

- the active `player_profile_id`
- the effective description text

and keeps the existing `player_state`.

## 12. Manually Override Player Description

If the session should stop using a stored player profile and use ad hoc text instead:

- `session.update_player_description`

This:

- directly overwrites the current description text
- clears `player_profile_id`

## 13. Inspect and Patch Conversation Variables

Session variable APIs expose the mutable `world_state` maps without exposing scene-control fields:

- `custom`
- `player_state`
- `character_state`

`session.update_variables` accepts variable-only `StateUpdate` ops. It does not allow:

- `SetCurrentNode`
- `SetActiveCharacters`
- `AddActiveCharacter`
- `RemoveActiveCharacter`

## 14. Save, Restore, and Switch

The store persists:

- `api_group`
- `preset`
- `schema`
- `player_profile`
- `character`
- `story_resources`
- `story_draft`
- `story`
- `session`
- `session_character`
- `session_message`

That enables:

- browsing multiple stories with `story.list`
- browsing multiple conversations with `session.list`
- switching to another conversation by sending the target `session_id` on later requests
