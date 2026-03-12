const en = {
  common: {
    actions: {
      openHealth: 'Health Check',
      reviewContract: 'View Transport',
      queueTurn: 'Queue Turn',
      saveDraft: 'Save Draft',
    },
    fields: {
      directorNote: 'Director Note',
      roleName: 'Role Name',
      sceneMood: 'Scene Mood',
    },
    language: 'Language',
    menu: 'Menu',
    navigation: 'Navigation',
    locales: {
      en: 'EN',
      'zh-CN': '中文',
    },
    theme: 'Theme',
    themes: {
      dark: 'Dark',
      light: 'Light',
      system: 'System',
    },
  },
  nav: {
    home: 'Home',
    workspace: 'Workspace',
  },
  home: {
    brand: {
      tagline: 'A control room for roleplay, stage cues, and live turns.',
    },
    landing: {
      start: 'Open Workspace',
      subtitle:
        'SillyStage is a stage-inspired engine for roleplay, designed to organize characters, prompts, and live creative rhythm in one place.',
      title: 'Give every roleplay scene a stage of its own.',
    },
    hero: {
      badge: 'Act I · Stage-First UI',
      eyebrow: 'Theater-grade interface',
      title: 'A bilingual shell, lit like a stage and ready for real scenes.',
      description:
        'SillyStage now has a reusable front-end base: theatrical black-gold styling, English and Simplified Chinese UI text, and a first batch of composable components for the scenes ahead.',
    },
    highlights: {
      metrics: {
        locales: 'Interface languages ready on day one',
        rpc: 'The contract still points to the current Rust backend',
        ui: 'The base layer uses local components on lightweight primitives',
      },
    },
    rehearsal: {
      eyebrow: 'Starter Form',
      title: 'Block the next scene before the actors walk on.',
      description:
        'This rehearsal panel previews the first shared form controls: input, select, textarea, and action buttons under the same visual system.',
      placeholders: {
        directorNote: 'Hold the line softly, then pivot into velvet menace.',
        roleName: 'Reimu Hakurei',
      },
      moods: {
        conflict: 'Conflict',
        intrigue: 'Intrigue',
        wonder: 'Wonder',
      },
    },
    principles: {
      description:
        'The frontend should stay expressive, but it still needs to map cleanly onto the existing backend boundaries and transport model.',
      eyebrow: 'Direction Cues',
      items: {
        composable: 'Use local UI components on top of Radix primitives so the app can grow without inheriting a rigid template system.',
        fidelity: 'Keep the frontend faithful to ss-protocol and the bilingual API docs instead of inventing a second wire format.',
        thinClient: 'Let Rust keep business behavior while the webapp handles presentation, interaction, and orchestrated request flow.',
      },
      title: 'Keep the interface close to the stage machinery.',
    },
    transport: {
      description:
        'The themed shell changes the presentation layer, not the backend contract. These surfaces remain the anchors for future screens.',
      eyebrow: 'Transport Surfaces',
      items: {
        health: {
          details: [
            'Use it as a quick stage-light check before trying larger session flows.',
            'It is a minimal readiness probe that stays decoupled from runtime internals.',
            'Helpful for smoke tests, local wiring, and deployment verification.',
          ],
          summary: 'A lightweight pulse check for local development and operational readiness.',
          tabLabel: 'Health',
          title: 'Health Signal',
        },
        rpc: {
          details: [
            'Use ss-protocol payloads directly instead of inventing a frontend-only request shape.',
            'Keep command orchestration in the webapp while business operations stay in the Rust handler layer.',
            'A strong fit for setup flows, CRUD actions, and explicit operator input.',
          ],
          summary: 'The main request and response lane for deliberate user actions.',
          tabLabel: 'RPC',
          title: 'Request / Response',
        },
        stream: {
          details: [
            'Consume server-sent events for live narration, planning, and tool execution updates.',
            'Preserve backend event semantics so the webapp remains a transport-aware presentation layer.',
            'This is the future home for live turn playback, actor traces, and director timing.',
          ],
          summary: 'Progressive runtime output should stay event-driven and visible like stage cues.',
          tabLabel: 'Streaming',
          title: 'Streaming Feed',
        },
      },
      title: 'The current backend contract still frames every future scene.',
    },
    stageKit: {
      description:
        'These are the first reusable building blocks for scenes, settings, and live operator flows. The home page uses them directly instead of hiding them behind one-off markup.',
      eyebrow: 'Stage Kit',
      items: {
        button: {
          description: 'Primary, secondary, and ghost buttons share one rhythm, focus style, and theatrical accent palette.',
          title: 'Buttons',
        },
        card: {
          description: 'Cards standardize panel depth, spacing, border treatment, and content grouping.',
          title: 'Cards',
        },
        input: {
          description: 'Text input and textarea controls inherit one visual language for editing prompts, names, and notes.',
          title: 'Inputs',
        },
        select: {
          description: 'Select uses Radix under the hood so future filters and editors can stay consistent.',
          title: 'Select',
        },
      },
      preview: {
        ghost: 'Ghost',
        info: 'Info',
        primary: 'Primary',
        secondary: 'Secondary',
        subtle: 'Subtle',
      },
      title: 'A small but reusable component base is now in motion.',
    },
  },
  workspace: {
    backHome: 'Back to Home',
    card: {
      description:
        'The workspace stays intentionally compact, but the core product surface is already in place: one shell, bilingual controls, and a steady navigation rhythm.',
      items: {
        i18n: 'Switch between English and Simplified Chinese without losing your place.',
        routing: 'Move across the app with animated transitions that keep the interface coherent.',
        shell: 'Keep one headbar and theme system across routes for a steadier working rhythm.',
      },
      kicker: 'Included Now',
      title: 'What this workspace already gives you',
    },
    description:
      'SillyStage keeps language, theme, and navigation controls within easy reach so the interface stays calm while you shape the session.',
    kicker: 'Creative Workspace',
    sidebar: {
      items: {
        characters: {
          label: 'Character Management',
        },
      },
      title: 'Dashboard Rail',
    },
    title: 'A focused workspace for scene-led creative control.',
  },
  characters: {
    actions: {
      back: 'Back',
      cancel: 'Cancel',
      chooseCover: 'Choose Cover',
      clearCover: 'Remove Cover',
      create: 'Create Character',
      export: 'Export .chr',
      exporting: 'Exporting...',
      import: 'Import Character Card',
      importing: 'Importing...',
      next: 'Next Step',
      removeStateField: 'Remove Field',
      replaceCover: 'Replace Cover',
      saving: 'Saving...',
      addStateField: 'Add State Field',
      addTendency: 'Add Tendency',
    },
    card: {
      coverAlt: '{{name}} cover',
      coverMissing: 'No cover attached yet.',
      coverPending: 'Cover Pending',
      idLabel: 'Character ID',
      noTendencies: 'No tendencies added yet',
      personality: 'Personality',
      style: 'Style',
      tendencies: 'Tendencies',
    },
    create: {
      errors: {
        coverTypeInvalid: 'Cover images must be PNG, JPEG, or WebP.',
        duplicateStateKey: 'State field keys must be unique.',
        idRequired: 'Character ID is required.',
        invalidDefault: {
          array: 'Array defaults must be valid JSON arrays.',
          bool: 'Boolean defaults must be true or false.',
          float: 'Float defaults must be valid numbers.',
          int: 'Integer defaults must be whole numbers.',
          object: 'Object defaults must be valid JSON objects.',
        },
        nameRequired: 'Character name is required.',
        personalityRequired: 'Personality is required.',
        stateKeyRequired: 'Every state row needs a key.',
        styleRequired: 'Style is required.',
        submitFailed: 'Failed to create the character card.',
      },
      fields: {
        characterId: 'Character ID',
        cover: 'Cover',
        name: 'Display Name',
        personality: 'Personality',
        stateDefault: 'Default Value',
        stateDescription: 'Field Description',
        stateKey: 'State Key',
        stateSchema: 'State Schema',
        stateType: 'Value Type',
        style: 'Speaking Style',
        systemPrompt: 'System Prompt',
        tendencies: 'Tendencies',
      },
      placeholders: {
        characterId: 'hakurei-reimu',
        cover: 'PNG, JPEG, or WebP cover art',
        name: 'Reimu Hakurei',
        personality:
          'Blunt, alert, and allergic to trouble, but always the one who steps in when an incident spirals.',
        stateDefault: 'true, 1, 3.14, ["secret"], or {"mood":"cold"}',
        stateDescription: 'What this field tracks about the current incident',
        stateKey: 'incident_status',
        style:
          'Direct, dry, and a little impatient, with the steady confidence of someone used to solving problems herself.',
        systemPrompt:
          'Stay grounded as the Hakurei shrine maiden: judge the incident first, keep the tone brisk, and never lose that effortless confidence.',
        tendency: 'Blunt, skeptical, money-minded, dependable when it matters...',
      },
      stateTypes: {
        array: 'Array',
        bool: 'Boolean',
        float: 'Float',
        int: 'Integer',
        null: 'Null',
        object: 'Object',
        string: 'String',
      },
      steps: {
        identity: {
          label: 'Identity',
        },
        system: {
          label: 'Prompt & State',
        },
        voice: {
          label: 'Voice',
        },
      },
      title: 'Create a character card',
    },
    empty: {
      title: 'Your cast table is still empty.',
    },
    feedback: {
      coverAttachFailed: 'The cover could not be attached.',
      created: '{{name}} is now in the library.',
      createdWithCoverWarning:
        '{{name}} was created, but the cover attachment needs another try.',
      exportFailed: 'Failed to export the character card.',
      exported: '{{name}} was exported as .chr.',
      exportNeedsCover:
        '{{name}} needs a cover before it can be exported as .chr.',
      imported: '{{name}} was imported into the library.',
      importFailed: 'Failed to import the character card.',
      invalidImportType: 'Only .chr character card files can be imported here.',
      loadFailed: 'Failed to load the character library.',
    },
    metrics: {
      covered: 'Covers Ready',
      total: 'Characters',
    },
    title: 'Character Management',
  },
} as const

export default en
