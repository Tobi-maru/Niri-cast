use std::process::Command;

use anyhow::{anyhow, Context};

#[derive(Debug, Clone)]
pub struct AudioSink {
    pub id: String,
    pub name: String,
    pub is_default: bool,
    pub is_hdmi: bool,
}

impl AudioSink {
    pub fn display_line(&self) -> String {
        let default_mark = if self.is_default { "*" } else { " " };
        format!("{default_mark} {}. {}", self.id, self.name)
    }
}

#[derive(Debug, Default)]
pub struct AudioAdapter;

impl AudioAdapter {
    pub fn list_sink_objects(&self) -> anyhow::Result<Vec<AudioSink>> {
        let output = Command::new("wpctl")
            .arg("status")
            .output()
            .context("failed to run `wpctl status`")?;
        if !output.status.success() {
            return Err(anyhow!("`wpctl status` exited with non-zero status"));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_audio_sinks(&stdout))
    }

    pub fn list_sinks(&self) -> anyhow::Result<Vec<String>> {
        Ok(self
            .list_sink_objects()?
            .into_iter()
            .map(|sink| sink.display_line())
            .collect())
    }

    pub fn find_first_hdmi_sink(&self) -> anyhow::Result<Option<String>> {
        let sinks = self.list_sink_objects()?;
        Ok(sinks
            .into_iter()
            .find(|sink| sink.is_hdmi)
            .map(|sink| sink.display_line()))
    }

    pub fn select_first_hdmi_sink(&self) -> anyhow::Result<Option<String>> {
        let sink = self
            .list_sink_objects()?
            .into_iter()
            .find(|sink| sink.is_hdmi);
        if let Some(sink) = &sink {
            let _ = self.set_default_and_move_streams_by_id(&sink.id)?;
        }
        Ok(sink.map(|sink| sink.display_line()))
    }

    pub fn set_default_by_id(&self, id: &str) -> anyhow::Result<()> {
        let status = Command::new("wpctl")
            .args(["set-default", id])
            .status()
            .context("failed to run `wpctl set-default`")?;
        if !status.success() {
            return Err(anyhow!("`wpctl set-default {id}` exited with non-zero status"));
        }
        Ok(())
    }

    pub fn set_default_by_name(&self, sink_line: &str) -> anyhow::Result<()> {
        let id = extract_first_number(sink_line)
            .ok_or_else(|| anyhow!("could not parse sink id from line: {sink_line}"))?;

        self.set_default_by_id(&id)
    }

    pub fn set_default_and_move_streams_by_id(&self, id: &str) -> anyhow::Result<usize> {
        self.set_default_by_id(id)?;

        let sink_name = match self.sink_node_name_by_id(id)? {
            Some(name) => name,
            None => return Ok(0),
        };

        let stream_ids = self.active_sink_input_ids().unwrap_or_default();
        let mut moved = 0usize;
        for stream_id in stream_ids {
            let status = Command::new("pactl")
                .args(["move-sink-input", &stream_id, &sink_name])
                .status();
            if let Ok(status) = status {
                if status.success() {
                    moved += 1;
                }
            }
        }

        Ok(moved)
    }

    pub fn try_switch_card_profile_to_laptop(&self) -> anyhow::Result<bool> {
        let candidates = [
            "output:analog-stereo+input:analog-stereo",
            "output:analog-stereo",
            "analog-stereo",
        ];
        self.try_set_card_profile_candidates(&candidates)
    }

    pub fn try_switch_card_profile_to_tv(&self) -> anyhow::Result<bool> {
        let candidates = [
            "output:hdmi-stereo+input:analog-stereo",
            "output:hdmi-stereo",
            "hdmi-stereo",
        ];
        self.try_set_card_profile_candidates(&candidates)
    }

    fn try_set_card_profile_candidates(&self, candidates: &[&str]) -> anyhow::Result<bool> {
        if !Command::new("which")
            .arg("pactl")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Ok(false);
        }

        let cards = list_pactl_cards()?;
        if cards.is_empty() {
            return Ok(false);
        }

