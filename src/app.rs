use eframe::egui::{
    self, scroll_area::ScrollBarVisibility, Color32, CornerRadius, Margin, RichText, Stroke, Vec2,
};

use crate::model::{
    Action, Card, DecisionFeedback, FullHandPhase, FullHandSession, PostflopAction, StreetResult,
    Street, TrainingMode, TrainingSession,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum AppMode {
    QuickDrill,
    FullHand,
}

pub struct PokerTrainerApp {
    session: TrainingSession,
    full_hand: FullHandSession,
    mode: AppMode,
}

impl PokerTrainerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        configure_theme(&cc.egui_ctx);
        let session = TrainingSession::new();
        let full_hand = FullHandSession::new(session.config);
        Self { session, full_hand, mode: AppMode::QuickDrill }
    }

    fn render_titlebar(&self, ctx: &egui::Context, ui: &mut egui::Ui) {
        egui::Frame::new()
            .fill(Color32::from_rgb(12, 15, 22))
            .stroke(Stroke::new(1.0, Color32::from_rgb(35, 41, 52)))
            .corner_radius(CornerRadius::same(14))
            .inner_margin(Margin::symmetric(14, 10))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    let drag_width = (ui.available_width() - 120.0).max(120.0);
                    let (drag_rect, drag_response) = ui.allocate_exact_size(
                        Vec2::new(drag_width, 26.0),
                        egui::Sense::click_and_drag(),
                    );
                    ui.painter().text(
                        drag_rect.left_center(),
                        egui::Align2::LEFT_CENTER,
                        "Poker Trainer",
                        egui::FontId::proportional(16.0),
                        Color32::from_rgb(240, 236, 224),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let is_fullscreen =
                            ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                        if ui
                            .add(
                                egui::Button::new(RichText::new("X").size(14.0).strong())
                                    .min_size(Vec2::new(34.0, 26.0))
                                    .corner_radius(CornerRadius::same(8))
                                    .fill(Color32::from_rgb(122, 54, 58)),
                            )
                            .clicked()
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        let fs_label = if is_fullscreen { "Window" } else { "Full" };
                        if ui
                            .add(
                                egui::Button::new(RichText::new(fs_label).size(13.0).strong())
                                    .min_size(Vec2::new(74.0, 26.0))
                                    .corner_radius(CornerRadius::same(8))
                                    .fill(Color32::from_rgb(46, 74, 117)),
                            )
                            .clicked()
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(
                                !is_fullscreen,
                            ));
                        }
                    });
                    if drag_response.double_clicked() {
                        let fs =
                            ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
                        ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fs));
                    } else if drag_response.dragged() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                });
            });
    }

    fn render_header(&self, ui: &mut egui::Ui) {
        panel_frame().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new("Poker Trainer")
                            .size(28.0)
                            .strong()
                            .color(Color32::from_rgb(240, 236, 224)),
                    );
                    ui.label(
                        RichText::new(
                            "Randomized preflop scenarios with computed EV, equity, and pot odds",
                        )
                        .size(14.0)
                        .color(Color32::from_rgb(156, 167, 181)),
                    );
                });
                let available = ui.available_width();
                ui.allocate_ui_with_layout(
                    Vec2::new(available, 60.0),
                    egui::Layout::right_to_left(egui::Align::Center),
                    |ui| {
                        ui.spacing_mut().item_spacing.x = 12.0;
                        stat_chip(
                            ui,
                            "Hands",
                            self.session.answered_count.to_string(),
                            Color32::from_rgb(226, 174, 76),
                        );
                        stat_chip(
                            ui,
                            "Stack",
                            format!("{:.0} BB", self.session.config.stack_depth_bb),
                            Color32::from_rgb(101, 154, 214),
                        );
                        stat_chip(
                            ui,
                            "Accuracy",
                            format!("{:.0}%", self.session.accuracy_pct()),
                            Color32::from_rgb(92, 181, 144),
                        );
                    },
                );
            });
        });
    }

    fn render_spot_card(&self, ui: &mut egui::Ui) {
        let spot = self.session.current_spot();
        panel_frame().show(ui, |ui| {
            ui.vertical(|ui| {
                ui.label(
                    RichText::new(&spot.title)
                        .size(22.0)
                        .strong()
                        .color(Color32::from_rgb(240, 236, 224)),
                );
                ui.add_space(8.0);
                ui.horizontal_wrapped(|ui| {
                    badge(ui, format!("Street: {}", spot.street), "#184e4a");
                    badge(ui, format!("Spot: {}", spot.scenario_kind), "#543c63");
                    badge(ui, format!("Hero: {}", spot.hero_position), "#684b28");
                    if let Some(opener) = spot.opener_position {
                        badge(ui, format!("Opener: {}", opener), "#4b5d33");
                    }
                    badge(ui, format!("Villain range: {:.0}%", spot.villain_range_pct), "#2d405f");
                    badge(ui, format!("Pot odds: {:.1}%", spot.pot_odds_pct), "#36506f");
                    badge(ui, format!("Rake: {:.1}%", spot.rake_pct), "#5d4d2f");
                });
                ui.add_space(14.0);
                ui.label(
                    RichText::new(spot.action_history_summary())
                        .size(20.0)
                        .color(Color32::from_rgb(225, 228, 235)),
                );
                ui.label(
                    RichText::new(format!(
                        "Pot: {:.1} BB | Call: {:.1} BB | Raise to: {:.1} BB | Stack: {:.0} BB",
                        spot.pot_bb, spot.call_cost_bb, spot.raise_to_bb, spot.stack_bb
                    ))
                    .size(14.0)
                    .color(Color32::from_rgb(145, 154, 168)),
                );
                ui.add_space(14.0);
                ui.label(
                    RichText::new(&spot.prompt)
                        .size(16.0)
                        .color(Color32::from_rgb(190, 197, 210)),
                );
            });
        });
    }

    fn render_table_and_actions(&mut self, ui: &mut egui::Ui) {
        ui.columns(2, |columns| {
            panel_frame().show(&mut columns[0], |ui| {
                self.render_settings(ui);
                ui.add_space(18.0);
                ui.separator();
                ui.add_space(14.0);
                ui.label(
                    RichText::new("Hero Hand")
                        .size(18.0)
                        .strong()
                        .color(Color32::from_rgb(240, 236, 224)),
                );
                ui.add_space(16.0);
                let hand = self.session.current_spot().hole_cards;
                ui.horizontal(|ui| {
                    render_card(ui, hand.first);
                    render_card(ui, hand.second);
                });
                ui.add_space(14.0);
                ui.label(
                    RichText::new(hand.descriptor())
                        .size(18.0)
                        .color(Color32::from_rgb(220, 225, 231)),
                );
                ui.add_space(18.0);
                ui.label(
                    RichText::new("Board State")
                        .size(17.0)
                        .strong()
                        .color(Color32::from_rgb(240, 236, 224)),
                );
                ui.add_space(8.0);
                let board = &self.session.current_spot().board;
                if board.is_empty() {
                    ghost_box(
                        ui,
                        "No community cards yet. This panel is reserved for the flop, turn, and river modules.",
                    );
                } else {
                    ui.horizontal_wrapped(|ui| {
                        for card in board {
                            render_card(ui, *card);
                        }
                    });
                }
                ui.add_space(18.0);
                ui.separator();
                ui.add_space(14.0);
                ui.label(
                    RichText::new("Street Roadmap")
                        .size(17.0)
                        .strong()
                        .color(Color32::from_rgb(240, 236, 224)),
                );
                ui.add_space(8.0);
                for street in [Street::Preflop, Street::Flop, Street::Turn, Street::River] {
                    let ready = matches!(street, Street::Preflop);
                    let tone = if ready { "#2f6c52" } else { "#30384a" };
                    let text = if ready { "Dynamic" } else { "Reserved" };
                    ui.horizontal(|ui| {
                        badge(ui, street.to_string(), tone);
                        ui.label(
                            RichText::new(text)
                                .size(14.0)
                                .color(Color32::from_rgb(163, 171, 184)),
                        );
                    });
                    ui.add_space(6.0);
                }
            });

            panel_frame().show(&mut columns[1], |ui| {
                ui.label(
                    RichText::new("Choose Action")
                        .size(18.0)
                        .strong()
                        .color(Color32::from_rgb(240, 236, 224)),
                );
                ui.add_space(8.0);
                ui.label(
                    RichText::new("Raise means 3-bet or 4-bet depending on the spot.")
                        .size(14.0)
                        .color(Color32::from_rgb(150, 160, 173)),
                );
                ui.add_space(16.0);

                let answered = self.session.current_feedback.is_some();
                ui.horizontal(|ui| {
                    for action in [Action::Raise, Action::Call, Action::Fold] {
                        let button = egui::Button::new(
                            RichText::new(action.to_string()).size(17.0).strong(),
                        )
                        .min_size(Vec2::new(120.0, 46.0))
                        .corner_radius(CornerRadius::same(12))
                        .fill(button_fill(action))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(92, 98, 112)));
                        if ui.add_enabled(!answered, button).clicked() {
                            self.session.answer_current(action);
                        }
                    }
                });

                ui.add_space(20.0);
                match self.session.current_feedback.clone() {
                    Some(feedback) => self.render_feedback(ui, &feedback),
                    None => ghost_box(
                        ui,
                        "Make a decision first. Feedback will show the chosen EV, the best EV, your estimated equity, fold equity, and the pot-odds threshold.",
                    ),
                }
            });
        });
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(
            RichText::new("Trainer Settings")
                .size(18.0)
                .strong()
                .color(Color32::from_rgb(240, 236, 224)),
        );
        ui.add_space(10.0);

        let mut config = self.session.config;
        let mut changed = false;

        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Mode").size(14.0).color(Color32::from_rgb(180, 188, 199)));
            egui::ComboBox::from_id_salt("training_mode")
                .selected_text(config.training_mode.label())
                .show_ui(ui, |ui| {
                    changed |= ui
                        .selectable_value(&mut config.training_mode, TrainingMode::Mixed, "Mixed")
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut config.training_mode,
                            TrainingMode::RaiseFirstIn,
                            "RFI",
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut config.training_mode,
                            TrainingMode::OpenDefense,
                            "Vs Open",
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut config.training_mode,
                            TrainingMode::ThreeBetDefense,
                            "Vs 3-Bet",
                        )
                        .changed();
                    changed |= ui
                        .selectable_value(
                            &mut config.training_mode,
                            TrainingMode::SqueezeDefense,
                            "Vs Squeeze",
                        )
                        .changed();
                });
        });

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Stack").size(14.0).color(Color32::from_rgb(180, 188, 199)));
            for depth in [20.0_f32, 40.0, 60.0, 100.0, 150.0, 200.0] {
                let selected = (config.stack_depth_bb - depth).abs() < f32::EPSILON;
                if ui
                    .selectable_label(selected, format!("{:.0} BB", depth))
                    .clicked()
                {
                    config.stack_depth_bb = depth;
                    changed = true;
                }
            }
        });

        ui.add_space(8.0);
        ui.horizontal_wrapped(|ui| {
            ui.label(RichText::new("Rake").size(14.0).color(Color32::from_rgb(180, 188, 199)));
            for rake in [0.0_f32, 2.5, 5.0] {
                let selected = (config.rake_pct - rake).abs() < f32::EPSILON;
                if ui.selectable_label(selected, format!("{:.1}%", rake)).clicked() {
                    config.rake_pct = rake;
                    changed = true;
                }
            }
        });

        if changed {
            self.session.apply_config(config);
        }
    }

    fn render_feedback(&mut self, ui: &mut egui::Ui, feedback: &DecisionFeedback) {
        let accent =
            if feedback.is_correct { Color32::from_rgb(92, 181, 144) } else { Color32::from_rgb(220, 113, 97) };
        egui::Frame::new()
            .fill(Color32::from_rgb(26, 31, 40))
            .stroke(Stroke::new(1.0, accent))
            .corner_radius(CornerRadius::same(16))
            .inner_margin(Margin::same(18))
            .show(ui, |ui| {
                ui.label(
                    RichText::new(if feedback.is_correct { "Correct" } else { "Incorrect" })
                        .size(22.0)
                        .strong()
                        .color(accent),
                );
                ui.add_space(12.0);
                ui.horizontal_wrapped(|ui| {
                    badge(ui, format!("Your action: {}", feedback.selected_action), "#2d405f");
                    badge(ui, format!("Best action: {}", feedback.correct_action), "#184e4a");
                    badge(ui, format!("Best EV: {:+.2} BB", feedback.correct_ev_bb), "#684b28");
                    badge(ui, format!("Pot odds: {:.1}%", feedback.pot_odds_pct), "#36506f");
                });
                ui.add_space(14.0);
                ui.label(
                    RichText::new(format!(
                        "Selected EV: {:+.2} BB | Selected equity: {:.1}% | Selected fold equity: {:.1}%",
                        feedback.selected_ev_bb,
                        feedback.selected_equity_pct,
                        feedback.selected_fold_equity_pct
                    ))
                    .size(15.0)
                    .color(Color32::from_rgb(202, 209, 221)),
                );
                ui.label(
                    RichText::new(format!(
                        "Best equity: {:.1}% | Best fold equity: {:.1}%",
                        feedback.correct_equity_pct, feedback.correct_fold_equity_pct
                    ))
                    .size(15.0)
                    .color(Color32::from_rgb(202, 209, 221)),
                );
                ui.add_space(10.0);
                ui.label(
                    RichText::new(&feedback.explanation)
                        .size(15.0)
                        .color(Color32::from_rgb(216, 221, 228)),
                );
            });

        ui.add_space(16.0);
        ui.horizontal(|ui| {
            if ui
                .add(
                    egui::Button::new(RichText::new("Next Hand").size(16.0).strong())
                        .min_size(Vec2::new(150.0, 44.0))
                        .corner_radius(CornerRadius::same(12))
                        .fill(Color32::from_rgb(71, 109, 196)),
                )
                .clicked()
            {
                self.session.next_spot();
            }
            if ui
                .add(
                    egui::Button::new(RichText::new("Reset Score").size(16.0).strong())
                        .min_size(Vec2::new(150.0, 44.0))
                        .corner_radius(CornerRadius::same(12))
                        .fill(Color32::from_rgb(74, 84, 98)),
                )
                .clicked()
            {
                self.session.restart();
            }
        });
    }

    // ================================================================
    //  Full Hand Mode
    // ================================================================

    fn render_full_hand(&mut self, ui: &mut egui::Ui) {
        match self.full_hand.phase.clone() {
            FullHandPhase::Preflop => self.render_fh_preflop(ui),
            FullHandPhase::PostflopPending { street, villain_bet_bb, hero_equity_pct, pot_before_bb, hero_stack_bb } => {
                self.render_fh_postflop(ui, street, villain_bet_bb, hero_equity_pct, pot_before_bb, hero_stack_bb);
            }
            FullHandPhase::Complete => self.render_fh_summary(ui),
        }
    }

    fn render_fh_preflop(&mut self, ui: &mut egui::Ui) {
        let spot = &self.full_hand.preflop_spot;

        panel_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new(&spot.title)
                        .size(22.0)
                        .strong()
                        .color(Color32::from_rgb(240, 236, 224)),
                );
                badge(ui, "Full Hand Mode".to_owned(), "#2d3f5c");
            });
            ui.add_space(8.0);
            ui.horizontal_wrapped(|ui| {
                badge(ui, format!("Hero: {}", spot.hero_position), "#684b28");
                badge(ui, format!("Villain range: {:.0}%", spot.villain_range_pct), "#2d405f");
                badge(ui, format!("Pot odds: {:.1}%", spot.pot_odds_pct), "#36506f");
                badge(ui, format!("Stack: {:.0} BB", spot.stack_bb), "#303a50");
            });
            ui.add_space(12.0);
            ui.label(
                RichText::new(spot.action_history_summary())
                    .size(18.0)
                    .color(Color32::from_rgb(225, 228, 235)),
            );
            ui.label(
                RichText::new(format!(
                    "Pot: {:.1} BB  |  Call: {:.1} BB  |  Raise to: {:.1} BB",
                    spot.pot_bb, spot.call_cost_bb, spot.raise_to_bb
                ))
                .size(14.0)
                .color(Color32::from_rgb(145, 154, 168)),
            );
        });

        ui.add_space(14.0);

        ui.columns(2, |cols| {
            panel_frame().show(&mut cols[0], |ui| {
                ui.label(
                    RichText::new("Hero Hand")
                        .size(18.0)
                        .strong()
                        .color(Color32::from_rgb(240, 236, 224)),
                );
                ui.add_space(12.0);
                let hand = self.full_hand.preflop_spot.hole_cards;
                ui.horizontal(|ui| {
                    render_card(ui, hand.first);
                    render_card(ui, hand.second);
                });
                ui.add_space(8.0);
                ui.label(
                    RichText::new(hand.descriptor())
                        .size(17.0)
                        .color(Color32::from_rgb(220, 225, 231)),
                );
            });

            panel_frame().show(&mut cols[1], |ui| {
                ui.label(
                    RichText::new("Preflop Decision")
                        .size(18.0)
                        .strong()
                        .color(Color32::from_rgb(240, 236, 224)),
                );
                ui.add_space(6.0);
                ui.label(
                    RichText::new("Your choice is recorded. Feedback appears at the hand review.")
                        .size(13.0)
                        .color(Color32::from_rgb(150, 160, 173)),
                );
                ui.add_space(14.0);
                let already_answered = self.full_hand.preflop_action.is_some();
                ui.horizontal(|ui| {
                    for action in [Action::Raise, Action::Call, Action::Fold] {
                        let btn = egui::Button::new(
                            RichText::new(action.to_string()).size(17.0).strong(),
                        )
                        .min_size(Vec2::new(110.0, 46.0))
                        .corner_radius(CornerRadius::same(12))
                        .fill(button_fill(action))
                        .stroke(Stroke::new(1.0, Color32::from_rgb(92, 98, 112)));
                        if ui.add_enabled(!already_answered, btn).clicked() {
                            self.full_hand.submit_preflop(action);
                        }
                    }
                });
            });
        });
    }

    fn render_fh_postflop(
        &mut self,
        ui: &mut egui::Ui,
        street: Street,
        villain_bet_bb: Option<f32>,
        hero_equity_pct: f32,
        pot_before_bb: f32,
        hero_stack_bb: f32,
    ) {
        // Header strip
        panel_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                badge(ui, street.to_string(), "#2f6c52");
                badge(ui, format!("Pot: {:.1} BB", pot_before_bb), "#303a50");
                badge(
                    ui,
                    if villain_bet_bb.is_some() {
                        format!("Villain bet {:.1} BB", villain_bet_bb.unwrap())
                    } else {
                        "Villain checks".to_owned()
                    },
                    if villain_bet_bb.is_some() { "#5e3a22" } else { "#2e4d32" },
                );
                badge(ui, format!("Your equity: {:.1}%", hero_equity_pct), "#2d405f");
            });
            if let Some(bet) = villain_bet_bb {
                let pot_odds = bet / (pot_before_bb + 2.0 * bet) * 100.0;
                ui.add_space(4.0);
                ui.label(
                    RichText::new(format!(
                        "Pot odds: {:.1}%  —  you need >{:.1}% equity to call profitably",
                        pot_odds, pot_odds
                    ))
                    .size(14.0)
                    .color(Color32::from_rgb(190, 197, 210)),
                );
            }
        });

        ui.add_space(14.0);

        ui.columns(2, |cols| {
            // Left: hand + board
            panel_frame().show(&mut cols[0], |ui| {
                ui.label(
                    RichText::new("Hero Hand")
                        .size(17.0)
                        .strong()
                        .color(Color32::from_rgb(240, 236, 224)),
                );
                ui.add_space(10.0);
                let hand = self.full_hand.preflop_spot.hole_cards;
                ui.horizontal(|ui| {
                    render_card(ui, hand.first);
                    render_card(ui, hand.second);
                });
                ui.add_space(14.0);
                ui.label(
                    RichText::new("Board")
                        .size(17.0)
                        .strong()
                        .color(Color32::from_rgb(240, 236, 224)),
                );
                ui.add_space(10.0);
                ui.horizontal_wrapped(|ui| {
                    for card in &self.full_hand.board {
                        render_card(ui, *card);
                    }
                });
                ui.add_space(8.0);
                let hs = crate::model::describe_hand_strength_pub(hand, &self.full_hand.board);
                ui.label(
                    RichText::new(hs)
                        .size(16.0)
                        .color(Color32::from_rgb(226, 174, 76)),
                );
            });

            // Right: action buttons
            panel_frame().show(&mut cols[1], |ui| {
                ui.label(
                    RichText::new(if villain_bet_bb.is_some() {
                        "Facing a Bet"
                    } else {
                        "You Act First"
                    })
                    .size(18.0)
                    .strong()
                    .color(Color32::from_rgb(240, 236, 224)),
                );
                ui.add_space(6.0);
                let subtitle = if let Some(bet) = villain_bet_bb {
                    format!(
                        "Villain bet {:.1} BB into a {:.1} BB pot.",
                        bet, pot_before_bb
                    )
                } else {
                    format!(
                        "Villain checks. Your hand has {:.1}% equity.",
                        hero_equity_pct
                    )
                };
                ui.label(
                    RichText::new(subtitle).size(14.0).color(Color32::from_rgb(150, 160, 173)),
                );
                ui.add_space(16.0);
                if let Some(bet) = villain_bet_bb {
                    // Facing a bet: Call / Fold
                    ui.horizontal(|ui| {
                        for (action, fill, label) in [
                            (PostflopAction::Call, Color32::from_rgb(44, 121, 91),
                             format!("Call {:.1}bb", bet)),
                            (PostflopAction::Fold, Color32::from_rgb(122, 54, 58),
                             "Fold".to_owned()),
                        ] {
                            let btn = egui::Button::new(RichText::new(&label).size(17.0).strong())
                                .min_size(Vec2::new(120.0, 46.0))
                                .corner_radius(CornerRadius::same(12))
                                .fill(fill)
                                .stroke(Stroke::new(1.0, Color32::from_rgb(92, 98, 112)));
                            if ui.add(btn).clicked() {
                                self.full_hand.submit_postflop(action);
                            }
                        }
                    });
                } else {
                    // Hero acts first: multiple bet sizes + check
                    let spr = if pot_before_bb > 0.0 { hero_stack_bb / pot_before_bb } else { 99.0 };
                    ui.horizontal_wrapped(|ui| {
                        // Bet sizes: 33%, 67%, 100%
                        let sizes: &[(f32, &str)] = &[
                            (0.33, "33%"),
                            (0.67, "67%"),
                            (1.0,  "Pot"),
                        ];
                        for &(frac, pct_label) in sizes {
                            let bet_bb = (pot_before_bb * frac).max(0.5);
                            if bet_bb >= hero_stack_bb && hero_stack_bb > 0.0 {
                                continue; // skip if sizing would exceed stack (show all-in instead)
                            }
                            let label = format!("Bet {:.1}bb ({})", bet_bb, pct_label);
                            // Shade darker for smaller bets, brighter for larger
                            let purple = if frac <= 0.33 {
                                Color32::from_rgb(85, 45, 115)
                            } else if frac <= 0.67 {
                                Color32::from_rgb(105, 58, 140)
                            } else {
                                Color32::from_rgb(125, 72, 162)
                            };
                            let btn = egui::Button::new(RichText::new(&label).size(15.0).strong())
                                .min_size(Vec2::new(130.0, 44.0))
                                .corner_radius(CornerRadius::same(12))
                                .fill(purple)
                                .stroke(Stroke::new(1.0, Color32::from_rgb(92, 98, 112)));
                            if ui.add(btn).clicked() {
                                self.full_hand.submit_postflop(PostflopAction::Bet(frac));
                            }
                        }
                        // All-in: show when SPR ≤ 4 or stack is small
                        if hero_stack_bb > 0.0 && (spr <= 4.0 || hero_stack_bb < pot_before_bb) {
                            let label = format!("All-In {:.1}bb", hero_stack_bb);
                            let btn = egui::Button::new(RichText::new(&label).size(15.0).strong())
                                .min_size(Vec2::new(130.0, 44.0))
                                .corner_radius(CornerRadius::same(12))
                                .fill(Color32::from_rgb(170, 50, 50))
                                .stroke(Stroke::new(1.0, Color32::from_rgb(92, 98, 112)));
                            if ui.add(btn).clicked() {
                                self.full_hand.submit_postflop(PostflopAction::AllIn);
                            }
                        }
                        // Check
                        let btn = egui::Button::new(RichText::new("Check").size(15.0).strong())
                            .min_size(Vec2::new(100.0, 44.0))
                            .corner_radius(CornerRadius::same(12))
                            .fill(Color32::from_rgb(74, 84, 98))
                            .stroke(Stroke::new(1.0, Color32::from_rgb(92, 98, 112)));
                        if ui.add(btn).clicked() {
                            self.full_hand.submit_postflop(PostflopAction::Check);
                        }
                    });
                }
                ui.add_space(18.0);

                // Progress tracker
                ui.label(
                    RichText::new("Hand Progress")
                        .size(14.0)
                        .strong()
                        .color(Color32::from_rgb(180, 188, 199)),
                );
                ui.add_space(6.0);
                let streets = [Street::Preflop, Street::Flop, Street::Turn, Street::River];
                let completed_streets: Vec<Street> = self
                    .full_hand
                    .street_results
                    .iter()
                    .map(|r| r.street)
                    .collect();
                let preflop_done = self.full_hand.preflop_action.is_some();
                for s in streets {
                    let done = if matches!(s, Street::Preflop) {
                        preflop_done
                    } else {
                        completed_streets.contains(&s)
                    };
                    let active = match s {
                        Street::Preflop => false,
                        _ => s == street,
                    };
                    let tone = if active {
                        "#2f6c52"
                    } else if done {
                        "#3d5c3a"
                    } else {
                        "#30384a"
                    };
                    ui.horizontal(|ui| {
                        badge(ui, s.to_string(), tone);
                        if active {
                            ui.label(
                                RichText::new("← current")
                                    .size(13.0)
                                    .color(Color32::from_rgb(92, 181, 144)),
                            );
                        } else if done {
                            let sr = self.full_hand.street_results.iter().find(|r| r.street == s);
                            if let Some(r) = sr {
                                let icon = if r.is_correct { "✓" } else { "✗" };
                                let col = if r.is_correct {
                                    Color32::from_rgb(92, 181, 144)
                                } else {
                                    Color32::from_rgb(220, 113, 97)
                                };
                                ui.label(RichText::new(icon).size(13.0).color(col));
                            } else if matches!(s, Street::Preflop) {
                                let correct = self.full_hand.preflop_was_correct();
                                let (icon, col) = if correct {
                                    ("✓", Color32::from_rgb(92, 181, 144))
                                } else {
                                    ("✗", Color32::from_rgb(220, 113, 97))
                                };
                                ui.label(RichText::new(icon).size(13.0).color(col));
                            }
                        }
                    });
                    ui.add_space(4.0);
                }
            });
        });
    }

    fn render_fh_summary(&mut self, ui: &mut egui::Ui) {
        let mistakes = self.full_hand.total_mistakes();
        let ev_lost = self.full_hand.total_ev_lost();
        let accent = if mistakes == 0 {
            Color32::from_rgb(92, 181, 144)
        } else if mistakes == 1 {
            Color32::from_rgb(226, 174, 76)
        } else {
            Color32::from_rgb(220, 113, 97)
        };

        // ---- top verdict ----
        panel_frame().show(ui, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.label(
                    RichText::new("Hand Review")
                        .size(26.0)
                        .strong()
                        .color(Color32::from_rgb(240, 236, 224)),
                );
                badge(
                    ui,
                    format!(
                        "{} mistake{}",
                        mistakes,
                        if mistakes == 1 { "" } else { "s" }
                    ),
                    if mistakes == 0 { "#2f6c52" } else { "#5e3a22" },
                );
                badge(
                    ui,
                    format!("Total EV lost: {:+.2} BB", -ev_lost),
                    if ev_lost < 0.1 { "#2f6c52" } else { "#5e3a22" },
                );
            });
            ui.add_space(6.0);
            ui.label(
                RichText::new(if mistakes == 0 {
                    "Perfect hand — every decision was optimal.".to_owned()
                } else {
                    format!(
                        "You made {} decision{} that cost {:.2} BB in expectation. Review each street below.",
                        mistakes,
                        if mistakes == 1 { "" } else { "s" },
                        ev_lost
                    )
                })
                .size(15.0)
                .color(Color32::from_rgb(190, 197, 210)),
            );
        });

        ui.add_space(14.0);

        // ---- preflop row ----
        let spot = &self.full_hand.preflop_spot;
        let pf_action = self.full_hand.preflop_action.unwrap_or(Action::Fold);
        let pf_best = spot.best_action().action;
        let pf_correct = pf_action == pf_best;
        let pf_ev_chosen = spot.evaluation_for(pf_action).ev_bb;
        let pf_ev_best = spot.best_action().ev_bb;
        let pf_ev_lost = (pf_ev_best - pf_ev_chosen).max(0.0);

        let pf_accent = if pf_correct {
            Color32::from_rgb(92, 181, 144)
        } else {
            Color32::from_rgb(220, 113, 97)
        };

        egui::Frame::new()
            .fill(Color32::from_rgb(20, 24, 33))
            .stroke(Stroke::new(1.5, pf_accent))
            .corner_radius(CornerRadius::same(14))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    badge(ui, "Preflop".to_owned(), "#543c63");
                    badge(
                        ui,
                        format!("Your hand: {}", spot.hole_cards.descriptor()),
                        "#30384a",
                    );
                    badge(ui, format!("You: {}", pf_action), "#2d405f");
                    badge(ui, format!("Best: {}", pf_best), "#184e4a");
                    badge(ui, format!("EV chosen: {:+.2} BB", pf_ev_chosen), "#303a50");
                    badge(ui, format!("EV best: {:+.2} BB", pf_ev_best), "#303a50");
                    if !pf_correct {
                        badge(ui, format!("Cost: -{:.2} BB", pf_ev_lost), "#5e3a22");
                    }
                });
                ui.add_space(8.0);
                ui.label(
                    RichText::new(spot.evaluation_for(pf_best).explanation.clone())
                        .size(14.0)
                        .color(Color32::from_rgb(200, 207, 218)),
                );
                if !pf_correct {
                    ui.add_space(4.0);
                    ui.label(
                        RichText::new(format!(
                            "You chose {} (EV {:+.2} BB) but {} was correct (EV {:+.2} BB). {}",
                            pf_action,
                            pf_ev_chosen,
                            pf_best,
                            pf_ev_best,
                            spot.evaluation_for(pf_action).explanation.clone()
                        ))
                        .size(14.0)
                        .color(Color32::from_rgb(220, 113, 97)),
                    );
                }
            });

        ui.add_space(10.0);

        // ---- postflop rows ----
        let results: Vec<StreetResult> = self.full_hand.street_results.clone();
        for result in &results {
            let row_accent = if result.is_correct {
                Color32::from_rgb(92, 181, 144)
            } else {
                Color32::from_rgb(220, 113, 97)
            };

            egui::Frame::new()
                .fill(Color32::from_rgb(20, 24, 33))
                .stroke(Stroke::new(1.5, row_accent))
                .corner_radius(CornerRadius::same(14))
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        badge(ui, result.street.to_string(), street_color(result.street));
                        // Board cards inline
                        ui.add_space(4.0);
                        for card in &result.board {
                            let col = color_from_hex(card.suit.color_hex());
                            egui::Frame::new()
                                .fill(Color32::from_rgb(245, 241, 233))
                                .corner_radius(CornerRadius::same(6))
                                .inner_margin(Margin::symmetric(6, 3))
                                .show(ui, |ui| {
                                    ui.label(
                                        RichText::new(card.label())
                                            .size(13.0)
                                            .strong()
                                            .color(col),
                                    );
                                });
                        }
                        ui.add_space(4.0);
                        badge(
                            ui,
                            result.hand_strength.clone(),
                            "#303a50",
                        );
                        badge(
                            ui,
                            format!("Equity: {:.1}%", result.hero_equity_pct),
                            "#2d405f",
                        );
                        if let Some(bet) = result.villain_bet_bb {
                            badge(
                                ui,
                                format!("Villain bet {:.1} BB", bet),
                                "#5e3a22",
                            );
                            badge(
                                ui,
                                format!("Pot odds: {:.1}%", result.pot_odds_pct),
                                "#36506f",
                            );
                        } else {
                            badge(ui, "Villain checks".to_owned(), "#2e4d32");
                        }
                        badge(ui, format!("You: {}", result.hero_action), "#2d405f");
                        badge(ui, format!("Best: {}", result.best_action), "#184e4a");
                        badge(
                            ui,
                            format!("EV chosen: {:+.2} BB", result.ev_chosen_bb),
                            "#303a50",
                        );
                        badge(
                            ui,
                            format!("EV best: {:+.2} BB", result.ev_best_bb),
                            "#303a50",
                        );
                        if !result.is_correct {
                            badge(
                                ui,
                                format!("Cost: -{:.2} BB", result.ev_lost_bb),
                                "#5e3a22",
                            );
                        }
                    });
                    ui.add_space(8.0);
                    ui.label(
                        RichText::new(&result.explanation)
                            .size(14.0)
                            .color(Color32::from_rgb(200, 207, 218)),
                    );
                });

            ui.add_space(10.0);
        }

        // ---- total summary ----
        egui::Frame::new()
            .fill(Color32::from_rgb(17, 21, 30))
            .stroke(Stroke::new(1.5, accent))
            .corner_radius(CornerRadius::same(14))
            .inner_margin(Margin::same(16))
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    ui.label(
                        RichText::new("Session Summary")
                            .size(17.0)
                            .strong()
                            .color(Color32::from_rgb(240, 236, 224)),
                    );
                    ui.add_space(12.0);
                    stat_chip(
                        ui,
                        "Mistakes",
                        mistakes.to_string(),
                        if mistakes == 0 {
                            Color32::from_rgb(92, 181, 144)
                        } else {
                            Color32::from_rgb(220, 113, 97)
                        },
                    );
                    stat_chip(
                        ui,
                        "EV Lost",
                        format!("{:.2} BB", ev_lost),
                        if ev_lost < 0.1 {
                            Color32::from_rgb(92, 181, 144)
                        } else {
                            Color32::from_rgb(220, 113, 97)
                        },
                    );
                });
                ui.add_space(10.0);
                ui.label(
                    RichText::new(verdict_text(mistakes, ev_lost))
                        .size(14.0)
                        .color(Color32::from_rgb(190, 197, 210)),
                );
            });

        ui.add_space(16.0);
        if ui
            .add(
                egui::Button::new(RichText::new("New Hand").size(16.0).strong())
                    .min_size(Vec2::new(160.0, 46.0))
                    .corner_radius(CornerRadius::same(12))
                    .fill(Color32::from_rgb(71, 109, 196)),
            )
            .clicked()
        {
            let config = self.session.config;
            self.full_hand.reset(config);
        }
    }
}

