use serde::Deserialize;
use std::sync::OnceLock;

use crate::model::Position;

static CHART_BOOK: OnceLock<ChartBook> = OnceLock::new();

#[derive(Debug, Deserialize)]
pub struct ChartBook {
    pub opens: Vec<OpenChartEntry>,
    pub cold_calls: Vec<VersusChartEntry>,
    pub continue_vs_3bet: Vec<VersusChartEntry>,
    pub three_bets: Vec<VersusChartEntry>,
    pub continue_vs_4bet: Vec<VersusChartEntry>,
}

#[derive(Debug, Deserialize)]
pub struct OpenChartEntry {
    pub position: String,
    pub max_stack_bb: Option<f32>,
    pub tokens: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct VersusChartEntry {
    pub hero_position: String,
    pub villain_position: String,
    pub max_stack_bb: Option<f32>,
    pub tokens: Vec<String>,
}

pub fn chart_book() -> &'static ChartBook {
    CHART_BOOK.get_or_init(|| {
        serde_json::from_str(include_str!("../data/preflop_charts.json"))
            .expect("valid preflop chart json")
    })
}

impl ChartBook {
    pub fn open_range(&self, position: Position, stack_bb: f32) -> &[String] {
        select_open_entry(&self.opens, position, stack_bb)
    }

    pub fn cold_call_range(
        &self,
        hero_position: Position,
        villain_position: Position,
        stack_bb: f32,
    ) -> &[String] {
        select_versus_entry(&self.cold_calls, hero_position, villain_position, stack_bb)
    }

    pub fn continue_vs_3bet(
        &self,
        opener: Position,
        three_bettor: Position,
        stack_bb: f32,
    ) -> &[String] {
        select_versus_entry(&self.continue_vs_3bet, opener, three_bettor, stack_bb)
    }

    pub fn three_bet_range(
        &self,
        hero_position: Position,
        villain_position: Position,
        stack_bb: f32,
    ) -> &[String] {
        select_versus_entry(&self.three_bets, hero_position, villain_position, stack_bb)
    }

    pub fn continue_vs_4bet(
        &self,
        hero_position: Position,
        villain_position: Position,
        stack_bb: f32,
    ) -> &[String] {
        select_versus_entry(&self.continue_vs_4bet, hero_position, villain_position, stack_bb)
    }
}

fn select_open_entry(entries: &[OpenChartEntry], position: Position, stack_bb: f32) -> &[String] {
    entries
        .iter()
        .filter(|entry| entry.position == position.to_string())
        .filter(|entry| entry.max_stack_bb.map(|max| stack_bb <= max).unwrap_or(true))
        .min_by(|left, right| {
            let left_key = left.max_stack_bb.unwrap_or(f32::INFINITY);
            let right_key = right.max_stack_bb.unwrap_or(f32::INFINITY);
            left_key.total_cmp(&right_key)
        })
        .map(|entry| entry.tokens.as_slice())
        .unwrap_or(&[])
}

fn select_versus_entry(
    entries: &[VersusChartEntry],
    hero_position: Position,
    villain_position: Position,
    stack_bb: f32,
) -> &[String] {
    entries
        .iter()
        .filter(|entry| {
            entry.hero_position == hero_position.to_string()
                && entry.villain_position == villain_position.to_string()
        })
        .filter(|entry| entry.max_stack_bb.map(|max| stack_bb <= max).unwrap_or(true))
        .min_by(|left, right| {
            let left_key = left.max_stack_bb.unwrap_or(f32::INFINITY);
            let right_key = right.max_stack_bb.unwrap_or(f32::INFINITY);
            left_key.total_cmp(&right_key)
        })
        .map(|entry| entry.tokens.as_slice())
        .unwrap_or(&[])
}
