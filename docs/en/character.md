# Character Card Structure and File Format

This document describes the current character content structure, the `.chr` archive format, and how character cards relate to standalone schema resources.

## 1. Character Content

The current character content structure is:

```json
{
  "id": "merchant",
  "name": "Old Merchant",
  "personality": "greedy but friendly trader",
  "style": "talkative, casual, slightly cunning",
  "schema_id": "schema-character-merchant",
  "system_prompt": "You are {{char}}. Speak naturally to {{user}} and avoid breaking immersion."
}
```

Field meanings:

- `id`: character id
- `name`: display name
- `personality`: personality summary
- `style`: speaking / behavior style
- `schema_id`: id of the character-private state schema
- `system_prompt`: actor-facing character system prompt

Template variables:

- `{{char}}`: replaced at runtime with the character display name
- `{{user}}`: replaced at runtime with the current player name; falls back to `User` if no player name is set
- `{{field_name}}`: replaced at runtime with the current character's own state value for that schema field

Replacement applies to:

- `personality`
- `style`
- `system_prompt`

Schema variable rules:

- The backend reads values from `world_state.character_state[character_id][field_name]`
- If no runtime value exists, the backend falls back to the schema field `default`
- If neither a runtime value nor a schema default exists, the placeholder is left unchanged
- Strings render as plain text
- Numbers and booleans render as compact plain text
- Arrays, objects, and `null` render as compact JSON text
- `char` and `user` are reserved names and do not come from schema fields

Important:

- Character cards no longer embed `state_schema`
- Private state schema is referenced through standalone `schema` resources

## 2. `.chr` Archive Format

`.chr` is a ZIP archive with fixed entries:

- `manifest.json`
- `content.json`
- `cover.<ext>`

Example layout:

```text
merchant.chr
├── manifest.json
├── content.json
└── cover.png
```

## 3. `manifest.json`

Example:

```json
{
  "format": "sillystage_character_card",
  "version": 1,
  "character_id": "merchant",
  "content_path": "content.json",
  "cover_path": "cover.png",
  "cover_mime_type": "image/png"
}
```

Constraints:

- `format` must be `sillystage_character_card`
- `version` is currently fixed to `1`
- `content_path` is currently fixed to `content.json`
- `cover_path` must start with `cover.`
- `character_id` must match `content.json.id`

Supported cover MIME types:

- `image/png`
- `image/jpeg`
- `image/webp`

## 4. `content.json`

`content.json` uses the character content structure shown above, i.e. `CharacterCardContent`.

Key point:

- `schema_id` is required
- it references a standalone `schema` resource managed by the backend

## 5. `cover`

The cover is stored as a separate binary entry inside the ZIP archive.

Common file names:

- `cover.png`
- `cover.jpg`
- `cover.webp`

Requirements:

- cover bytes must not be empty
- MIME type and file extension should match

## 6. Two Creation Paths

Character cards can currently be created in two ways.

### 6.1 Upload `.chr`

Through:

1. `upload.init`
2. `upload.chunk`
3. `upload.complete`

The backend parses the archive and creates the character object.

### 6.2 Create from Request Data

Through:

1. `character.create`
2. optional `character.set_cover`

This is the preferred flow for a frontend form editor.

## 7. Read and Export

### 7.1 Get Character Content

- `character.list`: get summaries
- `character.get`: get the full character content

### 7.2 Get Cover

- `character.get_cover`

Returns:

- `character_id`
- `cover_file_name`
- `cover_mime_type`
- `cover_base64`

### 7.3 Export `.chr`

- `character.export_chr`

Returns:

- `character_id`
- `file_name`
- `content_type`
- `chr_base64`

The backend repacks the `.chr` archive from the currently stored character content and cover. It does not need to preserve byte-for-byte identity with the originally uploaded archive.
