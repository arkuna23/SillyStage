import type { TFunction } from 'i18next'

import type { StateFieldSchema } from '../../lib/state-schema'

export type SchemaPresetKind = 'actor' | 'player' | 'world'

export type SchemaPresetDefinition = {
  description: string
  displayName: string
  fields: Record<string, StateFieldSchema>
  kind: SchemaPresetKind
  schemaId: string
  tags: string[]
}

function buildPlayerFields(t: TFunction): Record<string, StateFieldSchema> {
  return {
    current_location: {
      default: '',
      description: t('schemas.presets.player.fields.currentLocation'),
      value_type: 'string',
    },
    gold: {
      default: 0,
      description: t('schemas.presets.player.fields.gold'),
      value_type: 'int',
    },
    hp: {
      default: 100,
      description: t('schemas.presets.player.fields.hp'),
      value_type: 'int',
    },
    level: {
      default: 1,
      description: t('schemas.presets.player.fields.level'),
      value_type: 'int',
    },
    max_hp: {
      default: 100,
      description: t('schemas.presets.player.fields.maxHp'),
      value_type: 'int',
    },
    mp: {
      default: 30,
      description: t('schemas.presets.player.fields.mp'),
      value_type: 'int',
    },
  }
}

function buildWorldFields(t: TFunction): Record<string, StateFieldSchema> {
  return {
    current_event: {
      default: '',
      description: t('schemas.presets.world.fields.currentEvent'),
      value_type: 'string',
    },
    danger_level: {
      default: 0,
      description: t('schemas.presets.world.fields.dangerLevel'),
      value_type: 'int',
    },
    region: {
      default: '',
      description: t('schemas.presets.world.fields.region'),
      value_type: 'string',
    },
    time_of_day: {
      default: 'day',
      description: t('schemas.presets.world.fields.timeOfDay'),
      value_type: 'string',
    },
    weather: {
      default: 'clear',
      description: t('schemas.presets.world.fields.weather'),
      value_type: 'string',
    },
  }
}

function buildActorFields(t: TFunction): Record<string, StateFieldSchema> {
  return {
    affinity: {
      default: 0,
      description: t('schemas.presets.actor.fields.affinity'),
      value_type: 'int',
    },
    goal: {
      default: '',
      description: t('schemas.presets.actor.fields.goal'),
      value_type: 'string',
    },
    mood: {
      default: 'neutral',
      description: t('schemas.presets.actor.fields.mood'),
      value_type: 'string',
    },
    status_effects: {
      default: [],
      description: t('schemas.presets.actor.fields.statusEffects'),
      value_type: 'array',
    },
    trust: {
      default: 0,
      description: t('schemas.presets.actor.fields.trust'),
      value_type: 'int',
    },
  }
}

export function buildSchemaPresetDefinitions(t: TFunction): SchemaPresetDefinition[] {
  return [
    {
      description: t('schemas.presets.player.description'),
      displayName: t('schemas.presets.player.title'),
      fields: buildPlayerFields(t),
      kind: 'player',
      schemaId: 'schema-rpg-player-basic',
      tags: ['player', 'rpg', 'starter'],
    },
    {
      description: t('schemas.presets.world.description'),
      displayName: t('schemas.presets.world.title'),
      fields: buildWorldFields(t),
      kind: 'world',
      schemaId: 'schema-rpg-world-basic',
      tags: ['world', 'rpg', 'starter'],
    },
    {
      description: t('schemas.presets.actor.description'),
      displayName: t('schemas.presets.actor.title'),
      fields: buildActorFields(t),
      kind: 'actor',
      schemaId: 'schema-rpg-actor-basic',
      tags: ['character', 'actor', 'rpg', 'starter'],
    },
  ]
}
