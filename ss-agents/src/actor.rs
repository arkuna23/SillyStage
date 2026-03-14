use std::collections::{HashMap, VecDeque};
use std::fs;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;

use futures_core::Stream;
use futures_util::{StreamExt, stream};
use llm::{ChatRequest, LlmApi};
use serde::{Deserialize, Serialize};
use state::schema::StateFieldSchema;
use tracing::error;

use crate::director::ActorPurpose;
use state::{ActorMemoryEntry, ActorMemoryKind, WorldState};
use story::NarrativeNode;

const DEFAULT_MEMORY_LIMIT: usize = 8;

pub type ActorEventStream<'a> =
    Pin<Box<dyn Stream<Item = Result<ActorStreamEvent, ActorError>> + Send + 'a>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCard {
    pub id: String,
    pub name: String,
    pub personality: String,
    pub style: String,
    pub tendencies: Vec<String>,
    #[serde(default)]
    pub state_schema: HashMap<String, StateFieldSchema>,
    pub system_prompt: String,
}

impl CharacterCard {
    pub fn summary(&self) -> CharacterCardSummary {
        CharacterCardSummary {
            id: self.id.clone(),
            name: self.name.clone(),
            personality: self.personality.clone(),
            style: self.style.clone(),
            tendencies: self.tendencies.clone(),
            state_schema: self.state_schema.clone(),
        }
    }

