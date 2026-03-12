# SillyStage End-to-End Process

This document describes the full flow from importing resources, generating a story, starting a session, and letting the player participate in the conversation, to saving and switching runtime state. It does not repeat field-level request details. For exact payload shapes, use `docs/en/api/spec.md` and `docs/en/api/reference.md`.

## 1. Overview

The system is currently split into these layers:

- `ss-protocol`: defines request, response, and streaming event structures.
- `ss-handler`: receives protocol requests and performs application operations.
- `ss-engine`: handles story generation, session execution, and agent orchestration.
- `ss-store`: persists character cards, resources, stories, sessions, and config.
- `ss-server`: provides HTTP/SSE transport.
- `ss-app`: boots the full backend application.

From the outside, the main entrypoints are:

- `POST /rpc`
- `GET /healthz`

Regular requests return JSON-RPC responses. `session.run_turn` returns an SSE stream.

## 2. Main Flow from Resources to Story

### 2.1 Import character cards

Character cards use the `.chr` extension. Each `.chr` file is a packaged archive containing:

- `manifest.json`
- `content.json`
- `cover.<ext>`

Uploads use a chunked flow:

1. Call `upload.init`
2. Call `upload.chunk` multiple times
3. Call `upload.complete`

After upload completes, the backend parses the `.chr` archive, writes the character content and cover into `ss-store`, and returns a `character_id`. All later resources and stories only reference the `character_id`. They do not upload the full character card again.

To inspect or manage character cards, use:

- `character.get`
- `character.list`
- `character.delete`

### 2.2 Create story resources

Once character IDs are available, the client calls `story_resources.create` to create a set of story generation resources. These resources typically include:

- `story_concept`
- `character_ids`
- `world_state_schema_seed`
- `player_state_schema_seed`
- optional `planned_story`

`story resources` are the raw inputs used before a runnable story exists. They are not a playable story yet. They are the inputs later consumed by `Planner` and `Architect`.

After creation, they can be managed with:

- `story_resources.get`
- `story_resources.list`
- `story_resources.update`
- `story_resources.delete`

### 2.3 Optional: generate an editable story draft first

If you want a text draft that is easier for humans to edit and easier for `Architect` to read, call:

- `story.generate_plan`

This triggers `Planner`. It reads the current `story resources` and returns a plain-text `planned_story`.

A typical workflow is:

1. Call `story.generate_plan`
2. Receive `story_script`
3. Edit it on the client side
4. Write the edited `planned_story` back with `story_resources.update`

This step is optional. If you do not need a planning draft, move directly to story generation.

### 2.4 Generate the story

Call:

- `story.generate`

This request reads the specified `resource_id` and sends it to `Architect`. `Architect` currently produces:

- `graph`
- `world_state_schema`
- `player_state_schema`
- `introduction`

When generation succeeds, the backend saves the result as a new `story` object in `ss-store` and returns:

- `story_id`
- a generated result preview

From this point on, the story is a runnable object that can be used to start sessions.

To inspect or manage stories, use:

- `story.get`
- `story.list`
- `story.delete`

Notes:

- Deleting resources that were already used to generate a story should return a conflict.
- Deleting a story that still has dependent sessions should also return a conflict.

## 3. From Story to Session

### 3.1 Start a new session

Call:

- `story.start_session`

The input includes at least:

- `story_id`
- `player_description`

`player_description` is provided at runtime and does not belong to `story resources`. It describes the player for this specific playthrough, such as background, style, or role.

When starting a session, the backend:

1. Loads the `story` from `ss-store`
2. Loads the story's character cards and schemas
3. Builds the initial `RuntimeState`
4. Creates a `session`
5. Writes the initial snapshot into `ss-store`

The response returns:

- `session_id`
- `snapshot`
- character summaries for the current story

The same story can have multiple sessions. Each session is an independent save state.

### 3.2 List, load, and delete sessions

Once sessions exist, they can be managed with:

- `session.get`
- `session.list`
- `session.delete`

There is no dedicated "switch session" RPC. Switching sessions simply means sending later requests with a different `session_id`.

## 4. Runtime Conversation Flow

### 4.1 Start one turn

Each time the player sends an input, the client calls:

- `session.run_turn`

This is a streaming request. At the HTTP layer, the response becomes `text/event-stream`. The first frame is `ack`, and the following frames are `message` events. Every frame carries protocol JSON.

### 4.2 What the engine does internally

The current turn order is fixed:

1. Write the player input into shared memory
2. Run `Keeper` with `AfterPlayerInput`
3. Run `Director` to plan the beats
4. Execute `Narrator` and `Actor` beats in order
5. Run `Keeper` again with `AfterTurnOutputs`
6. Write the new `RuntimeSnapshot` back into the store

In other words:

`player input -> Keeper -> Director -> Actors/Narrator -> Keeper`

The key responsibilities are:

- `Actor` performs character expression
- `Narrator` describes environment and results
- `Keeper` converts observed facts into state updates
- `Director` decides what should happen during the turn

### 4.3 What the client receives as stream events

`session.run_turn` currently emits events such as:

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

The stream ends in one of two ways:

- `completed`
- `failed`

The `completed` frame includes the final aggregated result, so the frontend does not need to reconstruct the full turn result from deltas on its own.

## 5. How Runtime State Is Saved

### 5.1 Which objects are persisted

The system currently stores these objects in `ss-store`:

- character cards
- story resources
- generated stories
- sessions
- global config

The core persisted content of a session is:

- `session_id`
- `story_id`
- `RuntimeSnapshot`
- session config mode and agent API configuration

### 5.2 How the system restores and continues a conversation

When a new `session.run_turn` request arrives, the backend does not keep a single long-lived `Engine` instance in memory. Instead it:

1. Loads the session record from `ss-store` using `session_id`
2. Reads `RuntimeSnapshot`
3. Rebuilds runtime state from the corresponding story, character cards, and schemas
4. Creates a temporary `Engine`
5. Executes the turn
6. Writes the new snapshot back into the store

This means:

- sessions can be restored after service restart
- multiple stories and sessions can coexist
- switching conversations is fundamentally just loading another `session_id`

## 6. How Configuration Affects Story Generation and Runtime

Each agent can currently be bound to its own LLM API configuration, for example:

- planner
- architect
- director
- actor
- narrator
- keeper

Configuration exists in three layers:

- global config
- session config
- request override

Priority is:

`request override > session config / global config`

Sessions can also run in one of two modes:

- `UseGlobal`
- `UseSession`

Meaning:

- `UseGlobal`: the session reads the latest global config on each run
- `UseSession`: the session uses its own independent config

Related requests include:

- `config.get_global`
- `config.update_global`
- `session.get_config`
- `session.update_config`

## 7. A Complete User Journey

Putting everything together, a typical end-to-end flow looks like this:

1. Upload multiple `.chr` character cards
2. Receive multiple `character_id` values
3. Create `story resources`
4. Optionally generate `planned_story`
5. Optionally update `story resources`
6. Call `story.generate`
7. Receive `story_id`
8. Call `story.start_session`
9. Receive `session_id`
10. Let the player repeatedly call `session.run_turn`
11. Call `session.update_player_description` if needed
12. Call `session.get_runtime_snapshot` if needed
13. Use `session.list` to recover and continue prior conversations

## 8. The Three Most Important Things for Frontend Integration

- All regular requests go through `POST /rpc`
- `session.run_turn` is streaming and returns SSE over HTTP
- Switching stories or saves means switching `story_id` or `session_id`, not booting a different backend instance

For exact request and response fields, continue with:

- `docs/en/api/spec.md`
- `docs/en/api/reference.md`
