use agents::actor::ActorResponse;
use agents::director::{ActorPurpose, DirectorResult, NarratorPurpose};
use agents::keeper::KeeperPhase;
use agents::narrator::NarratorResponse;
use engine::RuntimeSnapshot;
use serde::{Deserialize, Serialize};
use state::{ActorMemoryEntry, StateUpdate};

use crate::session_character::SessionCharacterPayload;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum StreamEventBody {
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
        update: StateUpdate,
        snapshot: Box<RuntimeSnapshot>,
    },
    DirectorCompleted {
        result: DirectorResult,
        snapshot: Box<RuntimeSnapshot>,
    },
    SessionCharacterCreated {
        session_character: Box<SessionCharacterPayload>,
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
}
