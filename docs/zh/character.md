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
  "schema_id": "schema-character-merchant",
  "system_prompt": "你是 {{char}}。请自然地对 {{user}} 说话，并保持沉浸感。",
  "tags": ["merchant", "shop"],
  "folder": "harbor/npcs"
}
```

字段说明：

- `id`: 角色 id
- `name`: 展示名称
- `personality`: 性格摘要
- `style`: 语言或表现风格
- `schema_id`: 引用角色私有状态 schema 的 id
- `system_prompt`: actor 使用的角色级 system prompt
- `tags`: 角色卡标签列表
- `folder`: 角色卡文件夹分组；空字符串表示未分组

模板变量：

- `{{char}}`：运行时替换为角色展示名
- `{{user}}`：运行时替换为当前玩家名；如果当前没有玩家名，则回退为 `User`
- `{{field_name}}`：运行时替换为当前角色自己的该 schema 字段值

替换范围：

- `personality`
- `style`
- `system_prompt`

Schema 变量规则：

- 后端从 `world_state.character_state[character_id][field_name]` 读取当前值
- 如果当前没有运行时值，则回退到该 schema 字段的 `default`
- 如果运行时值和 schema 默认值都不存在，则保留原始占位符不变
- 字符串直接按原文替换
- 数字和布尔值替换为紧凑文本
- 数组、对象和 `null` 替换为紧凑 JSON 文本
- `char` 和 `user` 是保留变量名，不从 schema 字段读取

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

### 6.1 导入 `.chr`

通过：

1. `POST /upload/character:{character_id}/archive`

请求规则：

- body 是原始 `.chr` 字节
- 不使用 JSON-RPC envelope
- 不使用 base64 包装
- 客户端通常应发送 `Content-Type: application/x-sillystage-character-card`
- 压缩包中的 `content.id` 必须与 `{character_id}` 一致

服务端会解析归档、在内部保存封面，并创建角色卡对象。成功返回的 JSON body 是
`character:{character_id}/archive` 对应的 `ResourceFilePayload`。

### 6.2 直接创建对象

通过：

1. `character.create`
2. 可选 `POST /upload/character:{character_id}/cover`

封面上传规则：

- body 是原始图片字节
- `Content-Type` 必须是 `image/png`、`image/jpeg` 或 `image/webp`
- 可选 `x-file-name` 用来保存封面文件名
- 如果未提供 `x-file-name`，后端会使用 `cover.<ext>`

这种方式适合前端表单编辑器。

## 7. 读取与导出

### 7.1 获取角色卡内容

- `character.list`: 获取角色摘要
- `character.get`: 获取完整角色内容
- JSON payload 返回封面元数据，而不是直接内联封面字节

详情 payload 结构示例：

```json
{
  "character_id": "merchant",
  "content": {
    "id": "merchant",
    "name": "Old Merchant",
    "personality": "greedy but friendly trader",
    "style": "talkative, casual, slightly cunning",
    "schema_id": "schema-character-merchant",
    "system_prompt": "你是 {{char}}。请自然地对 {{user}} 说话，并保持沉浸感。",
    "tags": ["merchant", "shop"],
    "folder": "harbor/npcs"
  },
  "tags": ["merchant", "shop"],
  "folder": "harbor/npcs",
  "cover_file_name": "cover.png",
  "cover_mime_type": "image/png"
}
```

说明：

- 当角色尚未设置封面时，`cover_file_name` 与 `cover_mime_type` 为 `null`
- 当角色未设置文件夹时，`folder` 返回空字符串
- `tags` 始终返回数组，未设置时为空数组
- `character.list` 的摘要也会暴露相同的封面元数据字段
- 可以通过 `GET /download/character:{character_id}/cover` 按角色 id 获取当前封面

### 7.2 获取封面字节

- `GET /download/character:{character_id}/cover`

返回：

- HTTP body 中的原始封面字节
- `Content-Type` 为已保存的封面 MIME 类型
- 如果存在封面文件名，则可能返回附件 `Content-Disposition` 文件名

### 7.3 导出 `.chr`

- `GET /download/character:{character_id}/archive`

返回：

- HTTP body 中的原始 `.chr` 字节
- `Content-Type: application/x-sillystage-character-card`
- 可选附件文件名，通常是 `{character_id}.chr`

服务端会基于当前存储中的角色卡内容和封面重新打包 `.chr`，不要求与最初上传的原始字节完全一致。
