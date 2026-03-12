const zhCN = {
  common: {
    actions: {
      openHealth: '查看服务状态',
      reviewContract: '查看接口入口',
      queueTurn: '发起回合',
      saveDraft: '保存草稿',
    },
    fields: {
      directorNote: '导演提示',
      roleName: '角色名称',
      sceneMood: '场景氛围',
    },
    language: '语言',
    menu: '菜单',
    navigation: '导航',
    locales: {
      en: 'EN',
      'zh-CN': '中文',
    },
    theme: '主题',
    themes: {
      dark: '暗色',
      light: '亮色',
      system: '跟随系统',
    },
  },
  nav: {
    home: '首页',
    workspace: '工作台',
  },
  home: {
    brand: {
      tagline: '一个面向角色扮演、舞台调度和实时回合的创作控制台。',
    },
    landing: {
      start: '进入工作台',
      subtitle:
        'SillyStage 是一款以舞台为原型的角色扮演引擎，用更清晰的角色、提示与节奏来组织角色扮演。',
      title: '让角色扮演有自己的舞台。',
    },
    hero: {
      badge: '第一幕 · 前端底座',
      eyebrow: '一套带舞台气质的前端起点',
      title: '一个支持中英双语、带舞台气质，也能继续长大的前端起点。',
      description:
        'SillyStage 现在已经有了第一版前端底座：统一的剧场黑金主题、可切换的中英文界面，以及一组可复用的小组件，后面的真实页面可以直接接着往上搭。',
    },
    highlights: {
      metrics: {
        locales: '首页已经支持中英双语切换',
        rpc: '当前接口仍直接对接现有 Rust 后端',
        ui: '本地组件配合轻量 primitives，方便继续扩展',
      },
    },
    rehearsal: {
      eyebrow: '预演面板',
      title: '在演员上场之前，先把这一幕的调度写清楚。',
      description:
        '这里用第一批通用表单组件做了个示范：输入框、下拉选择、文本域和动作按钮都已经纳入同一套视觉和交互规范。',
      placeholders: {
        directorNote: '先把语气压低一些，再慢慢带出那种不动声色的压迫感。',
        roleName: '博丽灵梦',
      },
      moods: {
        conflict: '冲突',
        intrigue: '悬念',
        wonder: '奇想',
      },
    },
    principles: {
      description:
        '前端当然可以有风格，但前提还是要和现有后端边界、传输方式以及协议模型对得上。',
      eyebrow: '设计原则',
      items: {
        composable: '在 Radix primitives 之上封装本地 UI 组件，让后续页面能继续长，而不是被模板体系反过来限制。',
        fidelity: '前端直接贴着 ss-protocol 和双语 API 文档走，不额外发明另一套数据格式。',
        thinClient: '业务行为继续留在 Rust 层，webapp 负责展示、交互，以及有边界的请求编排。',
      },
      title: '界面可以有戏，但底层边界必须清楚。',
    },
    transport: {
      description:
        '主题和布局可以升级，但对接后端的方式不能飘。后面的真实页面，都会从这些现有入口往外长。',
      eyebrow: '接口入口',
      items: {
        health: {
          details: [
            '在接更复杂的会话流之前，可以先用它确认服务是不是已经正常起来了。',
            '它只返回最小化的就绪信号，不会把前端绑进运行时内部状态。',
            '适合冒烟测试、本地联调和部署验收。',
          ],
          summary: '用于本地开发和运行状态确认的轻量健康信号。',
          tabLabel: '健康',
          title: '健康检查',
        },
        rpc: {
          details: [
            '前端直接使用 ss-protocol 定义的请求体，不再额外包一层只给前端使用的结构。',
            '请求编排留在 webapp，真正的业务处理仍然放在 Rust handler 层。',
            '适合初始化流程、显式操作以及常规 CRUD 类动作。',
          ],
          summary: '面向明确操作的主请求 / 响应通道。',
          tabLabel: 'RPC',
          title: '请求 / 响应',
        },
        stream: {
          details: [
            '通过 SSE 接收实时叙述、规划过程和工具执行更新。',
            '尽量保留后端事件语义，让前端把重点放在展示和交互上。',
            '后面做实时回合、演员轨迹和导演节奏时，这里会是核心入口。',
          ],
          summary: '渐进式输出更适合按事件流展示，像舞台提示一样持续出现。',
          tabLabel: '流式',
          title: '流式输出',
        },
      },
      title: '现有后端契约，仍然是后续页面的基础入口。',
    },
    stageKit: {
      description:
        '这是一组面向场景、设定和实时操作流的基础组件。当前首页直接拿它们搭页面，后面做真实功能也可以沿用这套底子。',
      eyebrow: '基础组件',
      items: {
        button: {
          description: '主按钮、次按钮和 Ghost 按钮共享同一套节奏、焦点样式和强调色。',
          title: '按钮',
        },
        card: {
          description: 'Card 统一了承载面板的层级、留白、边框处理和内容分组方式。',
          title: '卡片',
        },
        input: {
          description: '输入框和文本域沿用同一套视觉语言，后面可以直接接到 prompt、角色名和备注编辑上。',
          title: '输入控件',
        },
        select: {
          description: 'Select 基于 Radix 封装，后续做筛选和编辑器时可以直接复用。',
          title: '下拉选择',
        },
      },
      preview: {
        ghost: 'Ghost',
        info: '信息',
        primary: '主按钮',
        secondary: '次按钮',
        subtle: '弱强调',
      },
      title: '第一批可复用组件已经成型，可以开始承载真实场景。',
    },
  },
  workspace: {
    backHome: '回到首页',
    card: {
      description:
        '这个工作台保持了克制的结构，但产品最核心的一层已经在这里：统一的页面壳、双语控制，以及稳定的导航节奏。',
      items: {
        i18n: '中英文界面可以随时切换，不会打断你当前所在的位置。',
        routing: '页面切换带过渡动画，跳转时依然能保持连贯的使用节奏。',
        shell: '同一套 headbar 和主题系统会贯穿不同页面，界面体验更稳定。',
      },
      kicker: '当前体验',
      title: '这个工作台现在已经具备什么',
    },
    description:
      'SillyStage 把导航、语言和主题控制收在同一块稳定的界面里，让你进入创作时不用被界面本身打断。',
    kicker: '创作工作台',
    sidebar: {
      items: {
        characters: {
          label: '角色管理',
        },
      },
      title: '工作台导航',
    },
    title: '一个围绕场景创作而设计的工作台。',
  },
  characters: {
    actions: {
      back: '上一步',
      cancel: '取消',
      chooseCover: '选择封面',
      clearCover: '移除封面',
      create: '创建角色卡',
      export: '导出 .chr',
      exporting: '正在导出...',
      import: '导入角色卡',
      importing: '正在导入...',
      next: '下一步',
      removeStateField: '移除字段',
      replaceCover: '更换封面',
      saving: '正在保存...',
      addStateField: '添加状态字段',
      addTendency: '添加倾向',
    },
    card: {
      coverAlt: '{{name}} 的封面',
      coverMissing: '当前还没有封面。',
      coverPending: '封面待补',
      idLabel: '角色 ID',
      noTendencies: '还没有添加倾向',
      personality: '性格',
      style: '表现风格',
      tendencies: '倾向',
    },
    create: {
      errors: {
        coverTypeInvalid: '封面图片只支持 PNG、JPEG 或 WebP。',
        duplicateStateKey: '状态字段 key 不能重复。',
        idRequired: '请填写角色 ID。',
        invalidDefault: {
          array: '数组默认值需要是合法的 JSON 数组。',
          bool: '布尔默认值只能填写 true 或 false。',
          float: '浮点默认值需要是合法数字。',
          int: '整数默认值需要是合法整数。',
          object: '对象默认值需要是合法的 JSON 对象。',
        },
        nameRequired: '请填写角色名称。',
        personalityRequired: '请填写角色性格。',
        stateKeyRequired: '每个状态字段都需要 key。',
        styleRequired: '请填写表现风格。',
        submitFailed: '创建角色卡失败。',
      },
      fields: {
        characterId: '角色 ID',
        cover: '封面',
        name: '显示名称',
        personality: '性格',
        stateDefault: '默认值',
        stateDescription: '字段说明',
        stateKey: '状态 key',
        stateSchema: '状态 Schema',
        stateType: '值类型',
        style: '表现风格',
        systemPrompt: 'System Prompt',
        tendencies: '倾向',
      },
      placeholders: {
        characterId: 'hakurei-reimu',
        cover: '选择 PNG、JPEG 或 WebP 封面',
        name: '博丽灵梦',
        personality:
          '直率、警觉、嫌麻烦，但在关键时刻总会站出来处理异变。',
        stateDefault: 'true、1、3.14、["secret"] 或 {"mood":"cold"}',
        stateDescription: '这个字段记录当前异变的处理进度',
        stateKey: 'incident_status',
        style: '说话干脆，不绕弯子，带一点不耐烦，但总能稳住场面。',
        systemPrompt:
          '始终以博丽神社巫女的身份行动，先判断异变和来意，再用简洁直接的口吻回应。',
        tendency: '嘴硬、护短、嫌麻烦、关键时刻会认真起来...',
      },
      stateTypes: {
        array: '数组',
        bool: '布尔',
        float: '浮点',
        int: '整数',
        null: '空值',
        object: '对象',
        string: '字符串',
      },
      steps: {
        identity: {
          label: '身份',
        },
        system: {
          label: 'Prompt 与状态',
        },
        voice: {
          label: '语气与表现',
        },
      },
      title: '创建一张角色卡',
    },
    empty: {
      title: '你的角色库还是空的。',
    },
    feedback: {
      coverAttachFailed: '封面附加失败。',
      created: '已将 {{name}} 加入角色库。',
      createdWithCoverWarning:
        '{{name}} 已创建成功，但封面还需要再上传一次。',
      exportFailed: '导出角色卡失败。',
      exported: '已将 {{name}} 导出为 .chr。',
      exportNeedsCover: '{{name}} 还没有封面，暂时不能导出为 .chr。',
      imported: '已将 {{name}} 导入角色库。',
      importFailed: '导入角色卡失败。',
      invalidImportType: '这里只支持导入 .chr 角色卡文件。',
      loadFailed: '角色库加载失败。',
    },
    metrics: {
      covered: '已补封面',
      total: '角色数量',
    },
    title: '角色管理',
  },
} as const

export default zhCN
