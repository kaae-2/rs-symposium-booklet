use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub enum PlanAction {
    CreateDir { path: PathBuf },
    WriteFile { path: PathBuf, summary: String },
    EmitTypst { path: PathBuf, template: String, command: Option<String> },
    UpdateManifest { path: PathBuf, manifest_summary: String },
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Plan {
    pub actions: Vec<PlanAction>,
}

impl Plan {
    pub fn push(&mut self, a: PlanAction) {
        self.actions.push(a);
    }

    pub fn pretty_print(&self) -> String {
        let mut out = String::new();
        for a in &self.actions {
            match a {
                PlanAction::CreateDir { path } => {
                    out.push_str(&format!("Create dir: {}\n", path.display()));
                }
                PlanAction::WriteFile { path, summary } => {
                    out.push_str(&format!("Write file: {} — {}\n", path.display(), summary));
                }
                PlanAction::EmitTypst { path, template, command } => {
                    out.push_str(&format!("Emit typst: {} (template {})\n", path.display(), template));
                    if let Some(cmd) = command {
                        out.push_str(&format!("  Command: {}\n", cmd));
                    }
                }
                PlanAction::UpdateManifest { path, manifest_summary } => {
                    out.push_str(&format!("Update manifest: {} — {}\n", path.display(), manifest_summary));
                }
            }
        }
        out
    }
}
