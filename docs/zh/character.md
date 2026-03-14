# 角色卡结构与文件格式

本文档描述当前角色卡的内容结构、`.chr` 文件格式，以及角色卡与独立 schema 资源的关系。

## 1. 角色卡内容

当前角色卡内容结构为：

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

字段说明：

- `id`: 角色 id
- `name`: 展示名称
- `personality`: 性格摘要
- `style`: 语言或表现风格
- `tendencies`: 行为倾向列表
- `schema_id`: 引用角色私有状态 schema 的 id
- `system_prompt`: actor 使用的角色级 system prompt

注意：

- 角色卡不再内联 `state_schema`
- 私有状态 schema 通过 `schema_id` 引用独立 `schema` 资源

## 2. `.chr` 文件格式

`.chr` 是一个 ZIP 容器，固定包含：

- `manifest.json`
- `content.json`
- `cover.<ext>`

目录示例：

```text
merchant.chr
├── manifest.json
├── content.json
└── cover.png
```

## 3. `manifest.json`

示例：

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

约束：

- `format` 必须为 `sillystage_character_card`
- `version` 当前固定为 `1`
- `content_path` 当前固定为 `content.json`
- `cover_path` 必须以 `cover.` 开头
- `character_id` 必须与 `content.json` 中的 `id` 一致

支持的封面 MIME：

- `image/png`
- `image/jpeg`
- `image/webp`

## 4. `content.json`

`content.json` 使用上面的角色卡内容结构，也就是 `CharacterCardContent`。

关键点：

- `schema_id` 是必须字段
- 它引用后端中已存在或将被管理的 `schema` 资源

## 5. `cover`

封面文件是 ZIP 里的单独二进制 entry。

常见文件名：

- `cover.png`
- `cover.jpg`
- `cover.webp`

要求：

- 封面字节不能为空
- MIME 与文件扩展名应匹配

## 6. 两种创建方式

当前角色卡支持两种创建路径。

### 6.1 上传 `.chr`

通过：

1. `upload.init`
2. `upload.chunk`
3. `upload.complete`

服务端会解析 `.chr`，并创建角色卡对象。

### 6.2 直接创建对象

通过：

1. `character.create`
2. 可选 `character.set_cover`

这种方式适合前端表单编辑器。

## 7. 读取与导出

### 7.1 获取角色卡内容

- `character.list`: 获取角色摘要
- `character.get`: 获取完整角色内容

### 7.2 获取封面

- `character.get_cover`

返回：

- `character_id`
- `cover_file_name`
- `cover_mime_type`
- `cover_base64`

### 7.3 导出 `.chr`

- `character.export_chr`

返回：

- `character_id`
- `file_name`
- `content_type`
- `chr_base64`

服务端会基于当前存储中的角色卡内容和封面重新打包 `.chr`，不要求与最初上传的原始字节完全一致。

