# Runtime Flow

This document describes the current flow from resource preparation to interactive play.

## 1. Prepare Base Resources

Start by creating reusable configuration resources:

- `api`
- `api_group`
- `preset`

Common methods:

- `api.create` / `api.list` / `api.update` / `api.delete`
- `api_group.create` / `api_group.list` / `api_group.update` / `api_group.delete`
- `preset.create` / `preset.list` / `preset.update` / `preset.delete`

If `api_group_id` or `preset_id` is omitted and at least one resource exists, the backend sorts ids and uses the first available value.

## 2. Create Schemas

Then create reusable `schema` resources for:

- character-private state
- player state
- world state seeds

Each schema contains:

- `schema_id`
- `display_name`
- `tags`
- `fields`

## 3. Prepare Player Profiles

Player setup is now the standalone `player_profile` resource.

Fields:

- `player_profile_id`
- `display_name`
- `description`

One session activates at most one profile at a time.

## 4. Import or Create Characters

### 4.1 Import `.chr`

```text
POST /upload/character:{character_id}/archive
```

- the request body is raw `.chr` bytes
- no JSON-RPC wrapper
- the server parses the archive, stores the cover, and creates a `character`

### 4.2 Create Directly

1. `character.create`
2. optional `POST /upload/character:{character_id}/cover`

Character content currently stores:

- `id`
- `name`
- `personality`
- `style`
- `schema_id`
- `system_prompt`

## 5. Create Story Resources

Once characters and schemas exist, create `story_resources` with:

- `story_concept`
- `character_ids`
- `player_schema_id_seed`
- `world_schema_id_seed`
- `lorebook_ids`
- `planned_story`

## 6. Optional: Run Planner First

If you want an editable draft first:

- `story.generate_plan`

Then store the edited plan back through:

- `story_resources.update`

## 7. Generate the Story

Recommended flow:

- `story_draft.start`
- `story_draft.continue`
- `story_draft.finalize`

Compatibility wrapper:

- `story.generate`

The draft flow:

1. reads `story_resources`
2. generates `planned_story` if needed
3. has `Architect` generate the first partial graph section plus schemas and introduction
4. stores the partial graph and optional `common_variables` in `story_draft`
5. appends one outline section per `story_draft.continue`
6. validates and creates the final `story` in `story_draft.finalize`

## 8. Start a Session

Use:

- `story.start_session`

Input may include:

- `story_id`
- `display_name`
- `player_profile_id`
- `api_group_id`
- `preset_id`

## 9. Run Turns

The main runtime method is:

- `session.run_turn`

It is initiated through `POST /rpc`, but the HTTP response is streamed as `text/event-stream`.
