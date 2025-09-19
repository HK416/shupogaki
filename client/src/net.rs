use std::cmp::Reverse;

use bevy::{prelude::*, tasks::Task};
use serde::Deserialize;

// --- MODELS ---

#[derive(Deserialize, Clone, PartialEq, Eq)]
pub struct RankingEntry {
    pub name: String,
    pub score: u32,
}

impl Ord for RankingEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        Reverse(self.score).cmp(&Reverse(other.score))
    }
}

impl PartialOrd<Self> for RankingEntry {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Reverse(self.score).partial_cmp(&Reverse(other.score))
    }
}

// --- COMPONENTS ---

#[derive(Component)]
pub struct RankingTask(pub Task<Result<Vec<RankingEntry>, reqwest::Error>>);

// --- RESOURCES ----

#[derive(Default, Resource)]
pub struct HttpClient(pub reqwest::Client);

#[derive(Default, Resource)]
pub enum RankingStatus {
    #[default]
    Loading,
    Success {
        timer: Timer,
        entires: Vec<RankingEntry>,
    },
    Failed(String),
}
