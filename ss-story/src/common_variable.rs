use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use state::StateFieldSchema;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommonVariableScope {
    World,
    Player,
    Character,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CommonVariableDefinition {
    pub scope: CommonVariableScope,
    pub key: String,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub character_id: Option<String>,
    #[serde(default = "default_pinned")]
    pub pinned: bool,
}

const fn default_pinned() -> bool {
    true
}

pub fn validate_common_variables(
    common_variables: &[CommonVariableDefinition],
    resource_character_ids: &[String],
    world_fields: &HashMap<String, StateFieldSchema>,
    player_fields: &HashMap<String, StateFieldSchema>,
    character_fields: &HashMap<String, HashMap<String, StateFieldSchema>>,
) -> Result<(), String> {
    let mut seen = HashSet::new();

    for variable in common_variables {
        if variable.key.trim().is_empty() {
            return Err("key must not be empty".to_owned());
        }
        if variable.display_name.trim().is_empty() {
            return Err(format!(
                "display_name must not be empty for key '{}'",
                variable.key
            ));
        }

        let unique_key = match variable.scope {
            CommonVariableScope::World => {
                if variable.character_id.is_some() {
                    return Err(format!(
                        "world variable '{}' must not set character_id",
                        variable.key
                    ));
                }
                ensure_schema_field_exists(world_fields, &variable.key, "world", None)?;
                format!("world:{}", variable.key)
            }
            CommonVariableScope::Player => {
                if variable.character_id.is_some() {
                    return Err(format!(
                        "player variable '{}' must not set character_id",
                        variable.key
                    ));
                }
                ensure_schema_field_exists(player_fields, &variable.key, "player", None)?;
                format!("player:{}", variable.key)
            }
            CommonVariableScope::Character => {
                let character_id = variable.character_id.as_deref().ok_or_else(|| {
                    format!(
                        "character variable '{}' must set character_id",
                        variable.key
                    )
                })?;
                if !resource_character_ids.iter().any(|id| id == character_id) {
                    return Err(format!(
                        "character variable '{}.{}' references a character not used by this story",
                        character_id, variable.key
                    ));
                }

                let fields = character_fields.get(character_id).ok_or_else(|| {
                    format!(
                        "character variable '{}.{}' references a character schema that is unavailable",
                        character_id, variable.key
                    )
                })?;
                ensure_schema_field_exists(fields, &variable.key, "character", Some(character_id))?;
                format!("character:{character_id}:{}", variable.key)
            }
        };

        if !seen.insert(unique_key) {
            return Err(format!(
                "duplicate common variable for key '{}'",
                variable.key
            ));
        }
    }

    Ok(())
}

fn ensure_schema_field_exists(
    fields: &HashMap<String, StateFieldSchema>,
    key: &str,
    scope: &str,
    character_id: Option<&str>,
) -> Result<(), String> {
    if fields.contains_key(key) {
        return Ok(());
    }

    let subject = character_id
        .map(|character_id| format!("{scope} variable '{character_id}.{key}'"))
        .unwrap_or_else(|| format!("{scope} variable '{key}'"));
    Err(format!("{subject} does not exist in the bound schema"))
}
