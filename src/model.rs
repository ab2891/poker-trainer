use rand::{rngs::StdRng, seq::SliceRandom, Rng, SeedableRng};
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::{Mutex, OnceLock};

use crate::charts::chart_book;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Street {
    Preflop,
    Flop,
    Turn,
    River,
}

impl fmt::Display for Street {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Street::Preflop => "Preflop",
            Street::Flop => "Flop",
            Street::Turn => "Turn",
            Street::River => "River",
        };
        f.write_str(label)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Position {
    Utg,
    Hj,
    Co,
    Btn,
    Sb,
    Bb,
}

impl Position {
    pub fn blind_contribution(self) -> f32 {
        match self {
            Position::Sb => 0.5,
            Position::Bb => 1.0,
            _ => 0.0,
        }
    }

    pub fn postflop_order(self) -> u8 {
        match self {
            Position::Sb => 0,
            Position::Bb => 1,
            Position::Utg => 2,
            Position::Hj => 3,
            Position::Co => 4,
            Position::Btn => 5,
        }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Position::Utg => "UTG",
            Position::Hj => "HJ",
            Position::Co => "CO",
            Position::Btn => "BTN",
            Position::Sb => "SB",
            Position::Bb => "BB",
        };
        f.write_str(label)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Raise,
    Call,
    Fold,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            Action::Raise => "Raise",
            Action::Call => "Call",
            Action::Fold => "Fold",
        };
        f.write_str(label)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

