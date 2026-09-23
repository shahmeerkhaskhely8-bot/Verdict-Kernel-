use std::path::Path;

use eframe::egui::{self, Color32, RichText};
use verdict::{evaluate_policy, PolicyEngine, Verdict};

const VERIFIED: Color32 = Color32::from_rgb(46, 125, 50);
const UNVERIFIED: Color32 = Color32::from_rgb(198, 40, 40);
const UNKNOWN: Color32 = Color32::from_rgb(239, 126, 34);

fn evaluate_integer(value: &i32) -> Verdict {
    evaluate_policy(*value)
}

struct VerdictApp {
    value: String,
    file_path: String,
    verdict: Verdict,
    reason: String,
    file_status: String,
    tab: Tab,
}

impl Default for VerdictApp {
    fn default() -> Self {
        Self {
            value: String::new(),
            file_path: String::new(),
            verdict: Verdict::Unknown,
            reason: String::new(),
            file_status: String::new(),
            tab: Tab::Audit,
        }
    }
}

#[derive(Default, PartialEq, Eq)]
enum Tab {
    #[default]
    Audit,
    Guide,
}

impl VerdictApp {
    fn evaluate(&mut self) {
        self.reason.clear();
        match self.value.trim().parse::<i32>() {
            Ok(value) => {
                let mut engine = PolicyEngine::<i32, 1>::new();
                let _ = engine.add_rule(evaluate_integer);
                self.verdict = engine.evaluate(&value);
                self.reason = format!("Input value {value} matched the mathematical rule n > 0.");
            }
            Err(_) => {
                self.verdict = Verdict::Unknown;
                self.reason = String::from("Enter a valid whole number to evaluate the policy.");
            }
        }
    }

    fn clear(&mut self) {
        self.value.clear();
        self.file_path.clear();
        self.file_status.clear();
        self.reason.clear();
        self.verdict = Verdict::Unknown;
    }

    fn audit_file_path(&mut self) {
        let path = self.file_path.trim();
        self.file_status = if path.is_empty() {
            String::from("No file path supplied.")
        } else if Path::new(path).is_file() {
            String::from("File path is available for auditing.")
        } else {
            String::from("File was not found. The numeric policy is unchanged.")
        };
    }

    fn verdict_color(&self) -> Color32 {
        match self.verdict {
            Verdict::Verified => VERIFIED,
            Verdict::Unverified => UNVERIFIED,
            Verdict::Unknown => UNKNOWN,
        }
    }

    fn verdict_text(&self) -> &'static str {
        match self.verdict {
            Verdict::Verified => "VERIFIED",
            Verdict::Unverified => "UNVERIFIED",
            Verdict::Unknown => "UNKNOWN",
        }
    }

    fn verdict_explanation(&self) -> &'static str {
        match self.verdict {
            Verdict::Verified => "The value is positive, so the policy accepts it.",
            Verdict::Unverified => {
                "The value is zero or negative, so the policy rejects it safely."
            }
            Verdict::Unknown => "No valid evaluation has been completed yet.",
        }
    }

    fn header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Proof Kernel VVIP Dashboard");
            ui.add_space(16.0);
            ui.label(
                RichText::new("Core Engine: Formally Verified (Coq)")
                    .color(Color32::WHITE)
                    .background_color(VERIFIED)
                    .strong(),
            );
        });
        ui.separator();
    }

    fn audit_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Audit & Evaluate");
        ui.label(
            "Run the proven integer policy and optionally record a file path for audit context.",
        );
        ui.add_space(8.0);

        egui::Grid::new("input_grid").num_columns(2).show(ui, |ui| {
            ui.label("Numeric value");
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.value).hint_text("e.g. 42"));
                if ui.button("Evaluate Value").clicked() {
                    self.evaluate();
                }
            });
            ui.end_row();

            ui.label("File path");
            ui.horizontal(|ui| {
                ui.add(egui::TextEdit::singleline(&mut self.file_path).hint_text("Optional path"));
                if ui.button("Audit Path").clicked() {
                    self.audit_file_path();
                }
            });
            ui.end_row();
        });

        if !self.file_status.is_empty() {
            ui.small(&self.file_status);
        }

        ui.add_space(12.0);
        egui::Frame::group(ui.style()).show(ui, |ui| {
            ui.heading("Decision Reason Inspector");
            ui.add_space(6.0);
            ui.label(
                RichText::new(self.verdict_text())
                    .size(28.0)
                    .color(Color32::WHITE)
                    .background_color(self.verdict_color())
                    .strong(),
            );
            ui.add_space(8.0);
            ui.label(RichText::new("Reason Breakdown").strong());
            ui.label(if self.value.trim().is_empty() {
                "Input value: not supplied."
            } else {
                "Input value: parsed as an integer."
            });
            ui.label("Mathematical Rule: n > 0 produces Verified; n <= 0 produces Unverified.");
            ui.label(format!("Explanation: {}", self.verdict_explanation()));
            if !self.reason.is_empty() {
                ui.label(format!("Details: {}", self.reason));
            }
        });

        ui.add_space(10.0);
        if ui.button("Clear").clicked() {
            self.clear();
        }
    }

    fn guide_tab(&self, ui: &mut egui::Ui) {
        ui.heading("User Guide & Help");
        ui.label("1. Enter a whole number in Numeric value.");
        ui.label("2. Select Evaluate Value to run the formally verified policy.");
        ui.label("3. Review the verdict badge and the Reason Breakdown.");
        ui.label(
            "4. Optionally enter a file path and select Audit Path to check its availability.",
        );
        ui.label("5. Select Clear to reset the value, file path, verdict, and messages.");
        ui.add_space(10.0);
        ui.label(RichText::new("Button reference").strong());
        ui.label("Evaluate Value: runs the policy check on the numeric input.");
        ui.label("Audit Path: checks whether the supplied file path is available; it does not alter the policy verdict.");
        ui.label("Clear: resets the dashboard to its initial Unknown state.");
        ui.add_space(10.0);
        ui.label(RichText::new("How auditing works").strong());
        ui.label("The Policy Engine applies the rule n > 0. A positive value is Verified; zero or a negative value is Unverified. A missing or invalid value remains Unknown until a valid evaluation is performed. File auditing only provides path context for the operator.");
    }
}

impl eframe::App for VerdictApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("header").show(ctx, |ui| self.header(ui));
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.tab, Tab::Audit, "Audit & Evaluate");
                ui.selectable_value(&mut self.tab, Tab::Guide, "User Guide & Help");
            });
            ui.separator();
            match self.tab {
                Tab::Audit => self.audit_tab(ui),
                Tab::Guide => self.guide_tab(ui),
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Proof Kernel VVIP Dashboard",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Box::new(VerdictApp::default())),
    )
}
