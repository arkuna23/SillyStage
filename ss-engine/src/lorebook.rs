use store::LorebookEntryRecord;

const MAX_BASE_ENTRIES: usize = 4;
const MAX_MATCHED_ENTRIES: usize = 8;
const MAX_ENTRY_CONTENT_CHARS: usize = 800;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct LorebookPromptSections {
    pub(crate) base: Option<String>,
    pub(crate) matched: Option<String>,
}

pub(crate) fn build_lorebook_prompt_sections(
    entries: &[LorebookEntryRecord],
    match_inputs: &[&str],
) -> LorebookPromptSections {
    let haystack = match_inputs
        .iter()
        .filter(|text| !text.trim().is_empty())
        .map(|text| text.to_lowercase())
        .collect::<Vec<_>>()
        .join("\n");

    let mut base_entries = Vec::new();
    let mut matched_entries = Vec::new();

    for entry in entries.iter().filter(|entry| entry.enabled) {
        if entry.always_include {
            if base_entries.len() < MAX_BASE_ENTRIES {
                base_entries.push(entry);
            }
            continue;
        }

        if matched_entries.len() >= MAX_MATCHED_ENTRIES {
            continue;
        }

        let matched = !haystack.is_empty()
            && entry
                .keywords
                .iter()
                .map(|keyword| keyword.trim())
                .filter(|keyword| !keyword.is_empty())
                .any(|keyword| haystack.contains(&keyword.to_lowercase()));
        if matched {
            matched_entries.push(entry);
        }
    }

    LorebookPromptSections {
        base: render_lorebook_entries(&base_entries),
        matched: render_lorebook_entries(&matched_entries),
    }
}

fn render_lorebook_entries(entries: &[&LorebookEntryRecord]) -> Option<String> {
    if entries.is_empty() {
        return None;
    }

    Some(
        entries
            .iter()
            .map(|entry| {
                let keywords = if entry.keywords.is_empty() {
                    "none".to_owned()
                } else {
                    entry
                        .keywords
                        .iter()
                        .map(|keyword| normalize_inline_text(keyword))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                format!(
                    "- {} | {} | keywords={} | {}",
                    entry.entry_id,
                    normalize_inline_text(&entry.title),
                    keywords,
                    truncate_chars(
                        &normalize_inline_text(&entry.content),
                        MAX_ENTRY_CONTENT_CHARS
                    )
                )
            })
            .collect::<Vec<_>>()
            .join("\n"),
    )
}

fn normalize_inline_text(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }

    let mut output = String::new();
    for ch in text.chars().take(max_chars) {
        output.push(ch);
    }
    output.push_str("...");
    output
}

#[cfg(test)]
mod tests {
    use super::build_lorebook_prompt_sections;
    use store::LorebookEntryRecord;

    fn entry(
        entry_id: &str,
        keywords: &[&str],
        enabled: bool,
        always_include: bool,
    ) -> LorebookEntryRecord {
        LorebookEntryRecord {
            entry_id: entry_id.to_owned(),
            title: format!("Title {entry_id}"),
            content: format!("Content for {entry_id}"),
            keywords: keywords
                .iter()
                .map(|keyword| (*keyword).to_owned())
                .collect(),
            enabled,
            always_include,
        }
    }

    #[test]
    fn splits_base_and_matched_entries() {
        let sections = build_lorebook_prompt_sections(
            &[
                entry("base", &[], true, true),
                entry("match", &["merchant"], true, false),
                entry("disabled", &["merchant"], false, false),
            ],
            &["merchant at the dock"],
        );

        assert!(
            sections
                .base
                .as_deref()
                .is_some_and(|text| text.contains("base"))
        );
        assert!(
            sections
                .matched
                .as_deref()
                .is_some_and(|text| text.contains("match"))
        );
        assert!(
            sections
                .matched
                .as_deref()
                .is_none_or(|text| !text.contains("disabled"))
        );
    }
}
