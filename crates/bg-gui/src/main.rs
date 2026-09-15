//! A hand-painted Board prototype: no stylesheet, no widget theming beyond
//! the window chrome. Every Slot is drawn with `egui::Painter` calls, so the
//! whole card layout is Rust code -- position, color and text are values we
//! compute, not rules a cascade resolves for us.

use bg_sim::action_phase::board_of;
use bg_sim::party::{Board, Party, SLOTS, Unit};
use bg_sim::units::{DefId, Keyword, UnitDef};
use eframe::egui;

fn u(name: &str, attack: i32, health: i32, keywords: &[Keyword]) -> Unit {
    Unit::new(&UnitDef {
        id: DefId::new(name),
        name: name.to_owned(),
        tier: 1,
        attack,
        health,
        tribes: vec![],
        all_tribes: false,
        keywords: keywords.to_vec(),
        abilities: vec![],
        token: false,
        text: String::new(),
    })
}

struct BoardApp {
    board: Board,
    screenshot_path: Option<String>,
    screenshot_requested: bool,
}

impl Default for BoardApp {
    fn default() -> Self {
        Self {
            screenshot_path: std::env::var("BG_GUI_SCREENSHOT").ok(),
            screenshot_requested: false,
            board: board_of(
                vec![
                    u("Bait", 1, 1, &[]),
                    u("Gusty", 3, 6, &[Keyword::Windfury]),
                    u("Rockpool", 2, 3, &[Keyword::Taunt]),
                ],
                vec![
                    u("Shielded", 2, 4, &[Keyword::DivineShield]),
                    u("Phoenix", 1, 2, &[Keyword::Reborn]),
                    u("Toxfin", 1, 1, &[Keyword::Poisonous, Keyword::Taunt]),
                    u("Herald", 4, 4, &[Keyword::Rally]),
                ],
            ),
        }
    }
}

const CARD_W: f32 = 84.0;
const CARD_H: f32 = 108.0;
const GAP: f32 = 10.0;

/// Paint one Slot: an occupied one as a card, an empty one as a dashed well.
fn paint_slot(painter: &egui::Painter, top_left: egui::Pos2, unit: Option<&Unit>) {
    let rect = egui::Rect::from_min_size(top_left, egui::vec2(CARD_W, CARD_H));

    let Some(unit) = unit else {
        painter.rect_stroke(
            rect,
            6.0,
            egui::Stroke::new(1.0, egui::Color32::from_gray(70)),
        );
        return;
    };

    let fill = if unit.has(Keyword::DivineShield) {
        egui::Color32::from_rgb(60, 84, 120)
    } else {
        egui::Color32::from_rgb(46, 42, 38)
    };
    let border = if unit.has(Keyword::Taunt) {
        egui::Color32::from_rgb(196, 160, 60)
    } else {
        egui::Color32::from_gray(110)
    };

    painter.rect_filled(rect, 8.0, fill);
    painter.rect_stroke(rect, 8.0, egui::Stroke::new(2.0, border));

    // Name, wrapped to the card width.
    painter.text(
        rect.left_top() + egui::vec2(6.0, 6.0),
        egui::Align2::LEFT_TOP,
        &unit.name,
        egui::FontId::proportional(13.0),
        egui::Color32::WHITE,
    );

    // Keyword badges: small filled circles with the single-letter badge,
    // laid out left-to-right under the name.
    let mut badge_pos = rect.left_top() + egui::vec2(8.0, 28.0);
    for kw in unit.keywords.iter() {
        painter.circle_filled(badge_pos, 8.0, egui::Color32::from_rgb(90, 90, 90));
        painter.text(
            badge_pos,
            egui::Align2::CENTER_CENTER,
            kw.badge(),
            egui::FontId::proportional(10.0),
            egui::Color32::WHITE,
        );
        badge_pos.x += 18.0;
    }

    // Attack, bottom-left gem.
    let atk_center = rect.left_bottom() + egui::vec2(14.0, -14.0);
    painter.circle_filled(atk_center, 13.0, egui::Color32::from_rgb(120, 60, 30));
    painter.text(
        atk_center,
        egui::Align2::CENTER_CENTER,
        unit.attack.to_string(),
        egui::FontId::proportional(14.0),
        egui::Color32::WHITE,
    );

    // Health, bottom-right gem -- red once damaged off its max.
    let hp_color = if unit.health < unit.max_health {
        egui::Color32::from_rgb(150, 30, 30)
    } else {
        egui::Color32::from_rgb(40, 110, 50)
    };
    let hp_center = rect.right_bottom() + egui::vec2(-14.0, -14.0);
    painter.circle_filled(hp_center, 13.0, hp_color);
    painter.text(
        hp_center,
        egui::Align2::CENTER_CENTER,
        unit.health.to_string(),
        egui::FontId::proportional(14.0),
        egui::Color32::WHITE,
    );
}

fn paint_party(ui: &mut egui::Ui, label: &str, party: &Party) {
    ui.label(
        egui::RichText::new(label)
            .size(15.0)
            .color(egui::Color32::from_gray(180)),
    );
    let (rect, _response) = ui.allocate_exact_size(
        egui::vec2(SLOTS as f32 * (CARD_W + GAP) - GAP, CARD_H),
        egui::Sense::hover(),
    );
    let painter = ui.painter_at(rect);
    for slot in 0..SLOTS {
        let top_left = rect.left_top() + egui::vec2(slot as f32 * (CARD_W + GAP), 0.0);
        paint_slot(&painter, top_left, party.get(slot));
    }
}

impl eframe::App for BoardApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Board (hand-painted prototype)");
            ui.add_space(12.0);
            paint_party(ui, "Opposing", &self.board.opposing);
            ui.add_space(24.0);
            paint_party(ui, "Player", &self.board.player);
        });

        // Headless verification hook: BG_GUI_SCREENSHOT=path.png dumps the
        // first frame and exits, so the prototype can be checked without a
        // display attached.
        if let Some(path) = self.screenshot_path.clone() {
            if !self.screenshot_requested {
                ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot);
                self.screenshot_requested = true;
            }
            ctx.input(|i| {
                for event in &i.events {
                    if let egui::Event::Screenshot { image, .. } = event {
                        let buf = image::RgbaImage::from_raw(
                            image.size[0] as u32,
                            image.size[1] as u32,
                            image.as_raw().to_vec(),
                        )
                        .expect("screenshot buffer matches its own size");
                        buf.save(&path).expect("write screenshot png");
                        std::process::exit(0);
                    }
                }
            });
        }
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "bg-gui prototype",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([820.0, 400.0]),
            ..Default::default()
        },
        Box::new(|_cc| Ok(Box::new(BoardApp::default()))),
    )
}
