use agents::actor::ActorResponse;
use agents::director::{ActorPurpose, DirectorResult, NarratorPurpose};
use agents::keeper::KeeperPhase;
use agents::narrator::NarratorResponse;
use serde::{Deserialize, Serialize};
use state::ActorMemoryEntry;
use store::SessionCharacterRecord;

use crate::{EngineTurnResult, RuntimeSnapshot};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EngineEvent {
    TurnStarted {
        next_turn_index: u64,
        player_input: String,
    },
    PlayerInputRecorded {
        entry: ActorMemoryEntry,
        snapshot: Box<RuntimeSnapshot>,
    },
    KeeperApplied {
        phase: KeeperPhase,
        update: state::StateUpdate,
        snapshot: Box<RuntimeSnapshot>,
    },
    DirectorCompleted {
        result: DirectorResult,
        snapshot: Box<RuntimeSnapshot>,
    },
    SessionCharacterCreated {
        character: SessionCharacterRecord,
        snapshot: Box<RuntimeSnapshot>,
    },
    SessionCharacterEnteredScene {
        session_character_id: String,
        snapshot: Box<RuntimeSnapshot>,
    },
    SessionCharacterLeftScene {
        session_character_id: String,
        snapshot: Box<RuntimeSnapshot>,
    },
    NarratorStarted {
        beat_index: usize,
        purpose: NarratorPurpose,
    },
    NarratorTextDelta {
        beat_index: usize,
        purpose: NarratorPurpose,
        delta: String,
    },
    NarratorCompleted {
        beat_index: usize,
        purpose: NarratorPurpose,
        response: Box<NarratorResponse>,
    },
    ActorStarted {
        beat_index: usize,
        speaker_id: String,
        purpose: ActorPurpose,
    },
    ActorThoughtDelta {
        beat_index: usize,
        speaker_id: String,
        delta: String,
    },
    ActorActionComplete {
        beat_index: usize,
        speaker_id: String,
        text: String,
    },
    ActorDialogueDelta {
        beat_index: usize,
        speaker_id: String,
        delta: String,
    },
    ActorCompleted {
        beat_index: usize,
        speaker_id: String,
        purpose: ActorPurpose,
        response: Box<ActorResponse>,
    },
    TurnCompleted {
        result: Box<EngineTurnResult>,
    },
    TurnFailed {
        stage: EngineStage,
        error: String,
        snapshot: Box<RuntimeSnapshot>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EngineStage {
    RecordPlayerInput,
    KeeperAfterPlayerInput,
    Director,
    Narrator,
    Actor,
    KeeperAfterTurnOutputs,
}