        for card in &cards {
            for profile in candidates {
                let status = Command::new("pactl")
                    .args(["set-card-profile", card, profile])
                    .status();
                if let Ok(status) = status {
                    if status.success() {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }
}

impl AudioAdapter {
    fn sink_node_name_by_id(&self, id: &str) -> anyhow::Result<Option<String>> {
        let output = Command::new("wpctl")
            .args(["inspect", id])
            .output()
            .context("failed to run `wpctl inspect <id>`")?;
        if !output.status.success() {
            return Ok(None);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_node_name_from_wpctl_inspect(&stdout))
    }

    fn active_sink_input_ids(&self) -> anyhow::Result<Vec<String>> {
        if !Command::new("which")
            .arg("pactl")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Ok(Vec::new());
        }

        let output = Command::new("pactl")
            .args(["list", "short", "sink-inputs"])
            .output()
            .context("failed to run `pactl list short sink-inputs`")?;
        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let ids = stdout
            .lines()
            .filter_map(|line| line.split_whitespace().next())
            .map(str::to_string)
            .collect::<Vec<_>>();
        Ok(ids)
    }
}

fn list_pactl_cards() -> anyhow::Result<Vec<String>> {
    let output = Command::new("pactl")
        .args(["list", "short", "cards"])
        .output()
        .context("failed to run `pactl list short cards`")?;
    if !output.status.success() {
        return Err(anyhow!("`pactl list short cards` exited with non-zero status"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let cards = stdout
        .lines()
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let _id = fields.next()?;
            let card = fields.next()?;
            Some(card.to_string())
        })
        .collect::<Vec<_>>();
    Ok(cards)
}

fn parse_node_name_from_wpctl_inspect(inspect_output: &str) -> Option<String> {
    inspect_output.lines().find_map(|line| {
        let marker = "node.name = \"";
        let idx = line.find(marker)?;
        let rest = &line[idx + marker.len()..];
        let end = rest.find('"')?;
        let name = rest[..end].trim();
        if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        }
    })
}

fn parse_audio_sinks(status_output: &str) -> Vec<AudioSink> {
    let mut sinks = Vec::new();
    let mut in_audio = false;
    let mut in_sinks = false;

    for raw in status_output.lines() {
        let line = raw.trim();

        if line == "Audio" {
            in_audio = true;
            in_sinks = false;
            continue;
        }

        if in_audio && (line == "Video" || line == "Settings") {
            break;
        }

        if !in_audio {
            continue;
        }

        if line.starts_with("├─ Sinks:") || line.starts_with("└─ Sinks:") {
            in_sinks = true;
            continue;
        }

        if in_sinks
            && (line.starts_with("├─ Sources:")
                || line.starts_with("└─ Sources:")
                || line.starts_with("├─ Streams:")
                || line.starts_with("└─ Streams:")
                || line.starts_with("├─ Filters:")
                || line.starts_with("└─ Filters:"))
        {
            break;
        }

        if in_sinks {
            if let Some(sink) = parse_sink_line(line) {
                sinks.push(sink);
            }
        }
    }

    sinks
}

fn parse_sink_line(line: &str) -> Option<AudioSink> {
    if !line.contains('.') {
        return None;
    }

    let is_default = line.contains('*');
    let id = extract_first_number(line)?;

    let after_dot = line.split_once(". ")?.1.trim();
    let name = after_dot
        .split(" [")
        .next()
        .map(str::trim)
        .unwrap_or(after_dot)
        .to_string();
    if name.is_empty() {
        return None;
    }

    Some(AudioSink {
        id,
        is_hdmi: name.to_ascii_lowercase().contains("hdmi"),
        is_default,
        name,
    })
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
    use super::{extract_first_number, parse_audio_sinks, parse_node_name_from_wpctl_inspect};

    #[test]
    fn extracts_digits() {
        assert_eq!(extract_first_number(" 42. HDMI Output"), Some("42".to_string()));
        assert_eq!(extract_first_number("sink id 103 ready"), Some("103".to_string()));
        assert_eq!(extract_first_number("no digits"), None);
    }

    #[test]
    fn parses_audio_sinks_section() {
        let status = r#"
Audio
 ├─ Devices:
 │      52. Built-in Audio                      [alsa]
 │
 ├─ Sinks:
 │  *   83. Built-in Audio Digital Stereo (HDMI) [vol: 0.34]
 │      91. Built-in Audio Analog Stereo [vol: 0.20]
 │
 ├─ Sources:
 │  *   71. Built-in Audio Analog Stereo        [vol: 1.00]
"#;

        let sinks = parse_audio_sinks(status);
        assert_eq!(sinks.len(), 2);
        assert_eq!(sinks[0].id, "83");
        assert!(sinks[0].is_default);
        assert!(sinks[0].is_hdmi);
        assert_eq!(sinks[1].id, "91");
        assert!(!sinks[1].is_default);
    }

    #[test]
    fn parses_node_name_from_inspect() {
        let inspect = r#"
id 83, type PipeWire:Interface:Node
  * media.class = "Audio/Sink"
  * node.name = "alsa_output.pci-0000_00_1f.3.hdmi-stereo"
"#;
        assert_eq!(
            parse_node_name_from_wpctl_inspect(inspect),
            Some("alsa_output.pci-0000_00_1f.3.hdmi-stereo".to_string())
        );
    }
}
