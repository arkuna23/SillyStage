# Character Card Structure and `.chr` File Format

This document describes the current SillyStage character card data structure and the `.chr` archive format. It documents the current protocol implementation, not an open-ended general standard. If implementation and docs diverge, `ss-protocol/src/character.rs` is the source of truth.

## 1. Role of Character Cards

Character cards are one of the core inputs before story generation. There are currently two creation paths:

- upload a `.chr` file and let the server parse it
- call `character.create` for the content first, then `character.set_cover` for the cover

Both flows end up creating the same stored character object and returning a `character_id`. After that:

- `story resources` only reference `character_id`
- story generation reads the character card content
- session runtime loads the matching character cards again

So character cards are uploaded once and referenced later, instead of being inlined again for every resource or story request.

## 2. Character Content Structure

The main content of a character card is `content.json`, which currently maps to protocol type `CharacterCardContent`. Its fields are:

- `id`
  - Stable character ID
  - Must match `manifest.json.character_id`
- `name`
  - Display name of the character
- `personality`
  - Short description of the character's personality
- `style`
  - Speaking or acting style
- `tendencies`
  - List of character tendencies
  - This is an array of strings
- `state_schema`
  - Schema for the character's private state
  - Keys are state field names and values are `StateFieldSchema`
- `system_prompt`
  - System prompt used by the character agent

A minimal example:

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
  "state_schema": {
    "trust": {
      "value_type": "int",
      "description": "How much the merchant trusts the player"
    }
  },
  "system_prompt": "You are a traveling merchant. Speak naturally as the character and avoid breaking immersion."
}
```

## 3. `.chr` File Format

`.chr` is a ZIP archive with exactly three required entries:

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

Current protocol constraints:

- the manifest path must be `manifest.json`
- the content path must be `content.json`
- the cover path must start with `cover.`
- the cover file name is usually derived from the mime type:
  - `image/png` -> `cover.png`
  - `image/jpeg` -> `cover.jpg`
  - `image/webp` -> `cover.webp`

## 4. `manifest.json`

`manifest.json` currently maps to `CharacterArchiveManifest`. Its fields are:

- `format`
  - currently fixed to `sillystage_character_card`
- `version`
  - currently fixed to `1`
- `character_id`
  - character ID
- `content_path`
  - currently fixed to `content.json`
- `cover_path`
  - cover file path, must start with `cover.`
- `cover_mime_type`
  - currently supported values:
    - `image/png`
    - `image/jpeg`
    - `image/webp`

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

## 5. `cover` File

The cover file is binary data, not JSON. Its purpose is to provide a visual cover for the character card.

Current requirements:

- it must exist in the archive
- its path must match `manifest.json.cover_path`
- its bytes must not be empty
- its mime type is declared in `manifest.json.cover_mime_type`

The currently supported cover mime types are:

- `image/png`
- `image/jpeg`
- `image/webp`

## 6. Validation Rules

When the server parses a `.chr` file, it currently performs these key checks:

- `manifest.format` must equal `sillystage_character_card`
- `manifest.version` must equal `1`
- `manifest.content_path` must equal `content.json`
- `manifest.cover_path` must start with `cover.`
- `content.id` must equal `manifest.character_id`
- the archive must contain:
  - `manifest.json`
  - `content.json`
  - the cover file referenced by `manifest.cover_path`
- cover bytes must not be empty

If any of these checks fail, character archive parsing fails.

## 7. Relation to Runtime Character Objects

Protocol type `CharacterCardContent` maps one-to-one to runtime type `ss-agents::actor::CharacterCard`. The current implementation provides direct conversion in both directions:

- `CharacterCard -> CharacterCardContent`
- `CharacterCardContent -> CharacterCard`

That means:

- `.chr` `content.json` is the persisted exchange format of the character card
- agents work with the equivalent runtime character object

## 8. How the Server Stores Created Character Cards

### 8.1 Through `.chr` upload

The client uploads character cards through:

1. `upload.init`
2. `upload.chunk`
3. `upload.complete`

After completion, the server:

1. parses the `.chr` archive
2. extracts `manifest`, `content`, and `cover`
3. builds a character summary
4. stores the character object in the store

### 8.2 Through request data

The client can also create a character card directly through requests:

1. `character.create`
2. optional `character.set_cover`

In this flow:

- `character.create` stores character content only
- before a cover is set, `cover_file_name` / `cover_mime_type` are `null`
- after `character.set_cover`, the character has the full cover metadata needed for cover retrieval and `.chr` export

The current returned character summary includes:

- `character_id`
- `name`
- `personality`
- `style`
- `tendencies`
- `cover_file_name`
- `cover_mime_type`

If later code needs the full character card, it should read the character object itself instead of relying on the upload response alone.

## 9. How to Fetch the Character Cover

The protocol currently exposes a dedicated cover retrieval method:

- `character.get_cover`

It returns:

- `character_id`
- `cover_file_name`
- `cover_mime_type`
- `cover_base64`

So the frontend currently receives base64 text for the cover, not a standalone image download URL.
If the character does not have a cover yet, this method returns `conflict`.

## 10. How to Export the Full `.chr`

The protocol also exposes a method for exporting the full character archive:

- `character.export_chr`

It returns:

- `character_id`
- `file_name`
- `content_type`
- `chr_base64`

Where:

- `file_name` currently defaults to `<character_id>.chr`
- `content_type` is currently `application/x-sillystage-character-card`
- `chr_base64` is the base64-encoded full `.chr` ZIP content

So if the frontend wants to let users download a character card, it should call this method first and then convert the returned base64 into a downloadable file.
If the character does not have a cover yet, this method returns `conflict`, because the exported `.chr` archive must include a cover file.

## 11. Related Documents

For how character cards participate in the full product flow, continue with:

- `docs/en/process.md`
- `docs/en/api/spec.md`
- `docs/en/api/reference.md`
