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
  "tendencies": [
    "likes profitable deals",
    "avoids danger",
    "tries to maintain good relationships"
  ],
  "schema_id": "schema-character-merchant",
  "system_prompt": "You are a traveling merchant. Speak naturally as the character and avoid breaking immersion."
}
```

Field meanings:

- `id`: character id
- `name`: display name
- `personality`: personality summary
- `style`: speaking / behavior style
- `tendencies`: behavioral tendencies
- `schema_id`: id of the character-private state schema
- `system_prompt`: actor-facing character system prompt

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

