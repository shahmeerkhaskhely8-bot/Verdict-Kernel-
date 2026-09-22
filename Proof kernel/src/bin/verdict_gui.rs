use eframe::egui;
use verdict::{Policy, PolicyEngine, Verdict};

/// Presentation-only desktop application.
///
/// The GUI owns no verdict logic. It forwards the parsed user input to the
/// verified kernel's policy evaluator and renders the returned verdict.
struct VerdictApp {
    input: String,
    result: Verdict,
    input_status: Option<&'static str>,
    policy: Policy<i32, 0>,
}

impl Default for VerdictApp {
    fn default() -> Self {
        Self {
            input: String::new(),
            result: Verdict::Unknown,
            input_status: None,
            policy: Policy::new(),
        }
    }
}

impl eframe::App for VerdictApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("Verified Verdict");
            ui.label("Enter an integer input for the verified policy kernel.");

            ui.add(egui::TextEdit::singleline(&mut self.input).hint_text("Input"));

            if ui.button("Evaluate").clicked() {
                match self.input.trim().parse::<i32>() {
                    Ok(value) => {
                        self.result = PolicyEngine::evaluate(&self.policy, &value);
                        self.input_status = None;
                    }
                    Err(_) => {
                        self.input_status = Some("Enter a valid integer input.");
                    }
                }
            }

            if let Some(status) = self.input_status {
                ui.colored_label(egui::Color32::RED, status);
            }

            ui.separator();
            ui.label("Kernel verdict:");
            ui.monospace(format!("{:?}", self.result));
        });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Verified Verdict",
        eframe::NativeOptions::default(),
        Box::new(|_creation_context| Ok(Box::new(VerdictApp::default()))),
    )
}
