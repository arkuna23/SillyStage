# Character Card Structure and `.chr`

This page summarizes character content, archive layout, and import/export behavior.

## 1. Character Content

```json
{
  "id": "merchant",
  "name": "Old Merchant",
  "personality": "greedy but friendly trader",
  "style": "talkative, casual, slightly cunning",
  "schema_id": "schema-character-merchant",
  "system_prompt": "You are {{char}}. Speak naturally to {{user}} and avoid breaking immersion.",
  "tags": ["merchant", "shop"],
  "folder": "harbor/npcs"
}
```

Important fields:

- `schema_id`: references the character-private schema
- `tags`: user-facing labels
- `folder`: character grouping; an empty string means unfiled

## 2. Template Variables

Supported variables:

- `{{char}}`
- `{{user}}`
- `{{field_name}}`

Replacement applies to:

- `personality`
- `style`
- `system_prompt`

## 3. `.chr` Archive Layout

`.chr` is a ZIP archive with fixed entries:

- `manifest.json`
- `content.json`
- `cover.<ext>`

Example:

```text
merchant.chr
├── manifest.json
├── content.json
└── cover.png
```

## 4. `manifest.json`

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

## 5. Create and Import

### 5.1 Import `.chr`

- `POST /upload/character:{character_id}/archive`

Rules:

- the body is raw `.chr` bytes
- clients usually send `Content-Type: application/x-sillystage-character-card`
- archive `content.id` must match `{character_id}`

### 5.2 Create Directly

1. `character.create`
2. optional `POST /upload/character:{character_id}/cover`

Cover upload requires `image/png`, `image/jpeg`, or `image/webp`.

## 6. Read and Export

- `character.list`: summaries
- `character.get`: full character data
- `GET /download/character:{character_id}/cover`: raw cover bytes
- `GET /download/character:{character_id}/archive`: export `.chr`

JSON payloads expose cover metadata only:

- `cover_file_name`
- `cover_mime_type`