    pub(crate) fn summary_ref(&self) -> CharacterCardSummaryRef<'_> {
        CharacterCardSummaryRef {
            id: &self.id,
            name: &self.name,
            personality: &self.personality,
            style: &self.style,
            tendencies: &self.tendencies,
            state_schema: &self.state_schema,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterCardSummary {
    pub id: String,
    pub name: String,
    pub personality: String,
    pub style: String,
    pub tendencies: Vec<String>,
    #[serde(default)]
    pub state_schema: HashMap<String, StateFieldSchema>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CharacterCardSummaryRef<'a> {
    id: &'a str,
    name: &'a str,
    personality: &'a str,
    style: &'a str,
    tendencies: &'a [String],
    state_schema: &'a HashMap<String, StateFieldSchema>,
}

#[derive(Debug, Clone)]
pub struct ActorRequest<'a> {
    pub character: &'a CharacterCard,
    pub cast: &'a [CharacterCard],
    pub player_description: &'a str,
    pub purpose: ActorPurpose,
    pub node: &'a NarrativeNode,
    pub memory_limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorResponse {
    pub speaker_id: String,
    pub speaker_name: String,
    pub segments: Vec<ActorSegment>,
    pub raw_output: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ActorSegment {
    pub kind: ActorSegmentKind,
    pub text: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActorSegmentKind {
    Dialogue,
    Thought,
    Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ActorStreamEvent {
    DialogueDelta { delta: String },
    ThoughtDelta { delta: String },
    ActionComplete { text: String },
    Done { response: ActorResponse },
}

pub struct Actor {
    llm: Arc<dyn LlmApi>,
    model: String,
    system_prompt: String,
    temperature: Option<f32>,
    max_tokens: Option<u32>,
}

impl Actor {
    pub fn new(llm: Arc<dyn LlmApi>, model: impl Into<String>) -> Result<Self, ActorError> {
        Self::new_with_options(llm, model, None, None)
    }

    pub fn new_with_options(
        llm: Arc<dyn LlmApi>,
        model: impl Into<String>,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
    ) -> Result<Self, ActorError> {
        Ok(Self {
            llm,
            model: model.into(),
            system_prompt: include_str!("./prompts/actor.txt").to_owned(),
            temperature,
            max_tokens,
        })
    }

    pub fn from_prompt_file(
        llm: Arc<dyn LlmApi>,
        model: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<Self, ActorError> {
        let system_prompt = fs::read_to_string(path).map_err(ActorError::ReadPrompt)?;
        Ok(Self {
            llm,
            model: model.into(),
            system_prompt,
            temperature: None,
            max_tokens: None,
        })
    }

    pub async fn perform(
        &self,
        request: ActorRequest<'_>,
        world_state: &mut WorldState,
    ) -> Result<ActorResponse, ActorError> {
        let mut stream = self.perform_stream(request, world_state).await?;
        let mut final_response = None;

        while let Some(event) = stream.next().await {
            if let ActorStreamEvent::Done { response } = event? {
                final_response = Some(response);
            }
        }

        final_response.ok_or_else(|| {
            ActorError::StreamParse("actor stream finished without a final response".to_owned())
        })
    }

    pub async fn perform_stream<'b>(
        &'b self,
        request: ActorRequest<'_>,
        world_state: &'b mut WorldState,
    ) -> Result<ActorEventStream<'b>, ActorError> {
        Self::validate_request(&request)?;

        let character_prompt = self.build_character_prompt(request.character)?;
        let user_prompt = self.build_user_prompt(&request, world_state)?;
        let stream = self
            .llm
            .chat_stream({
                let mut builder = ChatRequest::builder()
                    .model(&self.model)
                    .system_message(&self.system_prompt)
                    .system_message(character_prompt)
                    .user_message(user_prompt);
                if let Some(temperature) = self.temperature {
                    builder = builder.temperature(temperature);
                }
                if let Some(max_tokens) = self.max_tokens {
                    builder = builder.max_tokens(max_tokens);
                }
                builder.build()?
            })
            .await?;

        let state = ActorEventStreamState {
            llm_stream: stream,
            parser: ActorStreamParser::new(request.character),
            world_state,
            memory_limit: request.memory_limit.unwrap_or(DEFAULT_MEMORY_LIMIT),
            llm_finished: false,
            memory_persisted: false,
            terminated: false,
        };

        let stream = stream::unfold(state, |mut state| async move {
            if state.terminated {
                return None;
            }

            loop {
                if let Some(event) = state.parser.pop_event() {
                    if let Err(error) = state.persist_memory_if_needed(event.clone()) {
                        state.terminated = true;
                        return Some((Err(error), state));
                    }
                    return Some((Ok(event), state));
                }

                if state.llm_finished {
                    return None;
                }

                match state.llm_stream.next().await {
                    Some(Ok(chunk)) => {
                        if let Err(error) = state.parser.ingest(&chunk.delta) {
                            state.parser.log_stream_parse_error("ingest", &error);
                            state.terminated = true;
                            return Some((Err(error), state));
                        }

                        if chunk.done {
                            if let Err(error) = state.parser.finish() {
                                state
                                    .parser
                                    .log_stream_parse_error("finish_after_done", &error);
                                state.terminated = true;
                                return Some((Err(error), state));
                            }
                            state.llm_finished = true;
                        }
                    }
                    Some(Err(error)) => {
                        state.terminated = true;
                        return Some((Err(ActorError::Llm(error)), state));
                    }
                    None => {
                        if let Err(error) = state.parser.finish() {
                            state
                                .parser
                                .log_stream_parse_error("finish_on_stream_end", &error);
                            state.terminated = true;
                            return Some((Err(error), state));
                        }
                        state.llm_finished = true;
                    }
                }
            }
        });

        Ok(Box::pin(stream))
    }

    fn validate_request(request: &ActorRequest<'_>) -> Result<(), ActorError> {
        if !request
            .node
            .characters
            .iter()
            .any(|id| id == &request.character.id)
        {
            return Err(ActorError::InvalidRequest(format!(
                "current node does not contain character id '{}'",
                request.character.id
            )));
        }

        let cast_by_id: HashMap<&str, &CharacterCard> = request
            .cast
            .iter()
            .map(|card| (card.id.as_str(), card))
            .collect();

        for character_id in &request.node.characters {
            if !cast_by_id.contains_key(character_id.as_str()) {
                return Err(ActorError::InvalidRequest(format!(
                    "missing character card for current cast id '{character_id}'"
                )));
            }
        }

        Ok(())
    }

    fn build_character_prompt(&self, character: &CharacterCard) -> Result<String, ActorError> {
        let card_json =
            serde_json::to_string_pretty(character).map_err(ActorError::SerializePromptData)?;

        Ok(format!("CHARACTER_CARD:\n{card_json}"))
    }

    fn build_user_prompt(
        &self,
        request: &ActorRequest<'_>,
        world_state: &WorldState,
    ) -> Result<String, ActorError> {
        let purpose_json =
            serde_json::to_string(&request.purpose).map_err(ActorError::SerializePromptData)?;
        let node_json =
            serde_json::to_string_pretty(&request.node).map_err(ActorError::SerializePromptData)?;
        let world_state_json = serde_json::to_string_pretty(&world_state.actor_prompt_view())
            .map_err(ActorError::SerializePromptData)?;
        let memory_limit = request.memory_limit.unwrap_or(DEFAULT_MEMORY_LIMIT);
        let shared_history_json =
            serde_json::to_string_pretty(&world_state.recent_actor_shared_history(memory_limit))
                .map_err(ActorError::SerializePromptData)?;
        let private_memory_json = serde_json::to_string_pretty(
            &world_state.recent_actor_private_memory(&request.character.id, memory_limit),
        )
        .map_err(ActorError::SerializePromptData)?;
        let cast_json = serde_json::to_string_pretty(&self.current_cast_summaries(request)?)
            .map_err(ActorError::SerializePromptData)?;

        Ok(format!(
            "ACTOR_PURPOSE:\n{}\n\nPLAYER_DESCRIPTION:\n{}\n\nCURRENT_CAST:\n{}\n\nCURRENT_NODE:\n{}\n\nWORLD_STATE:\n{}\n\nSHARED_SCENE_HISTORY:\n{}\n\nPRIVATE_CHARACTER_MEMORY:\n{}",
            purpose_json,
            request.player_description,
            cast_json,
            node_json,
            world_state_json,
            shared_history_json,
            private_memory_json
        ))
    }

    fn current_cast_summaries<'b>(
        &self,
        request: &ActorRequest<'b>,
    ) -> Result<Vec<CharacterCardSummaryRef<'b>>, ActorError> {
        let cast_by_id: HashMap<&str, &CharacterCard> = request
            .cast
            .iter()
            .map(|card| (card.id.as_str(), card))
            .collect();

        request
            .node
            .characters
            .iter()
            .map(|character_id| {
                cast_by_id
                    .get(character_id.as_str())
                    .map(|card| card.summary_ref())
                    .ok_or_else(|| {
                        ActorError::InvalidRequest(format!(
                            "missing character card for current cast id '{character_id}'"
                        ))
                    })
            })
            .collect()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ActorError {
    #[error("{0}")]
    InvalidRequest(String),
    #[error(transparent)]
    ReadPrompt(std::io::Error),
    #[error(transparent)]
    SerializePromptData(serde_json::Error),
    #[error(transparent)]
    Llm(#[from] llm::LlmError),
    #[error("stream parse error: {0}")]
    StreamParse(String),
}

struct ActorEventStreamState<'a> {
    llm_stream: llm::ChatStream,
    parser: ActorStreamParser,
    world_state: &'a mut WorldState,
    memory_limit: usize,
    llm_finished: bool,
    memory_persisted: bool,
    terminated: bool,
}

impl ActorEventStreamState<'_> {
    fn persist_memory_if_needed(&mut self, event: ActorStreamEvent) -> Result<(), ActorError> {
        if self.memory_persisted {
            return Ok(());
        }

        let ActorStreamEvent::Done { response } = event else {
            return Ok(());
        };

        persist_actor_memory(self.world_state, &response, self.memory_limit);
        self.memory_persisted = true;
        Ok(())
    }
}

struct ActorStreamParser {
    speaker_id: String,
    speaker_name: String,
    pending: String,
    state: ParserState,
    completed_segments: Vec<ActorSegment>,
    queued_events: VecDeque<ActorStreamEvent>,
    raw_output: String,
    finished: bool,
}

fn persist_actor_memory(
    world_state: &mut WorldState,
    response: &ActorResponse,
    memory_limit: usize,
) {
    for segment in &response.segments {
        let entry = ActorMemoryEntry {
            speaker_id: response.speaker_id.clone(),
            speaker_name: response.speaker_name.clone(),
            kind: memory_kind(segment.kind),
            text: segment.text.clone(),
        };

        if matches!(
            segment.kind,
            ActorSegmentKind::Dialogue | ActorSegmentKind::Action
        ) {
            world_state.push_actor_shared_history(entry, memory_limit);
        } else if matches!(segment.kind, ActorSegmentKind::Thought) {
            world_state.push_actor_private_memory(response.speaker_id.clone(), entry, memory_limit);
        }
    }
}

fn memory_kind(kind: ActorSegmentKind) -> ActorMemoryKind {
    match kind {
        ActorSegmentKind::Dialogue => ActorMemoryKind::Dialogue,
        ActorSegmentKind::Thought => ActorMemoryKind::Thought,
        ActorSegmentKind::Action => ActorMemoryKind::Action,
    }
}

enum ParserState {
    SeekingTag,
    InSegment {
        kind: ActorSegmentKind,
        text: String,
    },
}

impl ActorStreamParser {
    fn new(character: &CharacterCard) -> Self {
        Self {
            speaker_id: character.id.clone(),
            speaker_name: character.name.clone(),
            pending: String::new(),
            state: ParserState::SeekingTag,
            completed_segments: Vec::new(),
            queued_events: VecDeque::new(),
            raw_output: String::new(),
            finished: false,
        }
    }

    fn pop_event(&mut self) -> Option<ActorStreamEvent> {
        self.queued_events.pop_front()
    }

    fn log_stream_parse_error(&self, context: &str, error: &ActorError) {
        if !matches!(error, ActorError::StreamParse(_)) {
            return;
        }

        error!(
            speaker_id = %self.speaker_id,
            speaker_name = %self.speaker_name,
            context,
            error = %error,
            raw_output = %self.raw_output,
            completed_segments = %self.completed_segments_for_log(),
            pending = %self.pending,
            "actor stream parse failed"
        );
    }

    fn completed_segments_for_log(&self) -> String {
        serde_json::to_string_pretty(&self.completed_segments)
            .unwrap_or_else(|error| format!("{{\"serialization_error\":\"{error}\"}}"))
    }

    fn ingest(&mut self, delta: &str) -> Result<(), ActorError> {
        if self.finished {
            return Err(ActorError::StreamParse(
                "received additional content after stream completion".to_owned(),
            ));
        }

        if delta.is_empty() {
            return Ok(());
        }

        self.raw_output.push_str(delta);
        self.pending.push_str(delta);
        self.process_pending(false)
    }

    fn finish(&mut self) -> Result<(), ActorError> {
        if self.finished {
            return Ok(());
        }

        self.process_pending(true)?;

        if !matches!(self.state, ParserState::SeekingTag) {
            return Err(ActorError::StreamParse(
                "actor output ended before closing the current segment".to_owned(),
            ));
        }

        if !self.pending.trim().is_empty() {
            return Err(ActorError::StreamParse(
                "actor output contained stray text outside segment tags".to_owned(),
            ));
        }

        if self.completed_segments.is_empty() {
            return Err(ActorError::StreamParse(
                "actor output contained no segments".to_owned(),
            ));
        }

        self.finished = true;
        self.queued_events.push_back(ActorStreamEvent::Done {
            response: ActorResponse {
                speaker_id: self.speaker_id.clone(),
                speaker_name: self.speaker_name.clone(),
                segments: self.completed_segments.clone(),
                raw_output: self.raw_output.clone(),
            },
        });
        Ok(())
    }

    fn process_pending(&mut self, is_final: bool) -> Result<(), ActorError> {
        loop {
            match &mut self.state {
                ParserState::SeekingTag => {
                    let trimmed_len = self.pending.len() - self.pending.trim_start().len();
                    if trimmed_len > 0 {
                        self.pending.drain(..trimmed_len);
                    }

                    if self.pending.is_empty() {
                        return Ok(());
                    }

                    if !self.pending.starts_with('<') {
                        return Err(ActorError::StreamParse(
                            "actor output contained text outside segment tags".to_owned(),
                        ));
                    }

                    let Some(close_index) = self.pending.find('>') else {
                        if is_final {
                            return Err(ActorError::StreamParse(
                                "actor output ended in the middle of an opening tag".to_owned(),
                            ));
                        }
                        return Ok(());
                    };

                    let tag_name = self.pending[1..close_index].trim();
                    let kind = ActorSegmentKind::from_opening_tag(tag_name)?;
                    self.pending.drain(..close_index + 1);
                    self.state = ParserState::InSegment {
                        kind,
                        text: String::new(),
                    };
                }
                ParserState::InSegment { kind, text } => {
                    let closing_tag = kind.closing_tag();

                    if let Some(index) = self.pending.find(closing_tag) {
                        let chunk = self.pending[..index].to_owned();
                        if let Some(event) = append_segment_text(*kind, text, &chunk) {
                            self.queued_events.push_back(event);
                        }
                        self.pending.drain(..index + closing_tag.len());

                        let completed = ActorSegment {
                            kind: *kind,
                            text: std::mem::take(text),
                        };
                        if completed.text.trim().is_empty() {
                            return Err(ActorError::StreamParse(format!(
                                "{} segment was empty",
                                kind.label()
                            )));
                        }

                        if matches!(completed.kind, ActorSegmentKind::Action) {
                            self.queued_events
                                .push_back(ActorStreamEvent::ActionComplete {
                                    text: completed.text.clone(),
                                });
                        }
                        self.completed_segments.push(completed);
                        self.state = ParserState::SeekingTag;
                        continue;
                    }

                    if is_final {
                        return Err(ActorError::StreamParse(format!(
                            "actor output ended before closing {} segment",
                            kind.label()
                        )));
                    }

                    let safe_len = self
                        .pending
                        .len()
                        .saturating_sub(retained_suffix_len(&self.pending, closing_tag));
                    if safe_len == 0 {
                        return Ok(());
                    }

                    let safe_text = self.pending[..safe_len].to_owned();
                    self.pending.drain(..safe_len);
                    if let Some(event) = append_segment_text(*kind, text, &safe_text) {
                        self.queued_events.push_back(event);
                    }
                    return Ok(());
                }
            }
        }
    }
}

fn append_segment_text(
    kind: ActorSegmentKind,
    text: &mut String,
    chunk: &str,
) -> Option<ActorStreamEvent> {
    if chunk.is_empty() {
        return None;
    }

    text.push_str(chunk);
    match kind {
        ActorSegmentKind::Dialogue => Some(ActorStreamEvent::DialogueDelta {
            delta: chunk.to_owned(),
        }),
        ActorSegmentKind::Thought => Some(ActorStreamEvent::ThoughtDelta {
            delta: chunk.to_owned(),
        }),
        ActorSegmentKind::Action => None,
    }
}

fn retained_suffix_len(pending: &str, closing_tag: &str) -> usize {
    let max_len = pending.len().min(closing_tag.len().saturating_sub(1));
    for len in (1..=max_len).rev() {
        if pending.ends_with(&closing_tag[..len]) {
            return len;
        }
    }

    0
}

impl ActorSegmentKind {
    fn from_opening_tag(tag: &str) -> Result<Self, ActorError> {
        match tag {
            "dialogue" => Ok(Self::Dialogue),
            "thought" => Ok(Self::Thought),
            "action" => Ok(Self::Action),
            _ => Err(ActorError::StreamParse(format!(
                "unknown actor segment tag '{tag}'"
            ))),
        }
    }

    fn closing_tag(&self) -> &'static str {
        match self {
            Self::Dialogue => "</dialogue>",
            Self::Thought => "</thought>",
            Self::Action => "</action>",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Self::Dialogue => "dialogue",
            Self::Thought => "thought",
            Self::Action => "action",
        }
    }
}