impl eframe::App for PokerTrainerApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(
                egui::Frame::new()
                    .fill(Color32::from_rgb(14, 17, 24))
                    .inner_margin(Margin::same(24)),
            )
            .show(ctx, |ui| {
                self.render_titlebar(ctx, ui);
                ui.add_space(10.0);

                // Mode tabs
                egui::Frame::new()
                    .fill(Color32::from_rgb(20, 24, 33))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(45, 51, 63)))
                    .corner_radius(CornerRadius::same(12))
                    .inner_margin(Margin::symmetric(14, 8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("Mode:")
                                    .size(14.0)
                                    .color(Color32::from_rgb(150, 160, 173)),
                            );
                            for (mode, label) in [
                                (AppMode::QuickDrill, "Quick Drill"),
                                (AppMode::FullHand, "Full Hand"),
                            ] {
                                let selected = self.mode == mode;
                                let btn = egui::Button::new(
                                    RichText::new(label).size(14.0).strong(),
                                )
                                .min_size(Vec2::new(120.0, 30.0))
                                .corner_radius(CornerRadius::same(8))
                                .fill(if selected {
                                    Color32::from_rgb(71, 109, 196)
                                } else {
                                    Color32::from_rgb(32, 37, 48)
                                })
                                .stroke(Stroke::new(
                                    1.0,
                                    if selected {
                                        Color32::from_rgb(92, 130, 210)
                                    } else {
                                        Color32::from_rgb(55, 63, 78)
                                    },
                                ));
                                if ui.add(btn).clicked() && !selected {
                                    self.mode = mode;
                                    if mode == AppMode::FullHand {
                                        let config = self.session.config;
                                        self.full_hand.reset(config);
                                    }
                                }
                            }
                        });
                    });

                ui.add_space(12.0);

                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .scroll_bar_visibility(ScrollBarVisibility::AlwaysHidden)
                    .show(ui, |ui| match self.mode {
                        AppMode::QuickDrill => {
                            self.render_header(ui);
                            ui.add_space(20.0);
                            self.render_spot_card(ui);
                            ui.add_space(18.0);
                            self.render_table_and_actions(ui);
                            ui.add_space(12.0);
                        }
                        AppMode::FullHand => {
                            self.render_full_hand(ui);
                            ui.add_space(12.0);
                        }
                    });
            });
    }
}