impl Suit {
    pub fn symbol(self) -> &'static str {
        match self {
            Suit::Hearts => "♥",
            Suit::Diamonds => "♦",
            Suit::Clubs => "♣",
            Suit::Spades => "♠",
        }
    }

    pub fn color_hex(self) -> &'static str {
        match self {
            Suit::Hearts => "#f06b78",
            Suit::Diamonds => "#f0a36b",
            Suit::Clubs => "#7bd389",
            Suit::Spades => "#b9c2d0",
        }
    }

    pub fn all() -> [Suit; 4] {
        [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Rank {
    Ace,
    King,
    Queen,
    Jack,
    Ten,
    Nine,
    Eight,
    Seven,
    Six,
    Five,
    Four,
    Three,
    Two,
}

impl Rank {
    pub fn short(self) -> &'static str {
        match self {
            Rank::Ace => "A",
            Rank::King => "K",
            Rank::Queen => "Q",
            Rank::Jack => "J",
            Rank::Ten => "T",
            Rank::Nine => "9",
            Rank::Eight => "8",
            Rank::Seven => "7",
            Rank::Six => "6",
            Rank::Five => "5",
            Rank::Four => "4",
            Rank::Three => "3",
            Rank::Two => "2",
        }
    }

    pub fn value(self) -> u8 {
        match self {
            Rank::Ace => 14,
            Rank::King => 13,
            Rank::Queen => 12,
            Rank::Jack => 11,
            Rank::Ten => 10,
            Rank::Nine => 9,
            Rank::Eight => 8,
            Rank::Seven => 7,
            Rank::Six => 6,
            Rank::Five => 5,
            Rank::Four => 4,
            Rank::Three => 3,
            Rank::Two => 2,
        }
    }

    pub fn from_value(value: u8) -> Rank {
        match value {
            14 => Rank::Ace,
            13 => Rank::King,
            12 => Rank::Queen,
            11 => Rank::Jack,
            10 => Rank::Ten,
            9 => Rank::Nine,
            8 => Rank::Eight,
            7 => Rank::Seven,
            6 => Rank::Six,
            5 => Rank::Five,
            4 => Rank::Four,
            3 => Rank::Three,
            _ => Rank::Two,
        }
    }

    pub fn from_char(ch: char) -> Rank {
        match ch {
            'A' => Rank::Ace,
            'K' => Rank::King,
            'Q' => Rank::Queen,
            'J' => Rank::Jack,
            'T' => Rank::Ten,
            '9' => Rank::Nine,
            '8' => Rank::Eight,
            '7' => Rank::Seven,
            '6' => Rank::Six,
            '5' => Rank::Five,
            '4' => Rank::Four,
            '3' => Rank::Three,
            '2' => Rank::Two,
            _ => panic!("unsupported rank"),
        }
    }

    pub fn all() -> [Rank; 13] {
        [
            Rank::Ace,
            Rank::King,
            Rank::Queen,
            Rank::Jack,
            Rank::Ten,
            Rank::Nine,
            Rank::Eight,
            Rank::Seven,
            Rank::Six,
            Rank::Five,
            Rank::Four,
            Rank::Three,
            Rank::Two,
        ]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    pub fn label(self) -> String {
        format!("{}{}", self.rank.short(), self.suit.symbol())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct HoleCards {
    pub first: Card,
    pub second: Card,
}

impl HoleCards {
    pub fn descriptor(self) -> String {
        format!("{} {}", self.first.label(), self.second.label())
    }

    pub fn contains(self, card: Card) -> bool {
        self.first == card || self.second == card
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScenarioKind {
    OpenRaiseFirstIn,
    FacingOpen,
    FacingThreeBet,
    FacingSqueeze,
}

impl fmt::Display for ScenarioKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self {
            ScenarioKind::OpenRaiseFirstIn => "RFI",
            ScenarioKind::FacingOpen => "Facing Open",
            ScenarioKind::FacingThreeBet => "Facing 3-Bet",
            ScenarioKind::FacingSqueeze => "Facing Squeeze",
        };
        f.write_str(label)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrainingMode {
    Mixed,
    RaiseFirstIn,
    OpenDefense,
    ThreeBetDefense,
    SqueezeDefense,
}

impl TrainingMode {
    pub fn label(self) -> &'static str {
        match self {
            TrainingMode::Mixed => "Mixed",
            TrainingMode::RaiseFirstIn => "RFI",
            TrainingMode::OpenDefense => "Vs Open",
            TrainingMode::ThreeBetDefense => "Vs 3-Bet",
            TrainingMode::SqueezeDefense => "Vs Squeeze",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TrainingConfig {
    pub stack_depth_bb: f32,
    pub rake_pct: f32,
    pub training_mode: TrainingMode,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            stack_depth_bb: 100.0,
            rake_pct: 0.0,
            training_mode: TrainingMode::Mixed,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FacingAction {
    pub size_bb: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreflopActionKind {
    FoldedToHero,
    OpenRaise,
    FlatCall,
    ThreeBet,
    Squeeze,
}

#[derive(Clone, Debug)]
pub struct PreflopAction {
    pub actor: Position,
    pub kind: PreflopActionKind,
    pub size_bb: Option<f32>,
}

#[derive(Clone, Debug)]
pub struct ActionEvaluation {
    pub action: Action,
    pub ev_bb: f32,
    pub equity_pct: f32,
    pub fold_equity_pct: f32,
    pub explanation: String,
}

#[derive(Clone, Debug)]
pub struct TrainingSpot {
    pub title: String,
    pub street: Street,
    pub scenario_kind: ScenarioKind,
    pub hero_position: Position,
    pub villain_position: Position,
    pub opener_position: Option<Position>,
    pub hole_cards: HoleCards,
    pub board: Vec<Card>,
    pub pot_bb: f32,
    pub stack_bb: f32,
    pub rake_pct: f32,
    pub hero_invested_bb: f32,
    pub call_cost_bb: f32,
    pub raise_to_bb: f32,
    pub pot_odds_pct: f32,
    pub villain_range_pct: f32,
    pub prompt: String,
    pub facing: FacingAction,
    pub action_history: Vec<PreflopAction>,
    pub evaluations: Vec<ActionEvaluation>,
}

impl TrainingSpot {
    pub fn best_action(&self) -> &ActionEvaluation {
        self.evaluations
            .iter()
            .max_by(|left, right| left.ev_bb.total_cmp(&right.ev_bb))
            .expect("training spot requires evaluations")
    }

    pub fn evaluation_for(&self, action: Action) -> &ActionEvaluation {
        self.evaluations
            .iter()
            .find(|entry| entry.action == action)
            .expect("evaluation missing for action")
    }

    pub fn hero_is_ip(&self) -> bool {
        self.hero_position.postflop_order() > self.villain_position.postflop_order()
    }

    pub fn action_history_summary(&self) -> String {
        self.action_history
            .iter()
            .map(|action| match action.kind {
                PreflopActionKind::FoldedToHero => "Action folds to you".to_owned(),
                PreflopActionKind::OpenRaise => {
                    format!("{} opened to {:.1} BB", action.actor, action.size_bb.unwrap_or(0.0))
                }
                PreflopActionKind::FlatCall => {
                    format!("{} called {:.1} BB", action.actor, action.size_bb.unwrap_or(0.0))
                }
                PreflopActionKind::ThreeBet => {
                    format!("{} 3-bet to {:.1} BB", action.actor, action.size_bb.unwrap_or(0.0))
                }
                PreflopActionKind::Squeeze => {
                    format!("{} squeezed to {:.1} BB", action.actor, action.size_bb.unwrap_or(0.0))
                }
            })
            .collect::<Vec<_>>()
            .join(" -> ")
    }
}

#[derive(Clone, Debug)]
pub struct DecisionFeedback {
    pub selected_action: Action,
    pub selected_ev_bb: f32,
    pub selected_equity_pct: f32,
    pub selected_fold_equity_pct: f32,
    pub correct_action: Action,
    pub correct_ev_bb: f32,
    pub correct_equity_pct: f32,
    pub correct_fold_equity_pct: f32,
    pub pot_odds_pct: f32,
    pub is_correct: bool,
    pub explanation: String,
}

#[derive(Debug)]
pub struct TrainingSession {
    current_spot: TrainingSpot,
    pub config: TrainingConfig,
    pub answered_count: usize,
    pub correct_count: usize,
    pub current_feedback: Option<DecisionFeedback>,
}

impl TrainingSession {
    pub fn new() -> Self {
        let config = TrainingConfig::default();
        Self {
            current_spot: generate_training_spot(config),
            config,
            answered_count: 0,
            correct_count: 0,
            current_feedback: None,
        }
    }

    pub fn current_spot(&self) -> &TrainingSpot {
        &self.current_spot
    }

    pub fn answer_current(&mut self, action: Action) {
        if self.current_feedback.is_some() {
            return;
        }

        let spot = self.current_spot.clone();
        let selected = spot.evaluation_for(action).clone();
        let best = spot.best_action().clone();
        let is_correct = selected.action == best.action;

        self.answered_count += 1;
        if is_correct {
            self.correct_count += 1;
        }

        self.current_feedback = Some(DecisionFeedback {
            selected_action: selected.action,
            selected_ev_bb: selected.ev_bb,
            selected_equity_pct: selected.equity_pct,
            selected_fold_equity_pct: selected.fold_equity_pct,
            correct_action: best.action,
            correct_ev_bb: best.ev_bb,
            correct_equity_pct: best.equity_pct,
            correct_fold_equity_pct: best.fold_equity_pct,
            pot_odds_pct: spot.pot_odds_pct,
            is_correct,
            explanation: build_feedback_explanation(&spot, &selected, &best),
        });
    }

    pub fn next_spot(&mut self) {
        self.current_spot = generate_training_spot(self.config);
        self.current_feedback = None;
    }

    pub fn restart(&mut self) {
        self.answered_count = 0;
        self.correct_count = 0;
        self.current_spot = generate_training_spot(self.config);
        self.current_feedback = None;
    }

    pub fn apply_config(&mut self, config: TrainingConfig) {
        self.config = config;
        self.current_spot = generate_training_spot(config);
        self.current_feedback = None;
    }

    pub fn accuracy_pct(&self) -> f32 {
        if self.answered_count == 0 {
            0.0
        } else {
            (self.correct_count as f32 / self.answered_count as f32) * 100.0
        }
    }
}

#[derive(Clone, Debug)]
struct HandProfile {
    high: u8,
    pair: bool,
    suited: bool,
    gap: u8,
    connected: bool,
    broadway_count: u8,
    ace_blocker: bool,
    king_blocker: bool,
    wheel_shape: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComboShape {
    Pair,
    Suited,
    Offsuit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct HandClass {
    high: Rank,
    low: Rank,
    shape: ComboShape,
}

#[derive(Clone, Debug)]
struct WeightedHandClass {
    hand_class: HandClass,
    weight: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct WeightedCombo {
    cards: HoleCards,
    weight: f32,
}

struct RangeModel {
    primary: Vec<WeightedCombo>,
    continue_vs_raise: Vec<WeightedCombo>,
}

#[derive(Clone, Debug)]
struct ActionMetrics {
    action: Action,
    ev_bb: f32,
    equity_pct: f32,
    fold_equity_pct: f32,
}

#[derive(Clone, Debug)]
struct SpotSolution {
    metrics: Vec<ActionMetrics>,
    villain_range_pct: f32,
}

static EQUITY_CACHE: OnceLock<Mutex<HashMap<u64, f32>>> = OnceLock::new();

pub fn generate_training_spot(config: TrainingConfig) -> TrainingSpot {
    let mut rng = rand::thread_rng();
    let hole_cards = random_hole_cards(&mut rng);
    let scenario_kind = match config.training_mode {
        TrainingMode::Mixed => {
            let roll: f32 = rng.gen();
            if roll < 0.30 {
                ScenarioKind::OpenRaiseFirstIn
            } else if roll < 0.68 {
                ScenarioKind::FacingOpen
            } else if roll < 0.92 {
                ScenarioKind::FacingThreeBet
            } else {
                ScenarioKind::FacingSqueeze
            }
        }
        TrainingMode::RaiseFirstIn => ScenarioKind::OpenRaiseFirstIn,
        TrainingMode::OpenDefense => ScenarioKind::FacingOpen,
        TrainingMode::ThreeBetDefense => ScenarioKind::FacingThreeBet,
        TrainingMode::SqueezeDefense => ScenarioKind::FacingSqueeze,
    };

    match scenario_kind {
        ScenarioKind::OpenRaiseFirstIn => generate_rfi_spot(&mut rng, hole_cards, config),
        ScenarioKind::FacingOpen => generate_open_spot(&mut rng, hole_cards, config),
        ScenarioKind::FacingThreeBet => generate_three_bet_spot(&mut rng, hole_cards, config),
        ScenarioKind::FacingSqueeze => generate_squeeze_spot(&mut rng, hole_cards, config),
    }
}

fn generate_rfi_spot<R: Rng + ?Sized>(
    rng: &mut R,
    hole_cards: HoleCards,
    config: TrainingConfig,
) -> TrainingSpot {
    let hero_position = *[
        Position::Utg,
        Position::Hj,
        Position::Co,
        Position::Btn,
        Position::Sb,
    ]
    .choose(rng)
    .expect("rfi positions");
    let open_size = sample_hero_open_size(rng, hero_position, config.stack_depth_bb);
    let hero_invested_bb = hero_position.blind_contribution();
    let pot_bb = round_to_half(1.5 + hero_invested_bb);
    let call_cost_bb = 0.0;
    let prompt = "Unopened pot. Choose whether to open, limp-equivalent fold, or pass on the spot."
        .to_owned();

    let mut spot = TrainingSpot {
        title: format!("{} unopened preflop", hero_position),
        street: Street::Preflop,
        scenario_kind: ScenarioKind::OpenRaiseFirstIn,
        hero_position,
        villain_position: Position::Bb,
        opener_position: None,
        hole_cards,
        board: vec![],
        pot_bb,
        stack_bb: config.stack_depth_bb,
        rake_pct: config.rake_pct,
        hero_invested_bb,
        call_cost_bb,
        raise_to_bb: open_size,
        pot_odds_pct: 0.0,
        villain_range_pct: 0.0,
        prompt,
        facing: FacingAction {
            size_bb: 0.0,
        },
        action_history: vec![PreflopAction {
            actor: hero_position,
            kind: PreflopActionKind::FoldedToHero,
            size_bb: None,
        }],
        evaluations: vec![],
    };
    let solution = solve_spot(&spot);
    spot.villain_range_pct = solution.villain_range_pct;
    spot.evaluations = build_action_evaluations(&spot, &solution);
    spot
}

fn generate_open_spot<R: Rng + ?Sized>(
    rng: &mut R,
    hole_cards: HoleCards,
    config: TrainingConfig,
) -> TrainingSpot {
    let candidates = [
        (Position::Utg, Position::Hj),
        (Position::Utg, Position::Co),
        (Position::Utg, Position::Btn),
        (Position::Utg, Position::Sb),
        (Position::Utg, Position::Bb),
        (Position::Hj, Position::Co),
        (Position::Hj, Position::Btn),
        (Position::Hj, Position::Sb),
        (Position::Hj, Position::Bb),
        (Position::Co, Position::Btn),
        (Position::Co, Position::Sb),
        (Position::Co, Position::Bb),
        (Position::Btn, Position::Sb),
        (Position::Btn, Position::Bb),
        (Position::Sb, Position::Bb),
    ];

    let (villain_position, hero_position) = *candidates.choose(rng).expect("open candidates");
    let open_size = sample_open_size(rng, villain_position, config.stack_depth_bb);
    let hero_invested_bb = hero_position.blind_contribution();
    let pot_bb = round_to_half(1.5 + open_size - villain_position.blind_contribution());
    let call_cost_bb = round_to_half((open_size - hero_invested_bb).max(0.5));
    let hero_is_ip = hero_position.postflop_order() > villain_position.postflop_order();
    let raise_to_bb = suggest_raise_size(
        ScenarioKind::FacingOpen,
        open_size,
        hero_position,
        villain_position,
        hero_is_ip,
        config.stack_depth_bb,
    );
    let pot_odds_pct = call_cost_bb / (pot_bb + call_cost_bb) * 100.0;
    let facing = FacingAction { size_bb: open_size };

    let mut spot = TrainingSpot {
        title: format!("{} vs {} open", hero_position, villain_position),
        street: Street::Preflop,
        scenario_kind: ScenarioKind::FacingOpen,
        hero_position,
        villain_position,
        opener_position: Some(villain_position),
        hole_cards,
        board: vec![],
        pot_bb,
        stack_bb: config.stack_depth_bb,
        rake_pct: config.rake_pct,
        hero_invested_bb,
        call_cost_bb,
        raise_to_bb,
        pot_odds_pct: round_to_tenth(pot_odds_pct),
        villain_range_pct: 0.0,
        prompt: "Choose the best baseline response against a position-based opening range."
            .to_owned(),
        facing,
        action_history: vec![PreflopAction {
            actor: villain_position,
            kind: PreflopActionKind::OpenRaise,
            size_bb: Some(open_size),
        }],
        evaluations: vec![],
    };
    let solution = solve_spot(&spot);
    spot.villain_range_pct = solution.villain_range_pct;
    spot.evaluations = build_action_evaluations(&spot, &solution);
    spot
}

fn generate_three_bet_spot<R: Rng + ?Sized>(
    rng: &mut R,
    hole_cards: HoleCards,
    config: TrainingConfig,
) -> TrainingSpot {
    let candidates = [
        (Position::Utg, Position::Hj),
        (Position::Utg, Position::Co),
        (Position::Utg, Position::Btn),
        (Position::Utg, Position::Sb),
        (Position::Utg, Position::Bb),
        (Position::Hj, Position::Co),
        (Position::Hj, Position::Btn),
        (Position::Hj, Position::Sb),
        (Position::Hj, Position::Bb),
        (Position::Co, Position::Btn),
        (Position::Co, Position::Sb),
        (Position::Co, Position::Bb),
        (Position::Btn, Position::Sb),
        (Position::Btn, Position::Bb),
        (Position::Sb, Position::Bb),
    ];

    let (hero_position, villain_position) = *candidates.choose(rng).expect("3-bet candidates");
    if hand_weight_in_tokens(
        hole_cards,
        chart_book().open_range(hero_position, config.stack_depth_bb),
    ) <= 0.0
    {
        return generate_training_spot(config);
    }
    let open_size = sample_hero_open_size(rng, hero_position, config.stack_depth_bb);
    let villain_is_ip = villain_position.postflop_order() > hero_position.postflop_order();
    let three_bet_size = sample_three_bet_size(
        rng,
        open_size,
        villain_position,
        hero_position,
        villain_is_ip,
        config.stack_depth_bb,
    );
    let hero_invested_bb = open_size;
    let pot_bb =
        round_to_half(1.5 + open_size + (three_bet_size - villain_position.blind_contribution()));
    let call_cost_bb = round_to_half((three_bet_size - open_size).max(1.0));
    let hero_is_ip = hero_position.postflop_order() > villain_position.postflop_order();
    let raise_to_bb = suggest_raise_size(
        ScenarioKind::FacingThreeBet,
        three_bet_size,
        hero_position,
        villain_position,
        hero_is_ip,
        config.stack_depth_bb,
    );
    let pot_odds_pct = call_cost_bb / (pot_bb + call_cost_bb) * 100.0;
    let facing = FacingAction {
        size_bb: three_bet_size,
    };

    let mut spot = TrainingSpot {
        title: format!("{} facing a {} 3-bet", hero_position, villain_position),
        street: Street::Preflop,
        scenario_kind: ScenarioKind::FacingThreeBet,
        hero_position,
        villain_position,
        opener_position: Some(hero_position),
        hole_cards,
        board: vec![],
        pot_bb,
        stack_bb: config.stack_depth_bb,
        rake_pct: config.rake_pct,
        hero_invested_bb,
        call_cost_bb,
        raise_to_bb,
        pot_odds_pct: round_to_tenth(pot_odds_pct),
        villain_range_pct: 0.0,
        prompt: "Treat Raise as a 4-bet and respond to a position-based 3-bet range."
            .to_owned(),
        facing,
        action_history: vec![
            PreflopAction {
                actor: hero_position,
                kind: PreflopActionKind::OpenRaise,
                size_bb: Some(open_size),
            },
            PreflopAction {
                actor: villain_position,
                kind: PreflopActionKind::ThreeBet,
                size_bb: Some(three_bet_size),
            },
        ],
        evaluations: vec![],
    };
    let solution = solve_spot(&spot);
    spot.villain_range_pct = solution.villain_range_pct;
    spot.evaluations = build_action_evaluations(&spot, &solution);
    spot
}

fn generate_squeeze_spot<R: Rng + ?Sized>(
    rng: &mut R,
    hole_cards: HoleCards,
    config: TrainingConfig,
) -> TrainingSpot {
    let candidates = [
        (Position::Utg, Position::Hj, Position::Btn),
        (Position::Hj, Position::Co, Position::Btn),
        (Position::Co, Position::Btn, Position::Sb),
        (Position::Co, Position::Btn, Position::Bb),
        (Position::Btn, Position::Sb, Position::Bb),
    ];
    let (open_position, hero_position, squeezer_position) =
        *candidates.choose(rng).expect("squeeze candidates");
    if hand_weight_in_tokens(
        hole_cards,
        chart_book().cold_call_range(hero_position, open_position, config.stack_depth_bb),
    ) <= 0.0
    {
        return generate_training_spot(config);
    }
    let open_size = sample_open_size(rng, open_position, config.stack_depth_bb);
    let squeeze_size = sample_squeeze_size(
        rng,
        open_size,
        squeezer_position,
        hero_position,
        config.stack_depth_bb,
    );
    let hero_invested_bb = open_size;
    let cold_call_bb = open_size;
    let pot_bb = round_to_half(
        1.5
            + open_size
            + cold_call_bb
            + (squeeze_size - squeezer_position.blind_contribution()),
    );
    let call_cost_bb = round_to_half((squeeze_size - open_size).max(1.0));
    let hero_is_ip = hero_position.postflop_order() > squeezer_position.postflop_order();
    let raise_to_bb = suggest_raise_size(
        ScenarioKind::FacingSqueeze,
        squeeze_size,
        hero_position,
        squeezer_position,
        hero_is_ip,
        config.stack_depth_bb,
    );
    let pot_odds_pct = call_cost_bb / (pot_bb + call_cost_bb) * 100.0;

    let mut spot = TrainingSpot {
        title: format!("{} facing a {} squeeze", hero_position, squeezer_position),
        street: Street::Preflop,
        scenario_kind: ScenarioKind::FacingSqueeze,
        hero_position,
        villain_position: squeezer_position,
        opener_position: Some(open_position),
        hole_cards,
        board: vec![],
        pot_bb,
        stack_bb: config.stack_depth_bb,
        rake_pct: config.rake_pct,
        hero_invested_bb,
        call_cost_bb,
        raise_to_bb,
        pot_odds_pct: round_to_tenth(pot_odds_pct),
        villain_range_pct: 0.0,
        prompt: format!(
            "{} opened to {:.1} BB, you called in {}, and {} squeezed to {:.1} BB.",
            open_position, open_size, hero_position, squeezer_position, squeeze_size
        ),
        facing: FacingAction {
            size_bb: squeeze_size,
        },
        action_history: vec![
            PreflopAction {
                actor: open_position,
                kind: PreflopActionKind::OpenRaise,
                size_bb: Some(open_size),
            },
            PreflopAction {
                actor: hero_position,
                kind: PreflopActionKind::FlatCall,
                size_bb: Some(open_size),
            },
            PreflopAction {
                actor: squeezer_position,
                kind: PreflopActionKind::Squeeze,
                size_bb: Some(squeeze_size),
            },
        ],
        evaluations: vec![],
    };
    let solution = solve_spot(&spot);
    spot.villain_range_pct = solution.villain_range_pct;
    spot.evaluations = build_action_evaluations(&spot, &solution);
    spot
}

fn solve_spot(spot: &TrainingSpot) -> SpotSolution {
    if matches!(spot.scenario_kind, ScenarioKind::OpenRaiseFirstIn) {
        return solve_rfi_spot(spot);
    }

    let profile = analyze_hand(spot.hole_cards);
    let hero_is_ip = spot.hero_is_ip();
    let range_model = build_range_model(spot);
    let villain_range_pct = combo_percent(weighted_combo_total(&range_model.primary));

    let sample_count = if spot.stack_bb <= 40.0 { 180 } else { 240 };
    let call_equity_pct = simulate_equity_pct(spot.hole_cards, &range_model.primary, sample_count);
    let continue_equity_pct = simulate_equity_pct(
        spot.hole_cards,
        &range_model.continue_vs_raise,
        sample_count.saturating_sub(60),
    );

    let primary_weight = weighted_combo_total(&range_model.primary);
    let continue_weight = weighted_combo_total(&range_model.continue_vs_raise);
    let raw_fold_equity_pct = if primary_weight <= f32::EPSILON {
        0.0
    } else {
        round_to_tenth((1.0 - continue_weight / primary_weight) * 100.0)
    };
    let fold_equity_pct = adjusted_fold_equity_pct(
        raw_fold_equity_pct,
        &profile,
        spot.scenario_kind,
        hero_is_ip,
        spot.stack_bb,
    );

    let realized_call_equity =
        call_equity_pct * call_realization(&profile, hero_is_ip, spot.scenario_kind, spot.stack_bb);
    let call_rake = rake_in_bb(spot.pot_bb + spot.call_cost_bb, spot.rake_pct, spot.stack_bb);
    let call_ev = (realized_call_equity / 100.0) * ((spot.pot_bb + spot.call_cost_bb) - call_rake)
        - spot.call_cost_bb;

    let raise_cost_bb = spot.raise_to_bb - spot.hero_invested_bb;
    let villain_call_cost_bb = spot.raise_to_bb - spot.facing.size_bb;
    let final_pot_if_called = spot.pot_bb + raise_cost_bb + villain_call_cost_bb;
    let raise_realized_equity =
        continue_equity_pct * raise_realization(&profile, hero_is_ip, spot.stack_bb);
    let raise_rake = rake_in_bb(final_pot_if_called, spot.rake_pct, spot.stack_bb);
    let weak_bluff_tax = weak_bluff_penalty_bb(&profile, spot.scenario_kind, spot.stack_bb);
    let raise_ev = (fold_equity_pct / 100.0) * spot.pot_bb
        + (1.0 - fold_equity_pct / 100.0)
            * ((raise_realized_equity / 100.0) * (final_pot_if_called - raise_rake) - raise_cost_bb)
        - weak_bluff_tax;

    let fold_ev = -spot.hero_invested_bb;

    let metrics = vec![
        ActionMetrics {
            action: Action::Raise,
            ev_bb: round_to_cent(raise_ev),
            equity_pct: round_to_tenth(continue_equity_pct),
            fold_equity_pct,
        },
        ActionMetrics {
            action: Action::Call,
            ev_bb: round_to_cent(call_ev),
            equity_pct: round_to_tenth(call_equity_pct),
            fold_equity_pct: 0.0,
        },
        ActionMetrics {
            action: Action::Fold,
            ev_bb: round_to_cent(fold_ev),
            equity_pct: 0.0,
            fold_equity_pct: 0.0,
        },
    ];

    let _ = profile;
    SpotSolution {
        metrics,
        villain_range_pct,
    }
}

fn solve_rfi_spot(spot: &TrainingSpot) -> SpotSolution {
    let open_tokens = chart_book().open_range(spot.hero_position, spot.stack_bb);
    let open_weight = hand_weight_in_tokens(spot.hole_cards, open_tokens);
    let open_freq_pct = combo_percent(total_weight_from_tokens(open_tokens));
    let in_open_range = open_weight > 0.0;
    let steal_success_pct = match spot.hero_position {
        Position::Utg => 41.0,
        Position::Hj => 45.0,
        Position::Co => 51.0,
        Position::Btn => 58.0,
        Position::Sb => 48.0,
        Position::Bb => 0.0,
    };
    let realization_bonus = if spot.stack_bb <= 40.0 { 0.82 } else { 1.0 };
    let raise_ev = if in_open_range {
        (steal_success_pct / 100.0) * spot.pot_bb
            + (1.0 - steal_success_pct / 100.0)
                * ((open_weight.min(1.0) * 0.24 * realization_bonus) * (spot.raise_to_bb + spot.pot_bb)
                    - (spot.raise_to_bb - spot.hero_invested_bb))
    } else {
        -0.35 - (spot.raise_to_bb * 0.08)
    };
    let call_ev = if matches!(spot.hero_position, Position::Sb) {
        -0.18
    } else {
        -0.30
    };
    let fold_ev = -spot.hero_invested_bb;

    SpotSolution {
        metrics: vec![
            ActionMetrics {
                action: Action::Raise,
                ev_bb: round_to_cent(raise_ev),
                equity_pct: round_to_tenth(steal_success_pct * 0.55),
                fold_equity_pct: round_to_tenth(steal_success_pct),
            },
            ActionMetrics {
                action: Action::Call,
                ev_bb: round_to_cent(call_ev),
                equity_pct: 0.0,
                fold_equity_pct: 0.0,
            },
            ActionMetrics {
                action: Action::Fold,
                ev_bb: round_to_cent(fold_ev),
                equity_pct: 0.0,
                fold_equity_pct: 0.0,
            },
        ],
        villain_range_pct: open_freq_pct,
    }
}

fn build_range_model(spot: &TrainingSpot) -> RangeModel {
    let primary_tokens: Vec<String> = match spot.scenario_kind {
        ScenarioKind::OpenRaiseFirstIn => {
            chart_book().open_range(spot.hero_position, spot.stack_bb).to_vec()
        }
        ScenarioKind::FacingOpen => {
            chart_book().open_range(spot.villain_position, spot.stack_bb).to_vec()
        }
        ScenarioKind::FacingThreeBet => {
            chart_book()
                .three_bet_range(spot.hero_position, spot.villain_position, spot.stack_bb)
                .to_vec()
        }
        ScenarioKind::FacingSqueeze => {
            chart_book()
                .three_bet_range(spot.hero_position, spot.villain_position, spot.stack_bb)
                .to_vec()
        }
    };
    let continue_tokens: Vec<String> = match spot.scenario_kind {
        ScenarioKind::OpenRaiseFirstIn => Vec::new(),
        ScenarioKind::FacingOpen => chart_book()
            .continue_vs_3bet(spot.villain_position, spot.hero_position, spot.stack_bb)
            .to_vec(),
        ScenarioKind::FacingThreeBet => {
            chart_book()
                .continue_vs_4bet(spot.hero_position, spot.villain_position, spot.stack_bb)
                .to_vec()
        }
        ScenarioKind::FacingSqueeze => {
            chart_book()
                .continue_vs_4bet(spot.hero_position, spot.villain_position, spot.stack_bb)
                .to_vec()
        }
    };

    let primary = expand_range_to_combos(&primary_tokens, spot.hole_cards);
    let continue_vs_raise = expand_range_to_combos(&continue_tokens, spot.hole_cards);

    RangeModel {
        primary,
        continue_vs_raise,
    }
}

fn build_action_evaluations(spot: &TrainingSpot, solution: &SpotSolution) -> Vec<ActionEvaluation> {
    let profile = analyze_hand(spot.hole_cards);
    let hero_is_ip = spot.hero_is_ip();
    solution
        .metrics
        .iter()
        .map(|metric| {
            let explanation = match metric.action {
                Action::Raise => raise_line_reason(
                    &profile,
                    spot.scenario_kind,
                    hero_is_ip,
                    metric.fold_equity_pct,
                    metric.equity_pct,
                    spot.stack_bb,
                ),
                Action::Call => call_line_reason(
                    &profile,
                    spot.pot_odds_pct,
                    metric.equity_pct,
                    hero_is_ip,
                    spot.stack_bb,
                ),
                Action::Fold => {
                    let call_equity = solution
                        .metrics
                        .iter()
                        .find(|entry| entry.action == Action::Call)
                        .map(|entry| entry.equity_pct)
                        .unwrap_or(0.0);
                    fold_line_reason(&profile, spot.pot_odds_pct, call_equity)
                }
            };

            ActionEvaluation {
                action: metric.action,
                ev_bb: metric.ev_bb,
                equity_pct: metric.equity_pct,
                fold_equity_pct: metric.fold_equity_pct,
                explanation,
            }
        })
        .collect()
}

fn expand_range_to_combos(tokens: &[String], hero_cards: HoleCards) -> Vec<WeightedCombo> {
    let mut combos = Vec::new();
    for hand_class in parse_range(tokens) {
        combos.extend(expand_hand_class(hand_class, hero_cards));
    }
    combos
}

fn parse_range(tokens: &[String]) -> Vec<WeightedHandClass> {
    let mut classes = Vec::new();
    for token in tokens {
        let (token_body, weight) = parse_weighted_token(token);
        if token_body.contains('-') {
            classes.extend(parse_dash_token(token_body, weight));
        } else if token_body.ends_with('+') {
            classes.extend(parse_plus_token(token_body, weight));
        } else {
            classes.push(WeightedHandClass {
                hand_class: parse_exact_hand_class(token_body),
                weight,
            });
        }
    }
    classes
}

fn parse_weighted_token(token: &str) -> (&str, f32) {
    if let Some((body, weight)) = token.split_once('@') {
        (body, weight.parse::<f32>().unwrap_or(1.0))
    } else {
        (token, 1.0)
    }
}

fn parse_dash_token(token: &str, weight: f32) -> Vec<WeightedHandClass> {
    let (start, end) = token.split_once('-').expect("invalid dash token");
    let first = parse_exact_hand_class(start);
    let last = parse_exact_hand_class(end);

    if first.shape == ComboShape::Pair && last.shape == ComboShape::Pair {
        let low = last.high.value();
        let high = first.high.value();
        return (low..=high)
            .rev()
            .map(|value| WeightedHandClass {
                hand_class: HandClass {
                    high: Rank::from_value(value),
                    low: Rank::from_value(value),
                    shape: ComboShape::Pair,
                },
                weight,
            })
            .collect();
    }

    if first.high == last.high && first.shape == last.shape {
        let low = first.low.value().min(last.low.value());
        let high = first.low.value().max(last.low.value());
        return (low..=high)
            .rev()
            .map(|value| WeightedHandClass {
                hand_class: HandClass {
                    high: first.high,
                    low: Rank::from_value(value),
                    shape: first.shape,
                },
                weight,
            })
            .collect();
    }

    panic!("unsupported dash token format: {token}");
}

fn parse_plus_token(token: &str, weight: f32) -> Vec<WeightedHandClass> {
    let base = &token[..token.len() - 1];
    let hand_class = parse_exact_hand_class(base);
    if hand_class.shape == ComboShape::Pair {
        return (hand_class.high.value()..=14)
            .rev()
            .map(|value| WeightedHandClass {
                hand_class: HandClass {
                    high: Rank::from_value(value),
                    low: Rank::from_value(value),
                    shape: ComboShape::Pair,
                },
                weight,
            })
            .collect();
    }

    let high = hand_class.high.value();
    let low_start = hand_class.low.value();
    (low_start..high)
        .rev()
        .map(|value| WeightedHandClass {
            hand_class: HandClass {
                high: hand_class.high,
                low: Rank::from_value(value),
                shape: hand_class.shape,
            },
            weight,
        })
        .collect()
}

fn parse_exact_hand_class(token: &str) -> HandClass {
    let chars: Vec<char> = token.chars().collect();
    match chars.len() {
        2 => {
            let first = Rank::from_char(chars[0]);
            let second = Rank::from_char(chars[1]);
            HandClass {
                high: first,
                low: second,
                shape: ComboShape::Pair,
            }
        }
        3 => {
            let mut high = Rank::from_char(chars[0]);
            let mut low = Rank::from_char(chars[1]);
            if low.value() > high.value() {
                std::mem::swap(&mut high, &mut low);
            }
            let shape = match chars[2] {
                's' => ComboShape::Suited,
                'o' => ComboShape::Offsuit,
                _ => panic!("unsupported hand class"),
            };
            HandClass { high, low, shape }
        }
        _ => panic!("unsupported token length"),
    }
}

fn normalized_hand_class(hole_cards: HoleCards) -> HandClass {
    let mut first = hole_cards.first.rank;
    let mut second = hole_cards.second.rank;
    if second.value() > first.value() {
        std::mem::swap(&mut first, &mut second);
    }
    let shape = if first == second {
        ComboShape::Pair
    } else if hole_cards.first.suit == hole_cards.second.suit {
        ComboShape::Suited
    } else {
        ComboShape::Offsuit
    };
    HandClass {
        high: first,
        low: second,
        shape,
    }
}

fn hand_weight_in_tokens(hole_cards: HoleCards, tokens: &[String]) -> f32 {
    let target = normalized_hand_class(hole_cards);
    parse_range(tokens)
        .into_iter()
        .find(|entry| entry.hand_class == target)
        .map(|entry| entry.weight)
        .unwrap_or(0.0)
}

fn total_weight_from_tokens(tokens: &[String]) -> f32 {
    parse_range(tokens)
        .into_iter()
        .map(|entry| {
            let combos = match entry.hand_class.shape {
                ComboShape::Pair => 6.0,
                ComboShape::Suited => 4.0,
                ComboShape::Offsuit => 12.0,
            };
            combos * entry.weight
        })
        .sum()
}

fn expand_hand_class(weighted: WeightedHandClass, hero_cards: HoleCards) -> Vec<WeightedCombo> {
    let mut combos = Vec::new();
    let hand_class = weighted.hand_class;
    match hand_class.shape {
        ComboShape::Pair => {
            let suits = Suit::all();
            for left_index in 0..suits.len() {
                for right_index in (left_index + 1)..suits.len() {
                    let combo = HoleCards {
                        first: Card {
                            rank: hand_class.high,
                            suit: suits[left_index],
                        },
                        second: Card {
                            rank: hand_class.low,
                            suit: suits[right_index],
                        },
                    };
                    if !hero_cards.contains(combo.first) && !hero_cards.contains(combo.second) {
                        combos.push(WeightedCombo { cards: combo, weight: weighted.weight });
                    }
                }
            }
        }
        ComboShape::Suited => {
            for suit in Suit::all() {
                let combo = HoleCards {
                    first: Card {
                        rank: hand_class.high,
                        suit,
                    },
                    second: Card {
                        rank: hand_class.low,
                        suit,
                    },
                };
                if !hero_cards.contains(combo.first) && !hero_cards.contains(combo.second) {
                    combos.push(WeightedCombo { cards: combo, weight: weighted.weight });
                }
            }
        }
        ComboShape::Offsuit => {
            for first_suit in Suit::all() {
                for second_suit in Suit::all() {
                    if first_suit == second_suit {
                        continue;
                    }
                    let combo = HoleCards {
                        first: Card {
                            rank: hand_class.high,
                            suit: first_suit,
                        },
                        second: Card {
                            rank: hand_class.low,
                            suit: second_suit,
                        },
                    };
                    if !hero_cards.contains(combo.first) && !hero_cards.contains(combo.second) {
                        combos.push(WeightedCombo { cards: combo, weight: weighted.weight });
                    }
                }
            }
        }
    }
    combos
}

fn weighted_combo_total(combos: &[WeightedCombo]) -> f32 {
    combos.iter().map(|combo| combo.weight).sum()
}

fn weighted_choice<'a>(combos: &'a [WeightedCombo], rng: &mut StdRng) -> Option<&'a WeightedCombo> {
    let total = weighted_combo_total(combos);
    if total <= f32::EPSILON {
        return None;
    }
    let mut roll = rng.gen_range(0.0..total);
    for combo in combos {
        roll -= combo.weight;
        if roll <= 0.0 {
            return Some(combo);
        }
    }
    combos.last()
}

fn equity_cache_key(hero_cards: HoleCards, villain_combos: &[WeightedCombo], iterations: usize) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    hero_cards.hash(&mut hasher);
    iterations.hash(&mut hasher);
    for combo in villain_combos {
        combo.cards.hash(&mut hasher);
        ((combo.weight * 1000.0).round() as i64).hash(&mut hasher);
    }
    hasher.finish()
}

fn simulate_equity_pct(
    hero_cards: HoleCards,
    villain_combos: &[WeightedCombo],
    iterations: usize,
) -> f32 {
    if villain_combos.is_empty() || iterations == 0 {
        return 0.0;
    }

    let cache_key = equity_cache_key(hero_cards, villain_combos, iterations);
    let cache = EQUITY_CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(value) = cache.lock().expect("equity cache").get(&cache_key).copied() {
        return value;
    }

    let deck = full_deck();
    let mut equity = 0.0;
    let mut rng = StdRng::seed_from_u64(cache_key);

    for _ in 0..iterations {
        let villain = weighted_choice(villain_combos, &mut rng)
            .copied()
            .expect("villain combo");
        let board = sample_board(&deck, hero_cards, villain.cards, &mut rng);
        let hero_score = best_of_seven([
            hero_cards.first,
            hero_cards.second,
            board[0],
            board[1],
            board[2],
            board[3],
            board[4],
        ]);
        let villain_score = best_of_seven([
            villain.cards.first,
            villain.cards.second,
            board[0],
            board[1],
            board[2],
            board[3],
            board[4],
        ]);

        if hero_score > villain_score {
            equity += 1.0;
        } else if hero_score == villain_score {
            equity += 0.5;
        }
    }

    let value = round_to_tenth((equity / iterations as f32) * 100.0);
    cache
        .lock()
        .expect("equity cache")
        .insert(cache_key, value);
    value
}

fn sample_board<R: Rng + ?Sized>(
    deck: &[Card],
    hero_cards: HoleCards,
    villain_cards: HoleCards,
    rng: &mut R,
) -> [Card; 5] {
    let available: Vec<Card> = deck
        .iter()
        .copied()
        .filter(|card| {
            *card != hero_cards.first
                && *card != hero_cards.second
                && *card != villain_cards.first
                && *card != villain_cards.second
        })
        .collect();
    let sampled: Vec<Card> = available.choose_multiple(rng, 5).copied().collect();
    [sampled[0], sampled[1], sampled[2], sampled[3], sampled[4]]
}

fn best_of_seven(cards: [Card; 7]) -> u64 {
    let mut best = 0;
    for a in 0..3 {
        for b in (a + 1)..4 {
            for c in (b + 1)..5 {
                for d in (c + 1)..6 {
                    for e in (d + 1)..7 {
                        let score = evaluate_five([cards[a], cards[b], cards[c], cards[d], cards[e]]);
                        if score > best {
                            best = score;
                        }
                    }
                }
            }
        }
    }
    best
}

fn evaluate_five(cards: [Card; 5]) -> u64 {
    let mut rank_counts = [0u8; 15];
    let mut suits = [cards[0].suit; 5];
    let mut values = [0u8; 5];
    for (index, card) in cards.iter().enumerate() {
        rank_counts[card.rank.value() as usize] += 1;
        suits[index] = card.suit;
        values[index] = card.rank.value();
    }
    values.sort_unstable_by(|left, right| right.cmp(left));

    let flush = suits.iter().all(|suit| *suit == suits[0]);
    let straight_high = straight_high(&rank_counts);

    if flush {
        if let Some(high) = straight_high {
            return encode_score(8, [high, 0, 0, 0, 0]);
        }
    }
    let mut groups: Vec<(u8, u8)> = (2..=14)
        .rev()
        .filter_map(|rank| {
            let count = rank_counts[rank as usize];
            if count > 0 {
                Some((count, rank))
            } else {
                None
            }
        })
        .collect();
    groups.sort_unstable_by(|left, right| right.cmp(left));

    if groups[0].0 == 4 {
        return encode_score(7, [groups[0].1, groups[1].1, 0, 0, 0]);
    }
    if groups[0].0 == 3 && groups[1].0 == 2 {
        return encode_score(6, [groups[0].1, groups[1].1, 0, 0, 0]);
    }
    if flush {
        return encode_score(5, values);
    }
    if let Some(high) = straight_high {
        return encode_score(4, [high, 0, 0, 0, 0]);
    }
    if groups[0].0 == 3 {
        let kickers: Vec<u8> = groups
            .iter()
            .filter(|(count, _)| *count == 1)
            .map(|(_, rank)| *rank)
            .collect();
        return encode_score(3, [groups[0].1, kickers[0], kickers[1], 0, 0]);
    }
    if groups[0].0 == 2 && groups[1].0 == 2 {
        let high_pair = groups[0].1.max(groups[1].1);
        let low_pair = groups[0].1.min(groups[1].1);
        let kicker = groups
            .iter()
            .find(|(count, _)| *count == 1)
            .map(|(_, rank)| *rank)
            .unwrap_or(0);
        return encode_score(2, [high_pair, low_pair, kicker, 0, 0]);
    }
    if groups[0].0 == 2 {
        let kickers: Vec<u8> = groups
            .iter()
            .filter(|(count, _)| *count == 1)
            .map(|(_, rank)| *rank)
            .collect();
        return encode_score(1, [groups[0].1, kickers[0], kickers[1], kickers[2], 0]);
    }
    encode_score(0, values)
}

fn straight_high(rank_counts: &[u8; 15]) -> Option<u8> {
    for high in (5..=14).rev() {
        let mut has_straight = true;
        for offset in 0..5 {
            let rank = if high == 5 && offset == 4 {
                14
            } else {
                high - offset
            };
            if rank_counts[rank as usize] == 0 {
                has_straight = false;
                break;
            }
        }
        if has_straight {
            return Some(high);
        }
    }
    None
}

fn encode_score(category: u8, values: [u8; 5]) -> u64 {
    ((category as u64) << 20)
        | ((values[0] as u64) << 16)
        | ((values[1] as u64) << 12)
        | ((values[2] as u64) << 8)
        | ((values[3] as u64) << 4)
        | values[4] as u64
}

fn build_feedback_explanation(
    spot: &TrainingSpot,
    selected: &ActionEvaluation,
    best: &ActionEvaluation,
) -> String {
    if selected.action == best.action {
        return match best.action {
            Action::Raise => format!(
                "Raise is best here. The simulated continue equity is {:.1}% and the position-based range only continues often enough to leave you with about {:.1}% fold equity. That pushes the aggressive line to {:+.2} BB.",
                best.equity_pct, best.fold_equity_pct, best.ev_bb
            ),
            Action::Call => format!(
                "Call is best here. Your simulated equity is {:.1}% against the range, which clears the {:.1}% pot-odds threshold without forcing extra money into a stronger continuing range.",
                best.equity_pct, spot.pot_odds_pct
            ),
            Action::Fold => format!(
                "Fold is best here. The hand only carries about {:.1}% equity against the current range, and that is not enough once pot odds, rake, and realization are accounted for.",
                spot.evaluation_for(Action::Call).equity_pct
            ),
        };
    }

    match best.action {
        Action::Raise => format!(
            "{} gives up EV here. {} The stronger line is Raise because {:.1}% fold equity plus {:.1}% equity when called outperforms your choice by {:.2} BB.",
            selected.action,
            selected.explanation,
            best.fold_equity_pct,
            best.equity_pct,
            best.ev_bb - selected.ev_bb
        ),
        Action::Call => format!(
            "{} misses the best middle ground. {} Calling wins because {:.1}% simulated equity clears the {:.1}% pot-odds threshold, but the hand does not justify the extra investment of a raise.",
            selected.action,
            selected.explanation,
            best.equity_pct,
            spot.pot_odds_pct
        ),
        Action::Fold => format!(
            "{} continues too loose here. {} The hand only shows {:.1}% simulated equity into a {:.1}% pot-odds threshold before realization and rake penalties, so folding preserves more EV.",
            selected.action,
            selected.explanation,
            spot.evaluation_for(Action::Call).equity_pct,
            spot.pot_odds_pct
        ),
    }
}

fn raise_line_reason(
    profile: &HandProfile,
    scenario_kind: ScenarioKind,
    hero_is_ip: bool,
    fold_equity_pct: f32,
    continue_equity_pct: f32,
    stack_bb: f32,
) -> String {
    let blocker_note = if profile.ace_blocker {
        "The ace blocker removes premium continues. "
    } else if profile.king_blocker {
        "The king blocker removes part of villain's strongest region. "
    } else {
        ""
    };
    let position_note = if hero_is_ip {
        "You keep position when called. "
    } else {
        "Taking initiative matters more because you are out of position. "
    };
    let depth_note = if stack_bb <= 40.0 {
        "The shorter stack depth also rewards decisive preflop aggression. "
    } else {
        ""
    };
    let context_note = match scenario_kind {
        ScenarioKind::OpenRaiseFirstIn => "Opening first in is mainly a range construction spot.",
        ScenarioKind::FacingOpen => "This raise is driven by fold equity plus enough backup equity.",
        ScenarioKind::FacingThreeBet => "A 4-bet needs blocker value or a hand strong enough to continue comfortably.",
        ScenarioKind::FacingSqueeze => "Against a squeeze, 4-bets should be tighter and more value-heavy.",
    };
    format!(
        "{}{}{}{} Simulated continue equity is {:.1}% and fold equity is {:.1}%.",
        blocker_note, position_note, depth_note, context_note, continue_equity_pct, fold_equity_pct
    )
}

fn call_line_reason(
    profile: &HandProfile,
    pot_odds_pct: f32,
    equity_pct: f32,
    hero_is_ip: bool,
    stack_bb: f32,
) -> String {
    let hand_note = if profile.pair {
        "Pairs realize equity well."
    } else if profile.suited && profile.connected {
        "Suited connected hands realize equity efficiently."
    } else if profile.suited {
        "Suited hands gain extra realization from flush potential."
    } else {
        "This hand still has enough raw showdown value."
    };
    let position_note = if hero_is_ip {
        "Position also helps."
    } else {
        "Realization is worse out of position, but still acceptable here."
    };
    let depth_note = if stack_bb <= 40.0 {
        "At shorter stacks, the continue threshold is tighter."
    } else {
        ""
    };
    format!(
        "{} {} {} Simulated equity is {:.1}% against a {:.1}% pot-odds requirement.",
        hand_note, position_note, depth_note, equity_pct, pot_odds_pct
    )
}

fn fold_line_reason(profile: &HandProfile, pot_odds_pct: f32, equity_pct: f32) -> String {
    let hand_note = if profile.gap >= 4 {
        "The hand is too disconnected."
    } else if !profile.suited && !profile.pair {
        "The hand lacks enough playability."
    } else {
        "The hand does not clear the continue threshold."
    };
    format!(
        "{} Simulated equity only lands around {:.1}%, which is too low once you compare it to the {:.1}% pot-odds requirement and apply realization penalties.",
        hand_note, equity_pct, pot_odds_pct
    )
}

fn analyze_hand(hole_cards: HoleCards) -> HandProfile {
    let mut values = [hole_cards.first.rank.value(), hole_cards.second.rank.value()];
    values.sort_unstable_by(|left, right| right.cmp(left));
    let high = values[0];
    let low = values[1];
    let pair = high == low;
    let suited = hole_cards.first.suit == hole_cards.second.suit;
    let gap = high.saturating_sub(low).saturating_sub(1);
    let connected = !pair && gap == 0;
    let broadway_count = [high, low].into_iter().filter(|value| *value >= 10).count() as u8;
    let ace_blocker = high == 14 || low == 14;
    let king_blocker = high == 13 || low == 13;
    let wheel_shape = ace_blocker && low <= 5;

    HandProfile {
        high,
        pair,
        suited,
        gap,
        connected,
        broadway_count,
        ace_blocker,
        king_blocker,
        wheel_shape,
    }
}

fn call_realization(
    profile: &HandProfile,
    hero_is_ip: bool,
    scenario_kind: ScenarioKind,
    stack_bb: f32,
) -> f32 {
    let mut realization: f32 = if hero_is_ip { 0.92 } else { 0.83 };
    realization += if profile.suited { 0.03 } else { 0.0 };
    realization += if profile.pair { 0.04 } else { 0.0 };
    realization += if profile.connected { 0.02 } else { 0.0 };
    realization += f32::from(profile.broadway_count) * 0.005;
    realization += if profile.wheel_shape { 0.01 } else { 0.0 };
    realization -= if matches!(scenario_kind, ScenarioKind::FacingThreeBet) {
        0.03
    } else {
        0.0
    };
    realization += if stack_bb <= 40.0 { -0.02 } else { 0.0 };
    realization.clamp(0.72, 1.0)
}

fn adjusted_fold_equity_pct(
    raw_fold_equity_pct: f32,
    profile: &HandProfile,
    scenario_kind: ScenarioKind,
    hero_is_ip: bool,
    stack_bb: f32,
) -> f32 {
    let mut aggression_score = 0.0;
    aggression_score += if profile.pair { 0.42 } else { 0.0 };
    aggression_score += if profile.suited { 0.15 } else { 0.0 };
    aggression_score += if profile.connected { 0.12 } else { 0.0 };
    aggression_score += if profile.ace_blocker { 0.25 } else { 0.0 };
    aggression_score += if profile.king_blocker { 0.08 } else { 0.0 };
    aggression_score += if profile.wheel_shape { 0.08 } else { 0.0 };
    aggression_score += f32::from(profile.broadway_count) * 0.08;
    aggression_score += if hero_is_ip { 0.03 } else { 0.0 };
    aggression_score -= if profile.gap >= 4 { 0.12 } else { 0.0 };
    aggression_score = aggression_score.clamp(0.05, 1.0);

    let depth_modifier = if stack_bb <= 40.0 { 0.92 } else { 1.0 };
    let base_factor = 0.28 + aggression_score * 0.62;
    let cap = match scenario_kind {
        ScenarioKind::OpenRaiseFirstIn => 0.0,
        ScenarioKind::FacingOpen => 46.0,
        ScenarioKind::FacingThreeBet => 32.0,
        ScenarioKind::FacingSqueeze => 26.0,
    };
    if matches!(scenario_kind, ScenarioKind::OpenRaiseFirstIn) {
        0.0
    } else {
        round_to_tenth((raw_fold_equity_pct * base_factor * depth_modifier).clamp(4.0, cap))
    }
}

fn raise_realization(profile: &HandProfile, hero_is_ip: bool, stack_bb: f32) -> f32 {
    let mut realization: f32 = if hero_is_ip { 0.97 } else { 0.90 };
    realization += if profile.suited { 0.02 } else { 0.0 };
    realization += if profile.pair && profile.high >= 10 { 0.02 } else { 0.0 };
    realization -= if !profile.pair && !profile.suited && profile.broadway_count == 0 {
        0.09
    } else {
        0.0
    };
    realization -= if profile.gap >= 4 { 0.04 } else { 0.0 };
    realization += if stack_bb <= 40.0 { 0.01 } else { 0.0 };
    realization.clamp(0.80, 1.02)
}

fn weak_bluff_penalty_bb(profile: &HandProfile, scenario_kind: ScenarioKind, stack_bb: f32) -> f32 {
    let mut penalty = 0.0;
    let hand_is_trashy =
        !profile.pair && !profile.suited && profile.broadway_count == 0 && profile.gap >= 3;
    if hand_is_trashy {
        penalty += match scenario_kind {
            ScenarioKind::OpenRaiseFirstIn => 0.0,
            ScenarioKind::FacingOpen => 0.85,
            ScenarioKind::FacingThreeBet => 1.25,
            ScenarioKind::FacingSqueeze => 1.45,
        };
    }
    if profile.high <= 9 && !profile.ace_blocker && !profile.king_blocker {
        penalty += 0.35;
    }
    if stack_bb > 100.0 && hand_is_trashy {
        penalty += 0.25;
    }
    round_to_cent(penalty)
}

fn sample_open_size<R: Rng + ?Sized>(rng: &mut R, opener: Position, stack_bb: f32) -> f32 {
    let options: &[f32] = match (opener, stack_bb <= 40.0) {
        (Position::Utg | Position::Hj, true) => &[2.2, 2.5],
        (Position::Utg | Position::Hj, false) => &[2.5, 3.0, 3.5],
        (Position::Co, true) => &[2.0, 2.2, 2.5],
        (Position::Co, false) => &[2.3, 2.5, 3.0],
        (Position::Btn, true) => &[2.0, 2.2],
        (Position::Btn, false) => &[2.0, 2.2, 2.5],
        (Position::Sb, true) => &[2.0, 2.5, 3.0],
        (Position::Sb, false) => &[2.5, 3.0, 3.5],
        (Position::Bb, _) => &[2.5],
    };
    *options.choose(rng).expect("open size")
}

fn sample_hero_open_size<R: Rng + ?Sized>(rng: &mut R, hero_position: Position, stack_bb: f32) -> f32 {
    let options: &[f32] = match (hero_position, stack_bb <= 40.0) {
        (Position::Utg, true) => &[2.2, 2.5],
        (Position::Utg, false) => &[2.5, 3.0],
        (Position::Hj | Position::Co, true) => &[2.0, 2.2, 2.5],
        (Position::Hj | Position::Co, false) => &[2.3, 2.5, 3.0],
        (Position::Btn, true) => &[2.0, 2.2],
        (Position::Btn, false) => &[2.0, 2.2, 2.5],
        (Position::Sb, true) => &[2.0, 2.5, 3.0],
        (Position::Sb, false) => &[2.5, 3.0, 3.5],
        _ => &[2.5],
    };
    *options.choose(rng).expect("hero open size")
}

fn sample_three_bet_size<R: Rng + ?Sized>(
    rng: &mut R,
    open_size: f32,
    villain_position: Position,
    hero_position: Position,
    villain_is_ip: bool,
    stack_bb: f32,
) -> f32 {
    let base_multiplier = if villain_is_ip {
        if stack_bb <= 40.0 { 2.8 } else { 3.0 }
    } else if stack_bb <= 40.0 {
        3.4
    } else {
        3.8
    };
    let adjustment = match (villain_position, hero_position) {
        (Position::Sb, Position::Btn) => 0.6,
        (Position::Bb, Position::Btn) => 0.8,
        (Position::Bb, Position::Sb) => 0.4,
        _ => 0.0,
    };
    let jitter = [0.0, 0.5, 1.0].choose(rng).copied().unwrap_or(0.0);
    round_to_half(open_size * base_multiplier + adjustment + jitter)
}

fn sample_squeeze_size<R: Rng + ?Sized>(
    rng: &mut R,
    open_size: f32,
    villain_position: Position,
    hero_position: Position,
    stack_bb: f32,
) -> f32 {
    let base_multiplier = if stack_bb <= 40.0 { 3.8 } else { 4.3 };
    let blind_adjustment = match (villain_position, hero_position) {
        (Position::Sb, Position::Btn) | (Position::Bb, Position::Btn) => 0.8,
        (Position::Bb, Position::Co) => 0.5,
        _ => 0.0,
    };
    let jitter = [0.0, 0.5, 1.0].choose(rng).copied().unwrap_or(0.0);
    round_to_half(open_size * base_multiplier + blind_adjustment + jitter)
}

fn suggest_raise_size(
    scenario_kind: ScenarioKind,
    facing_size_bb: f32,
    hero_position: Position,
    villain_position: Position,
    hero_is_ip: bool,
    stack_bb: f32,
) -> f32 {
    let base = match scenario_kind {
        ScenarioKind::FacingOpen => {
            if hero_is_ip {
                if stack_bb <= 40.0 { facing_size_bb * 2.7 } else { facing_size_bb * 3.1 }
            } else if stack_bb <= 40.0 {
                facing_size_bb * 3.2
            } else {
                facing_size_bb * 3.8
            }
        }
        ScenarioKind::FacingThreeBet => {
            if hero_is_ip {
                if stack_bb <= 40.0 { facing_size_bb * 2.0 } else { facing_size_bb * 2.2 }
            } else if stack_bb <= 40.0 {
                facing_size_bb * 2.15
            } else {
                facing_size_bb * 2.35
            }
        }
        ScenarioKind::FacingSqueeze => {
            if hero_is_ip {
                if stack_bb <= 40.0 { facing_size_bb * 1.95 } else { facing_size_bb * 2.15 }
            } else if stack_bb <= 40.0 {
                facing_size_bb * 2.05
            } else {
                facing_size_bb * 2.25
            }
        }
        ScenarioKind::OpenRaiseFirstIn => facing_size_bb,
    };

    let blind_adjustment = match (hero_position, villain_position) {
        (Position::Sb, _) | (Position::Bb, _) => 0.5,
        (_, Position::Sb) | (_, Position::Bb) => 0.5,
        _ => 0.0,
    };
    round_to_half(base + blind_adjustment)
}

fn random_hole_cards<R: Rng + ?Sized>(rng: &mut R) -> HoleCards {
    let mut deck = full_deck();
    deck.shuffle(rng);
    HoleCards {
        first: deck[0],
        second: deck[1],
    }
}

fn full_deck() -> Vec<Card> {
    let mut deck = Vec::with_capacity(52);
    for rank in Rank::all() {
        for suit in Suit::all() {
            deck.push(Card { rank, suit });
        }
    }
    deck
}

fn combo_percent(combo_count: f32) -> f32 {
    round_to_tenth((combo_count / 1225.0) * 100.0)
}

fn rake_in_bb(pot_bb: f32, rake_pct: f32, stack_depth_bb: f32) -> f32 {
    if rake_pct <= 0.0 {
        return 0.0;
    }
    let cap_bb = if stack_depth_bb <= 40.0 {
        1.0
    } else if stack_depth_bb <= 100.0 {
        2.0
    } else {
        3.0
    };
    (pot_bb * (rake_pct / 100.0)).min(cap_bb)
}

fn round_to_half(value: f32) -> f32 {
    (value * 2.0).round() / 2.0
}

fn round_to_tenth(value: f32) -> f32 {
    (value * 10.0).round() / 10.0
}

fn round_to_cent(value: f32) -> f32 {
    (value * 100.0).round() / 100.0
}

// ============================================================
//  Full Hand Play
// ============================================================

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PostflopAction {
    /// Fraction of pot (e.g. 0.33, 0.67, 1.0)
    Bet(f32),
    AllIn,
    Check,
    Call,
    Fold,
}

impl fmt::Display for PostflopAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PostflopAction::Bet(frac) => write!(f, "Bet {:.0}%", frac * 100.0),
            PostflopAction::AllIn => f.write_str("All-In"),
            PostflopAction::Check => f.write_str("Check"),
            PostflopAction::Call => f.write_str("Call"),
            PostflopAction::Fold => f.write_str("Fold"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct StreetResult {
    pub street: Street,
    pub board: Vec<Card>,
    pub hero_equity_pct: f32,
    pub villain_bet_bb: Option<f32>,
    pub pot_odds_pct: f32,
    pub hero_action: PostflopAction,
    pub best_action: PostflopAction,
    pub is_correct: bool,
    pub ev_chosen_bb: f32,
    pub ev_best_bb: f32,
    pub ev_lost_bb: f32,
    pub hand_strength: String,
    pub explanation: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum FullHandPhase {
    Preflop,
    PostflopPending {
        street: Street,
        villain_bet_bb: Option<f32>,
        hero_equity_pct: f32,
        pot_before_bb: f32,
        hero_stack_bb: f32,
    },
    Complete,
}

pub struct FullHandSession {
    pub preflop_spot: TrainingSpot,
    pub preflop_action: Option<Action>,
    pub villain_range: Vec<WeightedCombo>,
    pub board: Vec<Card>,
    pub pot_bb: f32,
    pub hero_stack_bb: f32,
    pub phase: FullHandPhase,
    pub street_results: Vec<StreetResult>,
}

impl FullHandSession {
    pub fn new(config: TrainingConfig) -> Self {
        let preflop_spot = generate_training_spot(config);
        let villain_range = derive_primary_range(&preflop_spot);
        let stack = preflop_spot.stack_bb;
        Self {
            preflop_spot,
            preflop_action: None,
            villain_range,
            board: vec![],
            pot_bb: 0.0,
            hero_stack_bb: stack,
            phase: FullHandPhase::Preflop,
            street_results: vec![],
        }
    }

    pub fn reset(&mut self, config: TrainingConfig) {
        *self = FullHandSession::new(config);
    }

    pub fn submit_preflop(&mut self, action: Action) {
        if self.preflop_action.is_some() {
            return;
        }
        self.preflop_action = Some(action);
        let spot = &self.preflop_spot;
        match action {
            Action::Fold => {
                self.phase = FullHandPhase::Complete;
                return;
            }
            Action::Call => {
                self.pot_bb = round_to_half(spot.pot_bb + spot.call_cost_bb);
                self.hero_stack_bb -= spot.call_cost_bb;
                // villain keeps their full primary range
            }
            Action::Raise => {
                let hero_invest = spot.raise_to_bb - spot.hero_invested_bb;
                self.pot_bb = round_to_half(
                    spot.pot_bb + hero_invest + (spot.raise_to_bb - spot.facing.size_bb),
                );
                self.hero_stack_bb -= hero_invest;
                self.villain_range = derive_continue_range(spot);
                if self.villain_range.is_empty() {
                    self.phase = FullHandPhase::Complete;
                    return;
                }
            }
        }
        self.advance_street();
    }

    pub fn submit_postflop(&mut self, action: PostflopAction) {
        let phase = self.phase.clone();
        let FullHandPhase::PostflopPending {
            street,
            villain_bet_bb,
            hero_equity_pct,
            pot_before_bb,
            hero_stack_bb,
        } = phase
        else {
            return;
        };

        let check_ev = (hero_equity_pct / 100.0) * pot_before_bb;

        let (ev_chosen, best_action, ev_best) = if let Some(bet) = villain_bet_bb {
            // Facing villain bet: Call vs Fold
            let total_pot = pot_before_bb + 2.0 * bet;
            let call_ev = (hero_equity_pct / 100.0) * total_pot - bet;
            let fold_ev = 0.0_f32;
            let best = if call_ev > fold_ev { PostflopAction::Call } else { PostflopAction::Fold };
            let chosen = if matches!(action, PostflopAction::Call) { call_ev } else { fold_ev };
            (chosen, best, call_ev.max(fold_ev))
        } else {
            // Hero acts first: evaluate all bet sizes + check, pick best
            let sizes: &[f32] = &[0.33, 0.67, 1.0];
            let mut best = PostflopAction::Check;
            let mut best_ev = check_ev;
            for &frac in sizes {
                let ev = postflop_bet_ev(frac, hero_equity_pct, pot_before_bb);
                if ev > best_ev {
                    best_ev = ev;
                    best = PostflopAction::Bet(frac);
                }
            }
            // All-in option when SPR ≤ 4
            if hero_stack_bb > 0.0 && hero_stack_bb / pot_before_bb <= 4.0 {
                let ai_ev = postflop_allin_ev(hero_stack_bb, hero_equity_pct, pot_before_bb);
                if ai_ev > best_ev {
                    best_ev = ai_ev;
                    best = PostflopAction::AllIn;
                }
            }
            let chosen = match action {
                PostflopAction::Bet(frac) => postflop_bet_ev(frac, hero_equity_pct, pot_before_bb),
                PostflopAction::AllIn => postflop_allin_ev(hero_stack_bb, hero_equity_pct, pot_before_bb),
                _ => check_ev,
            };
            (chosen, best, best_ev)
        };

        let pot_odds_pct = villain_bet_bb
            .map(|bet| bet / (pot_before_bb + 2.0 * bet) * 100.0)
            .unwrap_or(0.0);

        let hand_strength = describe_hand_strength(self.preflop_spot.hole_cards, &self.board);
        let is_correct = action == best_action;
        let explanation = build_postflop_explanation(
            action,
            best_action,
            hero_equity_pct,
            pot_odds_pct,
            villain_bet_bb,
            &hand_strength,
            ev_chosen,
            ev_best,
        );

        self.street_results.push(StreetResult {
            street,
            board: self.board.clone(),
            hero_equity_pct,
            villain_bet_bb,
            pot_odds_pct,
            hero_action: action,
            best_action,
            is_correct,
            ev_chosen_bb: round_to_cent(ev_chosen),
            ev_best_bb: round_to_cent(ev_best),
            ev_lost_bb: round_to_cent((ev_best - ev_chosen).max(0.0)),
            hand_strength,
            explanation,
        });

        // Update pot and stack
        match action {
            PostflopAction::Fold => {
                self.phase = FullHandPhase::Complete;
                return;
            }
            PostflopAction::Call => {
                let bet = villain_bet_bb.unwrap_or(0.0);
                self.pot_bb = round_to_half(pot_before_bb + 2.0 * bet);
                self.hero_stack_bb = (hero_stack_bb - bet).max(0.0);
            }
            PostflopAction::Bet(frac) => {
                let hero_bet = round_to_half(pot_before_bb * frac).max(0.5);
                self.pot_bb = round_to_half(pot_before_bb + 2.0 * hero_bet);
                self.hero_stack_bb = (hero_stack_bb - hero_bet).max(0.0);
                self.villain_range = narrow_range_by_strength(
                    &self.villain_range,
                    self.preflop_spot.hole_cards,
                    &self.board,
                    0.50,
                );
            }
            PostflopAction::AllIn => {
                self.pot_bb = round_to_half(pot_before_bb + 2.0 * hero_stack_bb);
                self.hero_stack_bb = 0.0;
                self.villain_range = narrow_range_by_strength(
                    &self.villain_range,
                    self.preflop_spot.hole_cards,
                    &self.board,
                    0.35,
                );
            }
            PostflopAction::Check => {
                // pot and stack unchanged
            }
        }

        if matches!(street, Street::River) {
            self.phase = FullHandPhase::Complete;
        } else {
            self.advance_street();
        }
    }

    fn advance_street(&mut self) {
        let next = match self.board.len() {
            0 => Street::Flop,
            3 => Street::Turn,
            4 => Street::River,
            _ => {
                self.phase = FullHandPhase::Complete;
                return;
            }
        };

        let hero = self.preflop_spot.hole_cards;
        let deck = full_deck();
        let blocked: Vec<Card> =
            self.board.iter().copied().chain([hero.first, hero.second]).collect();
        let available: Vec<Card> =
            deck.into_iter().filter(|c| !blocked.contains(c)).collect();
        let n = if matches!(next, Street::Flop) { 3 } else { 1 };
        let mut rng = rand::thread_rng();
        let new_cards: Vec<Card> =
            available.choose_multiple(&mut rng, n).copied().collect();
        self.board.extend(new_cards);

        let hero_equity =
            simulate_equity_pct_with_board(hero, &self.villain_range, &self.board, 120);

        // Villain bets when they have >55% equity vs hero's hand
        let pot = self.pot_bb;
        let villain_bet_bb = if (100.0 - hero_equity) > 55.0 {
            self.villain_range = narrow_range_by_strength(
                &self.villain_range,
                hero,
                &self.board,
                0.60,
            );
            Some(round_to_half(pot * 0.67).max(0.5))
        } else {
            None
        };

        self.phase = FullHandPhase::PostflopPending {
            street: next,
            villain_bet_bb,
            hero_equity_pct: hero_equity,
            pot_before_bb: pot,
            hero_stack_bb: self.hero_stack_bb,
        };
    }

    pub fn preflop_was_correct(&self) -> bool {
        self.preflop_action
            .map(|a| a == self.preflop_spot.best_action().action)
            .unwrap_or(false)
    }

    pub fn total_ev_lost(&self) -> f32 {
        let preflop_lost = self.preflop_action.map(|chosen| {
            let ev_best = self.preflop_spot.best_action().ev_bb;
            let ev_chosen = self.preflop_spot.evaluation_for(chosen).ev_bb;
            (ev_best - ev_chosen).max(0.0)
        }).unwrap_or(0.0);
        let postflop_lost: f32 = self.street_results.iter().map(|r| r.ev_lost_bb).sum();
        round_to_cent(preflop_lost + postflop_lost)
    }

    pub fn total_mistakes(&self) -> usize {
        let pf = self
            .preflop_action
            .map(|a| a != self.preflop_spot.best_action().action)
            .unwrap_or(false) as usize;
        pf + self.street_results.iter().filter(|r| !r.is_correct).count()
    }
}

// ---- range derivation ----

fn derive_primary_range(spot: &TrainingSpot) -> Vec<WeightedCombo> {
    let tokens: Vec<String> = match spot.scenario_kind {
        ScenarioKind::OpenRaiseFirstIn => vec![],
        ScenarioKind::FacingOpen => {
            chart_book().open_range(spot.villain_position, spot.stack_bb).to_vec()
        }
        ScenarioKind::FacingThreeBet | ScenarioKind::FacingSqueeze => chart_book()
            .three_bet_range(spot.hero_position, spot.villain_position, spot.stack_bb)
            .to_vec(),
    };
    expand_range_to_combos(&tokens, spot.hole_cards)
}

fn derive_continue_range(spot: &TrainingSpot) -> Vec<WeightedCombo> {
    let tokens: Vec<String> = match spot.scenario_kind {
        ScenarioKind::OpenRaiseFirstIn => vec![],
        ScenarioKind::FacingOpen => chart_book()
            .continue_vs_3bet(spot.villain_position, spot.hero_position, spot.stack_bb)
            .to_vec(),
        ScenarioKind::FacingThreeBet | ScenarioKind::FacingSqueeze => chart_book()
            .continue_vs_4bet(spot.hero_position, spot.villain_position, spot.stack_bb)
            .to_vec(),
    };
    expand_range_to_combos(&tokens, spot.hole_cards)
}

// ---- postflop equity with a fixed board ----

fn simulate_equity_pct_with_board(
    hero_cards: HoleCards,
    villain_combos: &[WeightedCombo],
    board: &[Card],
    iterations: usize,
) -> f32 {
    if villain_combos.is_empty() || iterations == 0 {
        return 50.0;
    }
    let deck = full_deck();
    let seed = (hero_cards.first.rank.value() as u64).wrapping_mul(97)
        ^ (hero_cards.second.rank.value() as u64).wrapping_mul(31)
        ^ (board.len() as u64).wrapping_mul(13)
        ^ (villain_combos.len() as u64).wrapping_mul(7);
    let mut rng = StdRng::seed_from_u64(seed);
    let mut equity = 0.0f32;
    let mut count = 0usize;

    for _ in 0..iterations {
        let Some(villain) = weighted_choice(villain_combos, &mut rng) else { break };
        let villain = *villain;

        // skip if villain's cards overlap board or hero
        let vf = villain.cards.first;
        let vs = villain.cards.second;
        if board.contains(&vf)
            || board.contains(&vs)
            || vf == hero_cards.first
            || vf == hero_cards.second
            || vs == hero_cards.first
            || vs == hero_cards.second
        {
            continue;
        }

        let needed = 5 - board.len();
        let available: Vec<Card> = deck
            .iter()
            .copied()
            .filter(|c| {
                !board.contains(c)
                    && *c != hero_cards.first
                    && *c != hero_cards.second
                    && *c != vf
                    && *c != vs
            })
            .collect();

        if available.len() < needed {
            continue;
        }

        let runout: Vec<Card> = available.choose_multiple(&mut rng, needed).copied().collect();
        let mut fb = board.to_vec();
        fb.extend(runout);
        let full = [fb[0], fb[1], fb[2], fb[3], fb[4]];

        let hero_score = best_of_seven([
            hero_cards.first,
            hero_cards.second,
            full[0],
            full[1],
            full[2],
            full[3],
            full[4],
        ]);
        let villain_score = best_of_seven([
            vf, vs, full[0], full[1], full[2], full[3], full[4],
        ]);

        if hero_score > villain_score {
            equity += 1.0;
        } else if hero_score == villain_score {
            equity += 0.5;
        }
        count += 1;
    }

    if count == 0 {
        return 50.0;
    }
    round_to_tenth((equity / count as f32) * 100.0)
}

// ---- range narrowing ----

fn narrow_range_by_strength(
    range: &[WeightedCombo],
    hero_cards: HoleCards,
    board: &[Card],
    keep_fraction: f32,
) -> Vec<WeightedCombo> {
    if range.is_empty() {
        return vec![];
    }
    let mut scored: Vec<(u64, WeightedCombo)> = range
        .iter()
        .filter(|c| {
            let vf = c.cards.first;
            let vs = c.cards.second;
            !board.contains(&vf)
                && !board.contains(&vs)
                && vf != hero_cards.first
                && vf != hero_cards.second
                && vs != hero_cards.first
                && vs != hero_cards.second
        })
        .map(|c| (evaluate_combo_on_board(*c, board), *c))
        .collect();
    scored.sort_unstable_by(|(a, _), (b, _)| b.cmp(a));
    let keep = ((scored.len() as f32 * keep_fraction).ceil() as usize)
        .max(1)
        .min(scored.len());
    scored.into_iter().take(keep).map(|(_, c)| c).collect()
}

fn evaluate_combo_on_board(combo: WeightedCombo, board: &[Card]) -> u64 {
    let all: Vec<Card> =
        [combo.cards.first, combo.cards.second].iter().chain(board).copied().collect();
    match all.len() {
        5 => evaluate_five([all[0], all[1], all[2], all[3], all[4]]),
        6 => {
            let mut best = 0u64;
            for skip in 0..6 {
                let five: Vec<Card> =
                    all.iter().copied().enumerate().filter(|(i, _)| *i != skip).map(|(_, c)| c).collect();
                let s = evaluate_five([five[0], five[1], five[2], five[3], five[4]]);
                if s > best {
                    best = s;
                }
            }
            best
        }
        7 => best_of_seven([all[0], all[1], all[2], all[3], all[4], all[5], all[6]]),
        _ => 0,
    }
}

// ---- hand description ----

fn describe_hand_strength(hero: HoleCards, board: &[Card]) -> String {
    if board.len() < 3 {
        return format!("{}", hero.descriptor());
    }
    let all: Vec<Card> = [hero.first, hero.second].iter().chain(board).copied().collect();
    let score = match all.len() {
        5 => evaluate_five([all[0], all[1], all[2], all[3], all[4]]),
        6 => {
            let mut best = 0u64;
            for skip in 0..6 {
                let five: Vec<Card> = all
                    .iter()
                    .copied()
                    .enumerate()
                    .filter(|(i, _)| *i != skip)
                    .map(|(_, c)| c)
                    .collect();
                let s = evaluate_five([five[0], five[1], five[2], five[3], five[4]]);
                if s > best {
                    best = s;
                }
            }
            best
        }
        7 => best_of_seven([all[0], all[1], all[2], all[3], all[4], all[5], all[6]]),
        _ => 0,
    };
    match (score >> 20) as u8 {
        8 => "Straight Flush".to_owned(),
        7 => "Four of a Kind".to_owned(),
        6 => "Full House".to_owned(),
        5 => "Flush".to_owned(),
        4 => "Straight".to_owned(),
        3 => "Three of a Kind".to_owned(),
        2 => "Two Pair".to_owned(),
        1 => "One Pair".to_owned(),
        _ => "High Card".to_owned(),
    }
}

// ---- postflop EV helpers ----

/// GTO fold frequency = bet / (pot + bet) = frac / (1 + frac), capped at 60%
fn postflop_fold_eq(frac: f32) -> f32 {
    (frac / (1.0 + frac)).min(0.60)
}

pub fn postflop_bet_ev(frac: f32, equity_pct: f32, pot: f32) -> f32 {
    let bet = pot * frac;
    let fold_eq = postflop_fold_eq(frac);
    fold_eq * pot
        + (1.0 - fold_eq) * (equity_pct / 100.0) * (pot + 2.0 * bet)
        - bet
}

pub fn postflop_allin_ev(stack: f32, equity_pct: f32, pot: f32) -> f32 {
    if stack <= 0.0 {
        return 0.0;
    }
    let frac = stack / pot;
    let fold_eq = postflop_fold_eq(frac);
    fold_eq * pot
        + (1.0 - fold_eq) * (equity_pct / 100.0) * (pot + 2.0 * stack)
        - stack
}

pub fn describe_hand_strength_pub(hero: HoleCards, board: &[Card]) -> String {
    describe_hand_strength(hero, board)
}

// ---- postflop explanation ----

fn build_postflop_explanation(
    action: PostflopAction,
    best: PostflopAction,
    equity_pct: f32,
    pot_odds_pct: f32,
    villain_bet: Option<f32>,
    hand_strength: &str,
    ev_chosen: f32,
    ev_best: f32,
) -> String {
    let cost = (ev_best - ev_chosen).max(0.0);
    if villain_bet.is_some() {
        if action == best {
            match best {
                PostflopAction::Call => format!(
                    "Correct call. {hand_strength} ({equity_pct:.1}% equity) clears the {pot_odds_pct:.1}% pot-odds threshold. Call EV: {:+.2} BB.",
                    ev_chosen
                ),
                PostflopAction::Fold => format!(
                    "Correct fold. Only {equity_pct:.1}% equity vs a {pot_odds_pct:.1}% requirement. Best EV: {:+.2} BB.",
                    ev_best
                ),
                _ => String::new(),
            }
        } else {
            match best {
                PostflopAction::Call => format!(
                    "Mistake — folded {hand_strength} with {equity_pct:.1}% equity clearing the {pot_odds_pct:.1}% pot-odds threshold. Best (Call) EV: {:+.2} BB. Cost: {:.2} BB.",
                    ev_best, cost
                ),
                PostflopAction::Fold => format!(
                    "Mistake — called with only {equity_pct:.1}% equity against a {pot_odds_pct:.1}% pot-odds requirement. Fold was {:+.2} BB; call was {:+.2} BB. Cost: {:.2} BB.",
                    ev_best, ev_chosen, cost
                ),
                _ => String::new(),
            }
        }
    } else {
        if action == best {
            match best {
                PostflopAction::Bet(frac) => format!(
                    "Correct bet ({:.0}% pot). {hand_strength} ({equity_pct:.1}% equity) — EV {:+.2} BB.",
                    frac * 100.0, ev_chosen
                ),
                PostflopAction::AllIn => format!(
                    "Correct all-in. {hand_strength} ({equity_pct:.1}% equity) — EV {:+.2} BB.",
                    ev_chosen
                ),
                PostflopAction::Check => format!(
                    "Correct check. With {equity_pct:.1}% equity, checking ({:+.2} BB) is the highest-EV line.",
                    ev_chosen
                ),
                _ => String::new(),
            }
        } else {
            let best_label = match best {
                PostflopAction::Bet(frac) => format!("Bet {:.0}%", frac * 100.0),
                PostflopAction::AllIn => "All-In".to_owned(),
                PostflopAction::Check => "Check".to_owned(),
                _ => String::new(),
            };
            let action_label = match action {
                PostflopAction::Bet(frac) => format!("Bet {:.0}%", frac * 100.0),
                PostflopAction::AllIn => "All-In".to_owned(),
                PostflopAction::Check => "Check".to_owned(),
                _ => String::new(),
            };
            format!(
                "Mistake — {action_label} ({equity_pct:.1}% equity, EV {:+.2} BB) vs best {best_label} (EV {:+.2} BB). Cost: {:.2} BB.",
                ev_chosen, ev_best, cost
            )
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pair_plus_ranges() {
        let parsed = parse_range(&["TT+".to_owned()]);
        assert_eq!(parsed.len(), 5);
        assert_eq!(parsed[0].hand_class.high, Rank::Ace);
        assert_eq!(parsed[4].hand_class.high, Rank::Ten);
    }

    #[test]
    fn parses_suited_dash_ranges() {
        let parsed = parse_range(&["A2s-A5s".to_owned()]);
        assert_eq!(parsed.len(), 4);
        assert_eq!(parsed[0].hand_class.low, Rank::Five);
        assert_eq!(parsed[3].hand_class.low, Rank::Two);
    }

    #[test]
    fn parses_weighted_tokens() {
        let parsed = parse_range(&["A5s-A4s@0.5".to_owned()]);
        assert_eq!(parsed.len(), 2);
        assert!((parsed[0].weight - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn royal_flush_beats_full_house() {
        let royal_flush = best_of_seven([
            Card { rank: Rank::Ace, suit: Suit::Spades },
            Card { rank: Rank::King, suit: Suit::Spades },
            Card { rank: Rank::Queen, suit: Suit::Spades },
            Card { rank: Rank::Jack, suit: Suit::Spades },
            Card { rank: Rank::Ten, suit: Suit::Spades },
            Card { rank: Rank::Two, suit: Suit::Hearts },
            Card { rank: Rank::Two, suit: Suit::Clubs },
        ]);
        let full_house = best_of_seven([
            Card { rank: Rank::Ace, suit: Suit::Spades },
            Card { rank: Rank::Ace, suit: Suit::Hearts },
            Card { rank: Rank::Ace, suit: Suit::Clubs },
            Card { rank: Rank::King, suit: Suit::Spades },
            Card { rank: Rank::King, suit: Suit::Hearts },
            Card { rank: Rank::Two, suit: Suit::Diamonds },
            Card { rank: Rank::Three, suit: Suit::Clubs },
        ]);
        assert!(royal_flush > full_house);
    }

    #[test]
    fn generated_spot_has_evaluations() {
        let spot = generate_training_spot(TrainingConfig::default());
        assert_eq!(spot.evaluations.len(), 3);
        assert!(spot.villain_range_pct > 0.0);
    }

    #[test]
    fn weak_offsuit_trash_does_not_prefer_raise_into_utg() {
        let spot = TrainingSpot {
            title: "BB vs UTG open".to_owned(),
            street: Street::Preflop,
            scenario_kind: ScenarioKind::FacingOpen,
            hero_position: Position::Bb,
            villain_position: Position::Utg,
            opener_position: Some(Position::Utg),
            hole_cards: HoleCards {
                first: Card { rank: Rank::Nine, suit: Suit::Hearts },
                second: Card { rank: Rank::Three, suit: Suit::Clubs },
            },
            board: vec![],
            pot_bb: 4.0,
            stack_bb: 100.0,
            rake_pct: 0.0,
            hero_invested_bb: 1.0,
            call_cost_bb: 2.0,
            raise_to_bb: 10.0,
            pot_odds_pct: 33.3,
            villain_range_pct: 0.0,
            prompt: String::new(),
            facing: FacingAction {
                size_bb: 3.0,
            },
            action_history: vec![PreflopAction {
                actor: Position::Utg,
                kind: PreflopActionKind::OpenRaise,
                size_bb: Some(3.0),
            }],
            evaluations: vec![],
        };
        let solution = solve_spot(&spot);
        let best = solution
            .metrics
            .iter()
            .max_by(|left, right| left.ev_bb.total_cmp(&right.ev_bb))
            .unwrap();
        assert_ne!(best.action, Action::Raise);
    }

    #[test]
    fn rfi_mode_generates_rfi_spots() {
        let spot = generate_training_spot(TrainingConfig {
            training_mode: TrainingMode::RaiseFirstIn,
            ..TrainingConfig::default()
        });
        assert_eq!(spot.scenario_kind, ScenarioKind::OpenRaiseFirstIn);
    }

    #[test]
    fn three_bet_spots_use_hands_that_can_open() {
        for _ in 0..25 {
            let spot = generate_training_spot(TrainingConfig {
                training_mode: TrainingMode::ThreeBetDefense,
                ..TrainingConfig::default()
            });
            assert!(
                hand_weight_in_tokens(
                    spot.hole_cards,
                    chart_book().open_range(spot.hero_position, spot.stack_bb),
                ) > 0.0
            );
        }
    }

    #[test]
    fn squeeze_spots_use_hands_that_can_flat() {
        for _ in 0..25 {
            let spot = generate_training_spot(TrainingConfig {
                training_mode: TrainingMode::SqueezeDefense,
                ..TrainingConfig::default()
            });
            let opener = spot.opener_position.expect("squeeze spot requires opener");
            assert!(
                hand_weight_in_tokens(
                    spot.hole_cards,
                    chart_book().cold_call_range(spot.hero_position, opener, spot.stack_bb),
                ) > 0.0
            );
        }
    }

    #[test]
    fn rake_reduces_raise_ev() {
        let base_spot = generate_training_spot(TrainingConfig::default());
        let no_rake_solution = solve_spot(&TrainingSpot {
            rake_pct: 0.0,
            evaluations: vec![],
            ..base_spot.clone()
        });
        let rake_solution = solve_spot(&TrainingSpot {
            rake_pct: 5.0,
            evaluations: vec![],
            ..base_spot
        });

        let no_rake_raise = no_rake_solution
            .metrics
            .iter()
            .find(|metric| metric.action == Action::Raise)
            .unwrap()
            .ev_bb;
        let rake_raise = rake_solution
            .metrics
            .iter()
            .find(|metric| metric.action == Action::Raise)
            .unwrap()
            .ev_bb;
        assert!(rake_raise <= no_rake_raise);
    }
}
