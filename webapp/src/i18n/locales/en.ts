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
        dashboard: {
          label: 'Dashboard',
        },
        apis: {
          label: 'API Management',
        },
        characters: {
          label: 'Character Management',
        },
        playerProfiles: {
          label: 'Player Profiles',
        },
        schemas: {
          label: 'State Schemas',
        },
        storyResources: {
          label: 'Story Resources',
        },
      },
      title: 'Workspace Navigation',
    },
    rail: {
      actions: {
        close: 'Hide panel info',
        open: 'Show panel info',
      },
      heading: 'Current Panel',
    },
    title: 'A focused workspace for scene-led creative control.',
  },
  dashboard: {
    config: {
      description: 'Check whether the global default API assignments are in place.',
      emptyValue: 'Not set',
      summary: '{{assigned}} / {{total}} configured',
      summaryLabel: 'Default APIs',
    },
    counts: {
      characters: 'Characters',
      covers: 'With Covers',
      sessions: 'Sessions',
      stories: 'Stories',
      storyResources: 'Story Resources',
    },
    feedback: {
      loadFailed: 'Failed to load the dashboard.',
    },
    health: {
      ok: 'Healthy',
    },
    metrics: {
      activity: 'Recent Activity',
      resources: 'Resources',
      status: 'System',
    },
    rail: {
      description: 'Review workspace health, resource volume, and recent activity at a glance.',
    },
    recentSessions: {
      empty: 'No recent sessions yet.',
      storyPrefix: 'Story {{id}}',
      turnPrefix: 'Turn {{turn}}',
    },
    recentStories: {
      empty: 'No recent stories yet.',
      resourcePrefix: 'Resource {{id}}',
    },
    sections: {
      defaults: 'Global Defaults',
      health: 'Runtime Health',
      overview: 'Resource Overview',
      recentSessions: 'Recent Sessions',
      recentStories: 'Recent Stories',
    },
    title: 'Dashboard',
  },
  apis: {
    actions: {
      cancel: 'Cancel',
      close: 'Close',
      confirmDelete: 'Delete API',
      create: 'New API',
      delete: 'Delete',
      deleting: 'Deleting...',
      edit: 'Edit',
      save: 'Save',
      saveAssignments: 'Save Defaults',
      saving: 'Saving...',
      view: 'View',
    },
    assignments: {
      description: 'Choose which stored API each agent role should use by default.',
      empty: 'Create at least one API before assigning defaults.',
      roles: {
        actor_api_id: 'Actor',
        architect_api_id: 'Architect',
        director_api_id: 'Director',
        keeper_api_id: 'Keeper',
        narrator_api_id: 'Narrator',
        planner_api_id: 'Planner',
      },
      selectPlaceholder: 'Choose an API',
      title: 'Global Defaults',
    },
    deleteDialog: {
      conflict: 'This API is still referenced by global or session config.',
      message: 'Delete {{id}}? This action cannot be undone.',
      title: 'Delete API',
    },
    details: {
      apiKey: 'Masked Key',
      title: 'API Details',
    },
    feedback: {
      created: 'API {{id}} created.',
      defaultsSaved: 'Global defaults updated.',
      deleteFailed: 'Failed to delete the API.',
      deleted: 'API {{id}} deleted.',
      detailsLoadFailed: 'Failed to load API details.',
      loadApisFailed: 'Failed to load APIs.',
      loadConfigFailed: 'Failed to load global defaults.',
      updated: 'API {{id}} updated.',
    },
    form: {
      createTitle: 'New API',
      editTitle: 'Edit API',
      errors: {
        apiIdDuplicate: 'API ID already exists.',
        apiIdRequired: 'API ID is required.',
        apiKeyRequired: 'API key is required.',
        baseUrlRequired: 'Base URL is required.',
        loadFailed: 'Failed to load API details.',
        modelRequired: 'Model is required.',
        submitFailed: 'Failed to save the API.',
      },
      fields: {
        apiId: 'API ID',
        apiKey: 'API Key',
        apiKeyHint: 'Leave this empty to keep the current key.',
        baseUrl: 'Base URL',
        model: 'Model',
        provider: 'Provider',
      },
      placeholders: {
        apiId: 'openai-main',
        apiKey: 'sk-...',
        baseUrl: 'https://api.openai.com/v1',
        model: 'gpt-4.1-mini',
      },
    },
    list: {
      apiKey: 'API Key',
      emptyTitle: 'No APIs yet.',
      keyConfigured: 'Configured',
      keyMissing: 'Not configured',
      model: 'Model',
      title: 'Stored APIs',
    },
    metrics: {
      assigned: 'Assigned Defaults',
      total: 'Stored APIs',
    },
    providers: {
      open_ai: 'OpenAI-compatible',
    },
    rail: {
      description: 'Manage stored LLM endpoints and decide which default API each agent role should use.',
    },
    title: 'API Management',
  },
  schemas: {
    actions: {
      addField: 'Add Field',
      addTag: 'Add Tag',
      cancel: 'Cancel',
      create: 'New Schema',
      delete: 'Delete',
      deleting: 'Deleting...',
      edit: 'Edit',
      removeField: 'Remove Field',
      save: 'Save',
      saving: 'Saving...',
    },
    deleteDialog: {
      conflict: 'This schema is still referenced by characters, resources, stories, or sessions.',
      message: 'Delete {{name}} ({{id}})? This action cannot be undone.',
      title: 'Delete Schema',
    },
    empty: {
      title: 'No schemas yet.',
    },
    feedback: {
      created: '{{name}} created.',
      deleteFailed: 'Failed to delete the schema.',
      deleted: '{{name}} deleted.',
      loadFailed: 'Failed to load schemas.',
      loadSchemaFailed: 'Failed to load schema details.',
      updated: '{{name}} updated.',
    },
    form: {
      createTitle: 'New Schema',
      editTitle: 'Edit Schema',
      emptyFields: 'No fields yet. Add the first one when this schema needs runtime state.',
      errors: {
        displayNameRequired: 'Display name is required.',
        duplicateFieldKey: 'Field keys must be unique.',
        fieldKeyRequired: 'Every field needs a key.',
        invalidDefault: {
          array: 'Array defaults must be valid JSON arrays.',
          bool: 'Boolean defaults must be true or false.',
          float: 'Float defaults must be valid numbers.',
          int: 'Integer defaults must be whole numbers.',
          object: 'Object defaults must be valid JSON objects.',
        },
        schemaIdDuplicate: 'Schema ID already exists.',
        schemaIdRequired: 'Schema ID is required.',
        submitFailed: 'Failed to save the schema.',
      },
      fields: {
        displayName: 'Display Name',
        fieldDefault: 'Default Value',
        fieldDescription: 'Field Description',
        fieldKey: 'Field Key',
        fields: 'Fields',
        fieldType: 'Value Type',
        schemaId: 'Schema ID',
        schemaIdHint: 'Schema ID stays fixed after creation.',
        tags: 'Tags',
      },
      placeholders: {
        displayName: 'Hakurei Shrine State',
        fieldDefault: 'true, 1, 3.14, ["secret"], or {"mood":"cold"}',
        fieldDescription: 'Tracks the current shrine incident progress',
        fieldKey: 'incident_status',
        schemaId: 'schema-character-reimu',
        tag: 'character',
      },
      valueTypes: {
        array: 'Array',
        bool: 'Boolean',
        float: 'Float',
        int: 'Integer',
        null: 'Null',
        object: 'Object',
        string: 'String',
      },
    },
    list: {
      fieldsCount: '{{count}} fields',
      noTags: 'No tags',
    },
    metrics: {
      character: 'Character-tagged',
      player: 'Player-tagged',
      total: 'Schema Count',
    },
    rail: {
      description: 'Manage reusable state schemas for characters, players, and world-building seeds.',
    },
    title: 'State Schemas',
  },
  playerProfiles: {
    actions: {
      cancel: 'Cancel',
      confirmDelete: 'Delete Profile',
      create: 'New Profile',
      delete: 'Delete',
      deleting: 'Deleting...',
      edit: 'Edit',
      saveChanges: 'Save Changes',
      saving: 'Saving...',
    },
    deleteDialog: {
      conflict: 'This player profile is still referenced by a session.',
      message: 'Delete {{name}} ({{id}})? This action cannot be undone.',
      title: 'Delete Player Profile',
    },
    empty: {
      description: 'Store player setups here so later sessions can switch between them cleanly.',
      title: 'No player profiles yet.',
    },
    feedback: {
      created: '{{name}} created.',
      deleteFailed: 'Failed to delete the player profile.',
      deleted: '{{name}} deleted.',
      loadFailed: 'Failed to load player profile details.',
      loadListFailed: 'Failed to load player profiles.',
      updated: '{{name}} updated.',
    },
    form: {
      createTitle: 'New Player Profile',
      editTitle: 'Edit Player Profile',
      errors: {
        descriptionRequired: 'Description is required.',
        displayNameRequired: 'Display name is required.',
        playerProfileIdDuplicate: 'Player profile ID already exists.',
        playerProfileIdRequired: 'Player profile ID is required.',
        submitFailed: 'Failed to save the player profile.',
      },
      fields: {
        description: 'Description',
        displayName: 'Display Name',
        playerProfileId: 'Player Profile ID',
        playerProfileIdHint: 'Player profile ID stays fixed after creation.',
      },
      placeholders: {
        description:
          'A shrine maiden who treats incidents as routine work, speaks plainly, and protects familiar places without making a show of it.',
        displayName: 'Reimu as Player',
        playerProfileId: 'player-reimu',
      },
    },
    list: {
      description: 'Store reusable player setups here and keep them ready for future sessions.',
      descriptionLabel: 'Description',
      title: 'Stored Player Profiles',
    },
    metrics: {
      total: 'Profile Count',
    },
    rail: {
      description: 'Manage reusable player setups that sessions can switch between later.',
    },
    title: 'Player Profiles',
  },
  storyResources: {
    actions: {
      cancel: 'Cancel',
      confirmDelete: 'Delete Resource',
      create: 'New Resource',
      createAndGenerate: 'Create & Generate',
      delete: 'Delete',
      deleting: 'Deleting...',
      edit: 'Edit',
      generate: 'Generate Draft',
      generating: 'Generating...',
      saveChanges: 'Save Changes',
      saveAndGenerate: 'Save & Generate',
      saving: 'Saving...',
    },
    deleteDialog: {
      conflict: 'This resource is still referenced by a story and cannot be deleted.',
      message: 'Delete {{id}}? This action cannot be undone.',
      title: 'Delete Story Resource',
    },
    empty: {
      description: 'Create one resource bundle first, then let Planner turn it into an editable draft.',
      title: 'No story resources yet.',
    },
    feedback: {
      created: 'Resource {{id}} created.',
      deleteFailed: 'Failed to delete the resource.',
      deleted: 'Resource {{id}} deleted.',
      generateFailed: 'Failed to generate the draft.',
      generated: 'Draft generated for {{id}}.',
      loadFailed: 'Failed to load story resources.',
      loadReferencesFailed: 'Failed to load available characters or schemas.',
      loadResourceFailed: 'Failed to load the resource details.',
      savedButGenerateFailed: 'The resource was saved, but draft generation failed.',
      updated: 'Resource {{id}} updated.',
    },
    form: {
      createTitle: 'New Story Resource',
      editTitle: 'Edit Story Resource',
      emptyCharacters: 'No characters are available yet. Create or import at least one character first.',
      emptySelection: 'Choose at least one character.',
      errors: {
        charactersRequired: 'Choose at least one character.',
        storyConceptRequired: 'Story concept is required.',
        submitFailed: 'Failed to save the resource.',
      },
      fields: {
        characters: 'Characters',
        plannedStory: 'Draft Script',
        playerSchemaIdSeed: 'Player Schema Seed',
        resourceId: 'Resource ID',
        resourceIdHint: 'Resource ID is generated by the backend and stays fixed.',
        storyConcept: 'Story Concept',
        worldSchemaIdSeed: 'World Schema Seed',
      },
      placeholders: {
        plannedStory: 'Planner output will be written back here. You can also edit it manually.',
        schemaSeed: 'Choose an optional schema seed',
        storyConcept: 'A short prompt about the conflict, cast, and opening stage situation.',
      },
    },
    list: {
      charactersCount: '{{count}} characters',
      notPlanned: 'No Draft',
      planned: 'Draft Ready',
      title: 'Stored Story Resources',
    },
    metrics: {
      planned: 'Drafts Ready',
      total: 'Resource Count',
    },
    rail: {
      description: 'Manage the editable resource bundles that Planner and later story generation read from.',
    },
    title: 'Story Resources',
  },
  characters: {
    actions: {
      back: 'Back',
      cancel: 'Cancel',
      cancelSelection: 'Exit selection mode',
      chooseCover: 'Choose Cover',
      closeDetails: 'Close',
      clearCover: 'Remove Cover',
      create: 'Create Character',
      delete: 'Delete',
      deleteSelected: 'Delete Selected',
      deleting: 'Deleting...',
      edit: 'Edit',
      export: 'Export .chr',
      exporting: 'Exporting...',
      import: 'Import Character Card',
      importing: 'Importing...',
      next: 'Next Step',
      removeStateField: 'Remove Field',
      replaceCover: 'Replace Cover',
      saveChanges: 'Save Changes',
      saving: 'Saving...',
      addStateField: 'Add State Field',
      addTendency: 'Add Tendency',
      selectAll: 'Select All',
      selectMode: 'Selection Mode',
      viewDetails: 'View Details',
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
        schemaIdRequired: 'Choose a schema before saving this character.',
        stateKeyRequired: 'Every state row needs a key.',
        styleRequired: 'Style is required.',
        submitFailed: 'Failed to save the character card.',
      },
      fields: {
        characterId: 'Character ID',
        cover: 'Cover',
        name: 'Display Name',
        personality: 'Personality',
        schemaId: 'Schema',
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
        schemaId: 'Choose a saved schema',
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
          label: 'Prompt & Schema',
        },
        voice: {
          label: 'Voice',
        },
      },
      schemaEmpty: {
        action: 'Open State Schemas',
        description:
          'Characters now reference an existing schema resource. Create one in State Schemas first, then come back here.',
        title: 'No schema is available yet.',
      },
      title: 'Create a character card',
    },
    deleteDialog: {
      messageMany: 'Delete {{count}} selected characters? This action cannot be undone.',
      messageOne: 'Delete {{name}}? This action cannot be undone.',
      more: '+{{count}} more',
      titleMany: 'Delete characters',
      titleOne: 'Delete character',
    },
    empty: {
      title: 'Your cast table is still empty.',
    },
    edit: {
      characterIdHint: 'Character ID is locked after creation.',
      coverHint: 'Current cover: {{fileName}}. Choose a new file to replace it.',
      title: 'Edit character card',
    },
    feedback: {
      coverAttachFailed: 'The cover could not be attached.',
      created: '{{name}} is now in the library.',
      createdWithCoverWarning:
        '{{name}} was created, but the cover attachment needs another try.',
      deleteFailed: 'Failed to delete the selected characters.',
      deleted: '{{name}} was deleted.',
      deletedMany: '{{count}} characters were deleted.',
      deletedPartial:
        '{{success}} characters were deleted, but {{failed}} could not be removed.',
      exportFailed: 'Failed to export the character card.',
      exported: '{{name}} was exported as .chr.',
      exportNeedsCover:
        '{{name}} needs a cover before it can be exported as .chr.',
      imported: '{{name}} was imported into the library.',
      importFailed: 'Failed to import the character card.',
      invalidImportType: 'Only .chr character card files can be imported here.',
      loadCharacterFailed: 'Failed to load the character card.',
      loadFailed: 'Failed to load the character library.',
      loadSchemasFailed: 'Failed to load available schemas.',
      updated: '{{name}} was updated.',
      updatedWithCoverWarning:
        '{{name}} was updated, but the cover replacement needs another try.',
    },
    rail: {
      description: 'Create, import, and export character cards from one place.',
    },
    metrics: {
      covered: 'Covers Ready',
      total: 'Characters',
    },
    views: {
      grid: 'Cards',
      label: 'View mode',
      list: 'List',
    },
    title: 'Character Management',
    selection: {
      count: '{{count}} selected',
    },
  },
} as const

export default en
