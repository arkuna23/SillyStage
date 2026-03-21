# 角色卡结构与 `.chr` 文件

本文概括角色卡的内容结构、归档格式，以及导入导出方式。

## 1. 角色卡内容结构

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

关键字段：

- `schema_id`：引用角色私有状态 schema
- `tags`：角色卡标签
- `folder`：角色卡文件夹；空字符串表示未分组

## 2. 模板变量

支持的变量：

- `{{char}}`：当前角色展示名
- `{{user}}`：当前玩家名
- `{{field_name}}`：当前角色自己的 schema 字段值

替换范围：

- `personality`
- `style`
- `system_prompt`

## 3. `.chr` 归档格式

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

约束：

- `format` 必须为 `sillystage_character_card`
- `version` 当前固定为 `1`
- `content_path` 当前固定为 `content.json`
- `cover_path` 必须以 `cover.` 开头
- `character_id` 必须与 `content.json.id` 一致

支持的封面 MIME：

- `image/png`
- `image/jpeg`
- `image/webp`

## 5. 创建与导入

### 5.1 导入 `.chr`

- `POST /upload/character:{character_id}/archive`

规则：

- body 为原始 `.chr` 字节
- 客户端通常发送 `Content-Type: application/x-sillystage-character-card`
- 归档里的 `content.id` 必须与路径中的 `{character_id}` 一致

### 5.2 直接创建角色

1. `character.create`
2. 可选 `POST /upload/character:{character_id}/cover`

封面上传要求：

- body 为原始图片字节
- `Content-Type` 必须是 `image/png`、`image/jpeg` 或 `image/webp`

## 6. 读取与导出

- `character.list`：角色摘要
- `character.get`：完整角色内容
- `GET /download/character:{character_id}/cover`：封面字节
- `GET /download/character:{character_id}/archive`：导出 `.chr`

JSON payload 只暴露封面元数据：

- `cover_file_name`
- `cover_mime_type`