// ================================================================
//  Helpers
// ================================================================

fn verdict_text(mistakes: usize, ev_lost: f32) -> String {
    if mistakes == 0 {
        return "You played every street optimally. No EV was leaked.".to_owned();
    }
    let quality = if ev_lost < 1.0 {
        "minor"
    } else if ev_lost < 3.0 {
        "moderate"
    } else {
        "significant"
    };
    format!(
        "You made {} {} mistake{} costing {:.2} BB. Study the explanations above to understand what the correct play was and why.",
        mistakes,
        quality,
        if mistakes == 1 { "" } else { "s" },
        ev_lost
    )
}

fn street_color(street: Street) -> &'static str {
    match street {
        Street::Preflop => "#543c63",
        Street::Flop => "#184e4a",
        Street::Turn => "#684b28",
        Street::River => "#2d405f",
    }
}

fn configure_theme(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals::dark();
    style.spacing.item_spacing = Vec2::new(10.0, 10.0);
    style.spacing.button_padding = Vec2::new(18.0, 12.0);
    style.visuals.panel_fill = Color32::from_rgb(14, 17, 24);
    style.visuals.widgets.noninteractive.bg_fill = Color32::from_rgb(23, 27, 36);
    style.visuals.override_text_color = Some(Color32::from_rgb(228, 232, 238));
    ctx.set_style(style);
}

