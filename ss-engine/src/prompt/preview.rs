use std::collections::BTreeSet;

use store::PromptMessageRole;

use super::types::{
    CompiledPromptPreviewEntry, CompiledPromptPreviewEntryValue, CompiledPromptPreviewModule,
    CompiledPromptPreviewProfile, PromptPreview, PromptPreviewEntry, PromptPreviewMessage,
    PromptPreviewMessageRole, PromptPreviewModule,
};

pub(crate) fn render_profile_preview<F>(
    profile: &CompiledPromptPreviewProfile,
    include_placeholders: bool,
    resolve: F,
) -> PromptPreview
where
    F: Fn(&str) -> Option<String>,
{
    let (system_modules, system_unresolved) =
        render_modules(&profile.system_modules, include_placeholders, &resolve);
    let (user_modules, user_unresolved) =
        render_modules(&profile.user_modules, include_placeholders, &resolve);

    let mut unresolved = system_unresolved;
    unresolved.extend(user_unresolved);

    let mut messages = Vec::new();
    if !system_modules.is_empty() {
        messages.push(PromptPreviewMessage {
            role: PromptMessageRole::System,
            modules: system_modules,
        });
    }
    if !user_modules.is_empty() {
        messages.push(PromptPreviewMessage {
            role: PromptMessageRole::User,
            modules: user_modules,
        });
    }

    PromptPreview {
        message_role: PromptPreviewMessageRole::Full,
        messages,
        unresolved_context_keys: unresolved.into_iter().collect(),
    }
}

pub(crate) fn render_module_preview<F>(
    message_role: PromptMessageRole,
    module: Option<&CompiledPromptPreviewModule>,
    include_placeholders: bool,
    resolve: F,
) -> PromptPreview
where
    F: Fn(&str) -> Option<String>,
{
    let (module, unresolved_context_keys) = match module {
        Some(module) => render_module(module, include_placeholders, &resolve),
        None => (None, BTreeSet::new()),
    };

    let mut messages = Vec::new();
    if let Some(module) = module {
        messages.push(PromptPreviewMessage {
            role: message_role,
            modules: vec![module],
        });
    }

    PromptPreview {
        message_role: match message_role {
            PromptMessageRole::System => PromptPreviewMessageRole::System,
            PromptMessageRole::User => PromptPreviewMessageRole::User,
        },
        messages,
        unresolved_context_keys: unresolved_context_keys.into_iter().collect(),
    }
}

fn render_modules<F>(
    modules: &[CompiledPromptPreviewModule],
    include_placeholders: bool,
    resolve: &F,
) -> (Vec<PromptPreviewModule>, BTreeSet<String>)
where
    F: Fn(&str) -> Option<String>,
{
    let mut unresolved = BTreeSet::new();
    let rendered = modules
        .iter()
        .filter_map(|module| {
            let (module, module_unresolved) = render_module(module, include_placeholders, resolve);
            unresolved.extend(module_unresolved);
            module
        })
        .collect();

    (rendered, unresolved)
}

fn render_module<F>(
    module: &CompiledPromptPreviewModule,
    include_placeholders: bool,
    resolve: &F,
) -> (Option<PromptPreviewModule>, BTreeSet<String>)
where
    F: Fn(&str) -> Option<String>,
{
    let mut unresolved = BTreeSet::new();
    let entries = module
        .entries
        .iter()
        .filter_map(|entry| {
            let (entry, entry_unresolved) = render_entry(entry, include_placeholders, resolve);
            unresolved.extend(entry_unresolved);
            entry
        })
        .collect::<Vec<_>>();

    if entries.is_empty() {
        (None, unresolved)
    } else {
        (
            Some(PromptPreviewModule {
                module_id: module.module_id.clone(),
                display_name: module.display_name.clone(),
                order: module.order,
                entries,
            }),
            unresolved,
        )
    }
}

fn render_entry<F>(
    entry: &CompiledPromptPreviewEntry,
    include_placeholders: bool,
    resolve: &F,
) -> (Option<PromptPreviewEntry>, BTreeSet<String>)
where
    F: Fn(&str) -> Option<String>,
{
    let mut unresolved = BTreeSet::new();
    let compiled_text = match &entry.value {
        CompiledPromptPreviewEntryValue::Text(text) => {
            let text = text.trim();
            (!text.is_empty()).then(|| text.to_owned())
        }
        CompiledPromptPreviewEntryValue::ContextRef(key) => match resolve(key) {
            Some(value) => {
                let value = value.trim();
                (!value.is_empty()).then(|| value.to_owned())
            }
            None => {
                unresolved.insert(key.clone());
                include_placeholders.then(|| format!("<context:{key}>"))
            }
        },
    };

    (
        compiled_text.map(|compiled_text| PromptPreviewEntry {
            entry_id: entry.entry_id.clone(),
            display_name: entry.display_name.clone(),
            kind: entry.kind,
            order: entry.order,
            source: entry.source,
            compiled_text,
        }),
        unresolved,
    )
}
