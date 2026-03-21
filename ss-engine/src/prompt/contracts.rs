pub(super) const DIRECTOR_OUTPUT_CONTRACT: &str = r#"Return one JSON object only. No markdown. No commentary.

ResponsePlan schema:
{
  "role_actions": [SessionCharacterAction],
  "beats": [ResponseBeat]
}

SessionCharacterAction schema:
- {"type":"create_and_enter","session_character_id":"snake_case_id","display_name":"text","personality":"text","style":"text","system_prompt":"text"}
- {"type":"leave_scene","session_character_id":"session_character_id"}

ResponseBeat schema:
- {"type":"Narrator","purpose":"DescribeTransition"|"DescribeScene"|"DescribeResult"}
- {"type":"Actor","speaker_id":"character_id","purpose":"AdvanceGoal"|"ReactToPlayer"|"CommentOnScene"}

Always include "role_actions" and "beats". Use [] when empty. Every Actor beat speaker_id must be either a CURRENT_CAST id or an id created in this same response via create_and_enter. Never invent any other speaker_id."#;

pub(super) const ACTOR_OUTPUT_CONTRACT: &str = r#"Return tagged performance segments only. Do not produce JSON, markdown, speaker labels, or text outside tags.

Allowed tags:
- <thought>...</thought>
- <action>...</action>
- <dialogue>...</dialogue>

Rules:
- Each segment body must be non-empty
- Do not nest tags
- You may emit any subset of the three tag types
- You may repeat tag types when needed
- Tags may appear in any order that fits the turn

Example:
<thought>I should test the courier first.</thought><action>Haru studies the sealed satchel.</action><dialogue>Tell me what you are carrying.</dialogue>"#;

pub(super) const KEEPER_OUTPUT_CONTRACT: &str = r#"Return one JSON object only. No markdown. No commentary.

StateUpdate schema:
{
  "ops": [StateOp]
}

Allowed StateOp shapes:
- {"type":"SetState","key":"state_key","value":{}} - set one world/global state value
- {"type":"RemoveState","key":"state_key"} - remove one world/global state key
- {"type":"SetPlayerState","key":"player_state_key","value":{}} - set one player state value
- {"type":"RemovePlayerState","key":"player_state_key"} - remove one player state key
- {"type":"SetActiveCharacters","characters":["character_id"]} - replace the whole active cast list
- {"type":"AddActiveCharacter","character":"character_id"} - add one character to the active cast
- {"type":"RemoveActiveCharacter","character":"character_id"} - remove one character from the active cast
- {"type":"SetCharacterState","character":"character_id","key":"character_state_key","value":{}} - set one state value for a specific character
- {"type":"RemoveCharacterState","character":"character_id","key":"character_state_key"} - remove one state key from a specific character

Always include "ops". Use [] when empty. Do not output SetCurrentNode. Do not output any other StateOp type. Never introduce a brand-new character id in active-character ops; temporary characters must be created by Director via create_and_enter."#;

pub(super) const REPLYER_OUTPUT_CONTRACT: &str = r#"Return one JSON object only. No markdown. No commentary.

Reply suggestion schema:
{
  "replies": [
    {
      "id": "reply_id",
      "text": "player reply"
    }
  ]
}

Always include "replies". Each item must include both "id" and "text"."#;

pub(super) const ARCHITECT_OUTPUT_CORE: &str = r#"Return one JSON object only. No markdown. No commentary. No duplicate keys.

Common nested schemas:

SchemaField:
{
  "value_type": "bool"|"int"|"float"|"string"|"array"|"object"|"null",
  "default": {},
  "description": "text",
  "enum_values": [{}]
}

NarrativeNode:
{
  "id": "node_id",
  "title": "short title",
  "scene": "short scene description",
  "goal": "short narrative goal",
  "characters": ["character_id"],
  "transitions": [Transition],
  "on_enter_updates": [StateOp]
}