fn panel_frame() -> egui::Frame {
    egui::Frame::new()
        .fill(Color32::from_rgb(20, 24, 33))
        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 51, 63)))
        .corner_radius(CornerRadius::same(18))
        .inner_margin(Margin::same(18))
}

fn render_card(ui: &mut egui::Ui, card: Card) {
    let color = color_from_hex(card.suit.color_hex());
    ui.allocate_ui_with_layout(
        Vec2::new(96.0, 132.0),
        egui::Layout::top_down(egui::Align::Center),
        |ui| {
            egui::Frame::new()
                .fill(Color32::from_rgb(245, 241, 233))
                .stroke(Stroke::new(1.0, Color32::from_rgb(195, 185, 170)))
                .corner_radius(CornerRadius::same(18))
                .inner_margin(Margin::same(16))
                .show(ui, |ui| {
                    ui.set_min_size(Vec2::new(64.0, 100.0));
                    ui.set_max_size(Vec2::new(64.0, 100.0));
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(card.rank.short())
                                .size(34.0)
                                .strong()
                                .color(Color32::from_rgb(34, 38, 46)),
                        );
                        ui.label(RichText::new(card.suit.symbol()).size(32.0).color(color));
                    });
                });
        },
    );
}

fn stat_chip(ui: &mut egui::Ui, label: &str, value: String, accent: Color32) {
    egui::Frame::new()
        .fill(Color32::from_rgb(22, 26, 35))
        .stroke(Stroke::new(1.0, Color32::from_rgb(48, 55, 67)))
        .corner_radius(CornerRadius::same(14))
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_min_size(Vec2::new(128.0, 52.0));
            ui.horizontal(|ui| {
                ui.label(RichText::new(label).size(13.0).color(Color32::from_rgb(150, 160, 173)));
                ui.add_space(6.0);
                ui.label(RichText::new(value).size(18.0).strong().color(accent));
            });
        });
}

