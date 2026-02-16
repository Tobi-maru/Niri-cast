use std::process::Command;

use anyhow::{anyhow, Context};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Default)]
pub struct NiriAdapter;

#[derive(Debug, Clone, Deserialize)]
pub struct NiriLogical {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct NiriOutput {
    pub name: String,
    pub logical: NiriLogical,
}

impl NiriAdapter {
    pub fn list_outputs(&self) -> anyhow::Result<Vec<String>> {
        let output = Command::new("niri")
            .args(["msg", "outputs"])
            .output()
            .context("failed to run `niri msg outputs`")?;

        if !output.status.success() {
            return Err(anyhow!("`niri msg outputs` exited with non-zero status"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let lines = stdout
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect::<Vec<_>>();

        Ok(lines)
    }

    pub fn list_hdmi_outputs(&self) -> anyhow::Result<Vec<String>> {
        let outputs = self.list_outputs()?;
        Ok(outputs
            .into_iter()
            .filter(|line| line.to_ascii_uppercase().contains("HDMI"))
            .collect())
    }

    pub fn connected_output_names(&self) -> anyhow::Result<Vec<String>> {
        let lines = self.list_outputs()?;
        let mut names = Vec::new();

        for line in lines {
            if !line.starts_with("Output ") {
                continue;
            }
            if let (Some(open), Some(close)) = (line.rfind('('), line.rfind(')')) {
                if close > open + 1 {
                    names.push(line[open + 1..close].to_string());
                }
            }
        }

        Ok(names)
    }

    pub fn outputs_json(&self) -> anyhow::Result<Vec<NiriOutput>> {
        let output = Command::new("niri")
            .args(["msg", "-j", "outputs"])
            .output()
            .context("failed to run `niri msg -j outputs`")?;

        if !output.status.success() {
            return Err(anyhow!("`niri msg -j outputs` exited with non-zero status"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let parsed: BTreeMap<String, NiriOutput> =
            serde_json::from_str(&stdout).context("failed to parse niri outputs json")?;
        Ok(parsed.into_values().collect())
    }

    pub fn list_hdmi_names(&self) -> anyhow::Result<Vec<String>> {
        Ok(self
            .outputs_json()?
            .into_iter()
            .filter(|o| o.name.to_ascii_uppercase().contains("HDMI"))
            .map(|o| o.name)
            .collect())
    }

    pub fn list_non_hdmi_names(&self) -> anyhow::Result<Vec<String>> {
        Ok(self
            .outputs_json()?
            .into_iter()
            .filter(|o| !o.name.to_ascii_uppercase().contains("HDMI"))
            .map(|o| o.name)
            .collect())
    }

    pub fn output_on(&self, output_name: &str) -> anyhow::Result<()> {
        let status = Command::new("niri")
            .args(["msg", "output", output_name, "on"])
            .status()
            .context("failed to run `niri msg output <name> on`")?;
        if !status.success() {
            return Err(anyhow!("failed to turn on output {output_name}"));
        }
        Ok(())
    }

    pub fn output_off(&self, output_name: &str) -> anyhow::Result<()> {
        let status = Command::new("niri")
            .args(["msg", "output", output_name, "off"])
            .status()
            .context("failed to run `niri msg output <name> off`")?;
        if !status.success() {
            return Err(anyhow!("failed to turn off output {output_name}"));
        }
        Ok(())
    }

    pub fn set_position(&self, output_name: &str, x: i32, y: i32) -> anyhow::Result<()> {
        let status = Command::new("niri")
            .args([
                "msg",
                "output",
                output_name,
                "position",
                "set",
                &x.to_string(),
                &y.to_string(),
            ])
            .status()
            .context("failed to run `niri msg output <name> position set <x> <y>`")?;
        if !status.success() {
            return Err(anyhow!(
                "failed to set position for output {output_name} to {x},{y}"
            ));
        }
        Ok(())
    }

    pub fn set_position_auto(&self, output_name: &str) -> anyhow::Result<()> {
        let status = Command::new("niri")
            .args(["msg", "output", output_name, "position", "auto"])
            .status()
            .context("failed to run `niri msg output <name> position auto`")?;
        if !status.success() {
            return Err(anyhow!("failed to set auto position for output {output_name}"));
        }
        Ok(())
    }
}