NodeTransitionPatch:
{
  "node_id": "existing_node_id",
  "add_transitions": [Transition]
}

Transition:
{
  "to": "target_node",
  "condition": {
    "scope": "global"|"player"|"character",
    "character": "character_id when scope is character",
    "key": "state_key",
    "op": "eq"|"ne"|"gt"|"gte"|"lt"|"lte"|"contains",
    "value": {}
  }
}

Allowed StateOp shapes:
- {"type":"SetState","key":"state_key","value":{}} - set one world/global state value
- {"type":"RemoveState","key":"state_key"} - remove one world/global state key
- {"type":"SetPlayerState","key":"player_state_key","value":{}} - set one player state value
- {"type":"RemovePlayerState","key":"player_state_key"} - remove one player state key
- {"type":"SetCharacterState","character":"character_id","key":"character_state_key","value":{}} - set one state value for a specific character
- {"type":"RemoveCharacterState","character":"character_id","key":"character_state_key"} - remove one state key from a specific character

Every schema field object must include "value_type".
"enum_values" is only valid for scalar value_type values: "bool", "int", "float", "string".
If "enum_values" is present, omit "default" or make "default" exactly equal to one item from "enum_values".
Use only the exact StateOp type names listed above.
Every returned NarrativeNode id must be unique within the current response.
Prefer stable snake_case node ids.
Use only AVAILABLE_CHARACTERS ids in NarrativeNode.characters and all character-scoped references. Do not introduce temporary runtime-only character ids into graph data.
Use [] for empty arrays and {"fields":{}} for empty schemas."#;

pub(super) const ARCHITECT_GRAPH_OUTPUT_CONTRACT: &str = r#"Top-level output schema:
{
  "graph": {
    "start_node": "node_id",
    "nodes": [NarrativeNode]
  },
  "world_state_schema": {
    "fields": {
      "state_key": SchemaField
    }
  },
  "player_state_schema": {
    "fields": {
      "state_key": SchemaField
    }
  },
  "introduction": "short player-facing opening paragraph"
}

Always include "graph", "world_state_schema", "player_state_schema", and "introduction"."#;

pub(super) const ARCHITECT_DRAFT_INIT_OUTPUT_CONTRACT: &str = r#"Top-level output schema:
{
  "nodes": [NarrativeNode],
  "transition_patches": [NodeTransitionPatch],
  "section_summary": "one short sentence",
  "start_node": "node_id",
  "world_state_schema": {
    "fields": {
      "state_key": SchemaField
    }
  },
  "player_state_schema": {
    "fields": {
      "state_key": SchemaField
    }
  },
  "introduction": "short player-facing opening paragraph"
}

Always include every top-level key above. "start_node" must match one of the returned node ids. Returned node ids must all be new and unique within this response. In draft_init, every transition.to and every transition_patches target must exactly match one id from this same returned nodes list. If a target node is not present in returned nodes, do not output that transition yet. Do not point to future chunk nodes; add those links later with transition_patches."#;

pub(super) const ARCHITECT_DRAFT_CONTINUE_OUTPUT_CONTRACT: &str = r#"Top-level output schema:
{
  "nodes": [NarrativeNode],
  "transition_patches": [NodeTransitionPatch],
  "section_summary": "one short sentence"
}

Always include "nodes", "transition_patches", and "section_summary". Returned node ids must be unique within this response and must not reuse node ids that already exist in GRAPH_SUMMARY. If the next beat or scene already exists in GRAPH_SUMMARY, use that existing node id in transition targets or transition_patches instead of creating a duplicate node. Existing nodes may only be referenced in transition_patches or transition targets; never return an existing GRAPH_SUMMARY node as a new node. In draft_continue, every transition.to and every transition_patches target must exactly match either an existing GRAPH_SUMMARY node id or one of the node ids returned in this same response. If a target node is missing from both sets, do not output that transition yet. Do not point to future chunk nodes; add those links later with transition_patches."#;