fn badge(ui: &mut egui::Ui, text: String, hex: &str) {
    egui::Frame::new()
        .fill(color_from_hex(hex))
        .corner_radius(CornerRadius::same(255))
        .inner_margin(Margin::symmetric(10, 6))
        .show(ui, |ui| {
            ui.label(RichText::new(text).size(13.0).color(Color32::WHITE));
        });
}

fn ghost_box(ui: &mut egui::Ui, text: &str) {
    egui::Frame::new()
        .fill(Color32::from_rgb(24, 29, 38))
        .stroke(Stroke::new(1.0, Color32::from_rgb(42, 49, 59)))
        .corner_radius(CornerRadius::same(16))
        .inner_margin(Margin::same(18))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text).size(15.0).color(Color32::from_rgb(170, 178, 190)),
            );
        });
}

fn button_fill(action: Action) -> Color32 {
    match action {
        Action::Raise => Color32::from_rgb(115, 64, 147),
        Action::Call => Color32::from_rgb(44, 121, 91),
        Action::Fold => Color32::from_rgb(122, 54, 58),
    }
}

fn color_from_hex(hex: &str) -> Color32 {
    let hex = hex.trim_start_matches('#');
    let bytes = u32::from_str_radix(hex, 16).unwrap_or(0xffffff);
    let r = ((bytes >> 16) & 0xff) as u8;
    let g = ((bytes >> 8) & 0xff) as u8;
    let b = (bytes & 0xff) as u8;
    Color32::from_rgb(r, g, b)
}
