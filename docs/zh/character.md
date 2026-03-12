# 角色卡结构与 `.chr` 文件格式

本文说明当前 SillyStage 角色卡的数据结构，以及角色卡归档文件 `.chr` 的打包格式。这里描述的是当前协议实现，不是一个开放式可扩展规范；如果实现变化，应以 `ss-protocol/src/character.rs` 为准。

## 1. 角色卡的作用

角色卡是 story 生成前的基础资源之一。当前有两种创建路径：

- 上传 `.chr` 文件，由服务端解析并存储
- 直接调用 `character.create` 创建内容，再调用 `character.set_cover` 补封面

无论哪种路径，服务端最终都会存储为同一种角色卡对象，并返回 `character_id`。之后：

- `story resources` 只引用 `character_id`
- `story` 生成时读取角色卡内容
- session 运行时再读取对应角色卡

也就是说，角色卡是“先上传、后引用”的对象，而不是在每次创建资源或 story 时重复内联传输。

## 2. 角色卡内容结构

角色卡的核心内容是 `content.json`，当前对应协议中的 `CharacterCardContent`，字段如下：

- `id`
  - 角色的稳定 ID
  - 必须与 `manifest.json.character_id` 一致
- `name`
  - 角色显示名
- `personality`
  - 对角色性格的简要描述
- `style`
  - 角色说话或行动风格
- `tendencies`
  - 角色倾向列表
  - 类型为字符串数组
- `state_schema`
  - 角色私有状态的 schema
  - key 为状态字段名，value 为 `StateFieldSchema`
- `system_prompt`
  - 供角色 agent 使用的系统提示词

一个最小示例如下：

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

## 3. `.chr` 文件格式

`.chr` 是一个 ZIP 归档，内部固定包含三个 entry：

- `manifest.json`
- `content.json`
- `cover.<ext>`

目录结构示例：

```text
merchant.chr
├── manifest.json
├── content.json
└── cover.png
```

当前协议约束如下：

- `manifest.json` 的路径必须固定为 `manifest.json`
- `content.json` 的路径必须固定为 `content.json`
- 封面文件路径必须以 `cover.` 开头
- 封面文件名通常由 mime type 派生：
  - `image/png` -> `cover.png`
  - `image/jpeg` -> `cover.jpg`
  - `image/webp` -> `cover.webp`

## 4. `manifest.json`

当前 `manifest.json` 对应 `CharacterArchiveManifest`，字段如下：

- `format`
  - 当前固定为 `sillystage_character_card`
- `version`
  - 当前固定为 `1`
- `character_id`
  - 角色 ID
- `content_path`
  - 当前固定为 `content.json`
- `cover_path`
  - 封面文件路径，必须以 `cover.` 开头
- `cover_mime_type`
  - 当前支持：
    - `image/png`
    - `image/jpeg`
    - `image/webp`

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

## 5. `cover` 文件

封面文件是一个二进制文件，不是 JSON。它的作用是提供角色卡的展示封面。

当前要求：

- 必须存在于归档中
- 路径必须与 `manifest.json.cover_path` 一致
- 字节内容不能为空
- mime type 由 `manifest.json.cover_mime_type` 声明

协议当前支持的封面 mime type 只有：

- `image/png`
- `image/jpeg`
- `image/webp`

## 6. 校验规则

服务端解析 `.chr` 时，当前会执行这些关键校验：

- `manifest.format` 必须等于 `sillystage_character_card`
- `manifest.version` 必须等于 `1`
- `manifest.content_path` 必须等于 `content.json`
- `manifest.cover_path` 必须以 `cover.` 开头
- `content.id` 必须等于 `manifest.character_id`
- 归档内必须存在：
  - `manifest.json`
  - `content.json`
  - `manifest.cover_path` 指向的封面文件
- 封面字节不能为空

任一条件不满足，角色卡解析都会失败。

## 7. 与运行时角色对象的关系

协议层的 `CharacterCardContent` 与运行时使用的 `ss-agents::actor::CharacterCard` 一一对应。当前实现提供了直接映射：

- `CharacterCard -> CharacterCardContent`
- `CharacterCardContent -> CharacterCard`

因此：

- `.chr` 的 `content.json` 是角色卡的持久化交换格式
- agent 运行时拿到的是等价的角色对象

## 8. 创建后服务端如何保存

### 8.1 通过 `.chr` 上传

客户端通过以下流程上传角色卡：

1. `upload.init`
2. `upload.chunk`
3. `upload.complete`

完成后，服务端会：

1. 解析 `.chr`
2. 提取 `manifest`、`content`、`cover`
3. 生成角色卡摘要
4. 把角色卡对象写入 store

### 8.2 通过请求数据创建

客户端也可以直接通过请求创建角色卡：

1. `character.create`
2. 可选 `character.set_cover`

这里的行为是：

- `character.create` 只写入角色内容
- 如果此时还没设置封面，`cover_file_name` / `cover_mime_type` 为 `null`
- 调用 `character.set_cover` 后，角色卡才具备可读取封面和可导出 `.chr` 的完整能力

服务端返回的角色卡摘要当前包含：

- `character_id`
- `name`
- `personality`
- `style`
- `tendencies`
- `cover_file_name`
- `cover_mime_type`

后续如果需要完整角色卡内容，应通过角色卡对象读取，而不是依赖上传响应本身。

## 9. 如何读取角色卡封面

当前协议提供单独的封面读取方法：

- `character.get_cover`

这个方法会返回：

- `character_id`
- `cover_file_name`
- `cover_mime_type`
- `cover_base64`

也就是说，当前前端读取封面时拿到的是 base64 文本，而不是独立的图片下载 URL。
如果角色还没有封面，这个接口会返回 `conflict`。

## 10. 如何导出完整 `.chr`

当前协议还提供完整角色卡归档的导出方法：

- `character.export_chr`

这个方法会返回：

- `character_id`
- `file_name`
- `content_type`
- `chr_base64`

其中：

- `file_name` 当前默认为 `<character_id>.chr`
- `content_type` 当前为 `application/x-sillystage-character-card`
- `chr_base64` 是完整 `.chr` ZIP 文件内容的 base64 编码

也就是说，当前前端如果要做“下载角色卡”，应先调用这个方法，再把返回的 base64 转成可下载文件。
如果角色还没有封面，这个接口会返回 `conflict`，因为 `.chr` 归档要求必须包含封面。

## 11. 关联文档

如果需要继续看角色卡如何参与整个产品流程，请阅读：

- `docs/zh/process.md`
- `docs/zh/api/spec.md`
- `docs/zh/api/reference.md`
