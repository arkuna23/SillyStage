import type { LorebookEntry } from '../lorebooks/types'

export type DemoLorebook = {
  displayName: string
  entries: LorebookEntry[]
  lorebookId: string
}

function isChineseDemoLanguage(language?: string) {
  return language?.toLocaleLowerCase().startsWith('zh') ?? false
}

const zhDemoLorebookEntries: LorebookEntry[] = [
  {
    always_include: true,
    content:
      '这是一个典型的剑与魔法冒险世界。多数人围绕城镇、公会、商路与遗迹生活；魔物威胁长期存在，但还没有彻底压垮文明。冒险者既是解决麻烦的人，也是最常直面风险的人。',
    enabled: true,
    entry_id: 'entry-world-basics',
    keywords: ['世界观', '常识', '背景', '冒险者'],
    title: '世界基调与冒险常识',
  },
  {
    always_include: false,
    content:
      '故事通常从一座新手友好的边境城镇展开。这里有冒险者公会、旅店、杂货铺、铁匠铺和通往野外的主要道路。治安并不完美，但足以维持稳定的委托、补给和情报流通。',
    enabled: true,
    entry_id: 'entry-starter-town',
    keywords: ['城镇', '起始城镇', '旅店', '市场', '边境'],
    title: '起始城镇与日常生活',
  },
  {
    always_include: false,
    content:
      '冒险者公会负责登记身份、发布委托、核验成果并结算报酬。常见任务包括讨伐魔物、护送、采集、调查和跑腿。公会不会替冒险者承担风险，但会尽量避免把明显超纲的任务直接交给新人。',
    enabled: true,
    entry_id: 'entry-guild-rules',
    keywords: ['公会', '委托', '任务', '报酬', '登记'],
    title: '冒险者公会与委托规则',
  },
  {
    always_include: false,
    content:
      '城镇外分布着练手区域、魔物巢穴，以及年代不明的废墟和遗迹。越靠近未开发地带，补给越困难，情报也越不可靠。很多真正推动剧情的变化，都从一次看似普通的调查或护送开始。',
    enabled: true,
    entry_id: 'entry-wilderness-risks',
    keywords: ['野外', '魔物', '遗迹', '废墟', '调查'],
    title: '野外、魔物与遗迹风险',
  },
  {
    always_include: false,
    content:
      '魔法、祝福和特殊技能都真实存在，但成本、熟练度与环境限制同样重要。越强力、越夸张的能力，越需要准备、消耗体力，或者伴随明显后遗症，这使得冲突仍然保有张力。',
    enabled: true,
    entry_id: 'entry-magic-constraints',
    keywords: ['魔法', '祝福', '技能', '消耗', '限制'],
    title: '魔法体系与能力限制',
  },
]

const enDemoLorebookEntries: LorebookEntry[] = [
  {
    always_include: true,
    content:
      'This is a classic sword-and-sorcery adventure world. Most people live around towns, guilds, trade roads, and ruins. Monsters remain a constant threat, but civilization has not been crushed by them. Adventurers are both the people hired to solve trouble and the ones most often standing closest to risk.',
    enabled: true,
    entry_id: 'entry-world-basics',
    keywords: ['world', 'common sense', 'background', 'adventurer'],
    title: 'World Tone and Adventuring Basics',
  },
  {
    always_include: false,
    content:
      'Stories often begin in a beginner-friendly frontier town. It has an adventurers guild, inns, a general store, a blacksmith, and the main road leading into the wilds. Public order is imperfect, but stable enough to keep quests, supplies, and information flowing.',
    enabled: true,
    entry_id: 'entry-starter-town',
    keywords: ['town', 'starter town', 'inn', 'market', 'frontier'],
    title: 'Starter Town and Daily Life',
  },
  {
    always_include: false,
    content:
      'The adventurers guild registers identities, posts quests, verifies results, and settles rewards. Common jobs include monster subjugation, escort duty, gathering, investigation, and errands. The guild does not absorb risk for adventurers, but it still tries not to hand obviously impossible jobs straight to beginners.',
    enabled: true,
    entry_id: 'entry-guild-rules',
    keywords: ['guild', 'quest', 'job', 'reward', 'registration'],
    title: 'Guild Rules and Contracts',
  },
  {
    always_include: false,
    content:
      'Outside town lie training grounds, monster dens, ruins, and relic sites of uncertain age. The farther a party moves toward undeveloped land, the harder resupply becomes and the less reliable information is. Many story-changing turns begin with what looked like a routine escort or investigation.',
    enabled: true,
    entry_id: 'entry-wilderness-risks',
    keywords: ['wilderness', 'monster', 'ruins', 'relics', 'investigation'],
    title: 'Wilderness, Monsters, and Ruin Risk',
  },
  {
    always_include: false,
    content:
      'Magic, blessings, and special skills are all real, but cost, proficiency, and environmental limits matter just as much. The stronger and more spectacular a power is, the more preparation, stamina, or aftermath it usually demands, which keeps conflict tense instead of effortless.',
    enabled: true,
    entry_id: 'entry-magic-constraints',
    keywords: ['magic', 'blessing', 'skill', 'cost', 'limit'],
    title: 'Magic Systems and Power Limits',
  },
]

const zhDemoLorebook: DemoLorebook = {
  displayName: '新手冒险世界设定集',
  entries: zhDemoLorebookEntries,
  lorebookId: 'lorebook-rpg-starter-world',
}

const enDemoLorebook: DemoLorebook = {
  displayName: 'Starter Adventure World Lorebook',
  entries: enDemoLorebookEntries,
  lorebookId: 'lorebook-rpg-starter-world',
}

export function buildDemoLorebook(language?: string): DemoLorebook {
  return isChineseDemoLanguage(language) ? zhDemoLorebook : enDemoLorebook
}
