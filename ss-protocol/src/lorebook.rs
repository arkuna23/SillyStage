use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LorebookEntryPayload {
    pub entry_id: String,
    pub title: String,
    pub content: String,
    pub keywords: Vec<String>,
    pub enabled: bool,
    pub always_include: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LorebookCreateParams {
    pub lorebook_id: String,
    pub display_name: String,
    #[serde(default)]
    pub entries: Vec<LorebookEntryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LorebookGetParams {
    pub lorebook_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LorebookUpdateParams {
    pub lorebook_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct LorebookListParams {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LorebookDeleteParams {
    pub lorebook_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LorebookEntryCreateParams {
    pub lorebook_id: String,
    pub entry_id: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default)]
    pub always_include: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LorebookEntryGetParams {
    pub lorebook_id: String,
    pub entry_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LorebookEntryListParams {
    pub lorebook_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LorebookEntryUpdateParams {
    pub lorebook_id: String,
    pub entry_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub always_include: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LorebookEntryDeleteParams {
    pub lorebook_id: String,
    pub entry_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LorebookPayload {
    pub lorebook_id: String,
    pub display_name: String,
    pub entries: Vec<LorebookEntryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LorebooksListedPayload {
    pub lorebooks: Vec<LorebookPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LorebookDeletedPayload {
    pub lorebook_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LorebookEntriesListedPayload {
    pub lorebook_id: String,
    pub entries: Vec<LorebookEntryPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LorebookEntryDeletedPayload {
    pub lorebook_id: String,
    pub entry_id: String,
}

fn default_enabled() -> bool {
    true
}
