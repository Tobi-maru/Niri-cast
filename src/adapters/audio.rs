use std::process::Command;

use anyhow::{anyhow, Context};

#[derive(Debug, Default)]
pub struct AudioAdapter;

impl AudioAdapter {
    pub fn list_sinks(&self) -> anyhow::Result<Vec<String>> {
        let output = Command::new("wpctl")
            .arg("status")
            .output()
            .context("failed to run `wpctl status`")?;
        if !output.status.success() {
            return Err(anyhow!("`wpctl status` exited with non-zero status"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let sinks = stdout
            .lines()
            .filter(|l| l.contains(". ") && l.to_ascii_lowercase().contains("output"))
            .map(|l| l.trim().to_string())
            .collect::<Vec<_>>();

        Ok(sinks)
    }

    pub fn find_first_hdmi_sink(&self) -> anyhow::Result<Option<String>> {
        let sinks = self.list_sinks()?;
        Ok(sinks
            .into_iter()
            .find(|line| line.to_ascii_lowercase().contains("hdmi")))
    }

    pub fn select_first_hdmi_sink(&self) -> anyhow::Result<Option<String>> {
        let sink = self.find_first_hdmi_sink()?;
        if let Some(name) = &sink {
            self.set_default_by_name(name)?;
        }
        Ok(sink)
    }

    pub fn set_default_by_name(&self, sink_line: &str) -> anyhow::Result<()> {
        let id = extract_first_number(sink_line)
            .ok_or_else(|| anyhow!("could not parse sink id from line: {sink_line}"))?;

        let status = Command::new("wpctl")
            .args(["set-default", &id])
            .status()
            .context("failed to run `wpctl set-default`")?;
        if !status.success() {
            return Err(anyhow!("`wpctl set-default {id}` exited with non-zero status"));
        }
        Ok(())
    }
}

fn extract_first_number(input: &str) -> Option<String> {
    let mut current = String::new();
    for ch in input.chars() {
        if ch.is_ascii_digit() {
            current.push(ch);
        } else if !current.is_empty() {
            return Some(current);
        }
    }
    if current.is_empty() {
        None
    } else {
        Some(current)
    }
}

#[cfg(test)]
mod tests {
    use super::extract_first_number;

    #[test]
    fn extracts_digits() {
        assert_eq!(extract_first_number(" 42. HDMI Output"), Some("42".to_string()));
        assert_eq!(extract_first_number("sink id 103 ready"), Some("103".to_string()));
        assert_eq!(extract_first_number("no digits"), None);
    }
}
