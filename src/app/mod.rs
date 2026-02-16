use std::time::Duration;

use anyhow::Context;
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::adapters::{audio::AudioAdapter, niri::NiriAdapter, portal::PortalAdapter, system::SystemAdapter};
use crate::diagnostics::{run_troubleshooting, TroubleshootReport};
use crate::profiles::{ProfileStore, TvProfile};
use crate::ui;

pub struct App {
    pub selected_tab: usize,
    pub running: bool,
    pub log_lines: Vec<String>,
    pub last_outputs: Vec<String>,
    pub last_sinks: Vec<String>,
    pub diagnostics: Option<TroubleshootReport>,
    pub profile_store: ProfileStore,
    pub niri: NiriAdapter,
    pub audio: AudioAdapter,
    pub system: SystemAdapter,
    pub portal: PortalAdapter,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            selected_tab: 0,
            running: true,
            log_lines: vec!["niri-cast started".to_string()],
            last_outputs: Vec::new(),
            last_sinks: Vec::new(),
            diagnostics: None,
            profile_store: ProfileStore::new()?,
            niri: NiriAdapter::default(),
            audio: AudioAdapter::default(),
            system: SystemAdapter::default(),
            portal: PortalAdapter::default(),
        })
    }

    pub fn log(&mut self, line: impl Into<String>) {
        self.log_lines.push(line.into());
        if self.log_lines.len() > 200 {
            let _ = self.log_lines.drain(0..(self.log_lines.len() - 200));
        }
    }

    pub fn next_tab(&mut self) {
        self.selected_tab = (self.selected_tab + 1) % ui::TAB_TITLES.len();
    }

    pub fn previous_tab(&mut self) {
        if self.selected_tab == 0 {
            self.selected_tab = ui::TAB_TITLES.len() - 1;
        } else {
            self.selected_tab -= 1;
        }
    }

    pub fn refresh_discovery(&mut self) {
        self.last_outputs = self.niri.list_outputs().unwrap_or_default();
        self.last_sinks = self.audio.list_sinks().unwrap_or_default();
        self.log(format!(
            "discovered {} outputs, {} sinks",
            self.last_outputs.len(),
            self.last_sinks.len()
        ));
    }

    pub fn run_diagnostics(&mut self) {
        let report = run_troubleshooting(&self.system, &self.portal);
        self.log(format!(
            "diagnostics complete: {} ok / {} warn / {} error",
            report.ok_count(),
            report.warn_count(),
            report.error_count()
        ));
        self.diagnostics = Some(report);
    }

    pub fn apply_hdmi_audio(&mut self) {
        match self.audio.select_first_hdmi_sink() {
            Ok(Some(sink)) => self.log(format!("set default audio sink: {sink}")),
            Ok(None) => self.log("no HDMI sink found"),
            Err(err) => self.log(format!("audio switch failed: {err}")),
        }
    }

    pub fn discover_hdmi_outputs(&mut self) {
        match self.niri.list_hdmi_names() {
            Ok(outputs) if !outputs.is_empty() => {
                self.log(format!("HDMI outputs: {}", outputs.join(", ")))
            }
            Ok(_) => self.log("no HDMI outputs found"),
            Err(err) => self.log(format!("HDMI discovery failed: {err}")),
        }
    }

    pub fn save_profile(&mut self) {
        let profile = TvProfile {
            name: "default-tv".to_string(),
            hdmi_output: self.niri.list_hdmi_names().ok().and_then(|mut v| v.pop()),
            audio_sink: self.audio.find_first_hdmi_sink().ok().flatten(),
        };

        match self.profile_store.save_profile(profile) {
            Ok(()) => self.log("saved profile: default-tv"),
            Err(err) => self.log(format!("failed to save profile: {err}")),
        }
    }

    pub fn load_profile(&mut self) {
        match self.profile_store.load_profile("default-tv") {
            Ok(Some(profile)) => {
                self.log(format!("loaded profile: {}", profile.name));
                if let Some(sink) = profile.audio_sink {
                    match self.audio.set_default_by_name(&sink) {
                        Ok(()) => self.log(format!("applied audio sink: {sink}")),
                        Err(err) => self.log(format!("failed to apply sink {sink}: {err}")),
                    }
                }
            }
            Ok(None) => self.log("profile default-tv not found"),
            Err(err) => self.log(format!("failed to load profile: {err}")),
        }
    }

    pub fn cast_preflight(&mut self) {
        let preflight = crate::core::cast::preflight(&self.system, &self.portal);
        self.log(preflight.summary_line());
        for item in preflight.missing_items {
            self.log(format!("missing: {item}"));
        }
    }

    pub fn cast_extend_right(&mut self) {
        match self.apply_layout_cast(LayoutCastMode::ExtendRight) {
            Ok(msg) => self.log(msg),
            Err(err) => self.log(format!("extend-right failed: {err}")),
        }
    }

    pub fn cast_extend_left(&mut self) {
        match self.apply_layout_cast(LayoutCastMode::ExtendLeft) {
            Ok(msg) => self.log(msg),
            Err(err) => self.log(format!("extend-left failed: {err}")),
        }
    }

    pub fn cast_mirror(&mut self) {
        match self.apply_layout_cast(LayoutCastMode::Mirror) {
            Ok(msg) => self.log(msg),
            Err(err) => self.log(format!("mirror failed: {err}")),
        }
    }

    pub fn cast_hdmi_only(&mut self) {
        match self.apply_layout_cast(LayoutCastMode::HdmiOnly) {
            Ok(msg) => self.log(msg),
            Err(err) => self.log(format!("hdmi-only failed: {err}")),
        }
    }

    pub fn cast_restore_all(&mut self) {
        match self.restore_all_outputs() {
            Ok(msg) => self.log(msg),
            Err(err) => self.log(format!("restore failed: {err}")),
        }
    }

    fn apply_layout_cast(&mut self, mode: LayoutCastMode) -> anyhow::Result<String> {
        let mut outputs = self.niri.outputs_json()?;
        let hdmi_name = outputs
            .iter()
            .find(|o| o.name.to_ascii_uppercase().contains("HDMI"))
            .map(|o| o.name.clone())
            .ok_or_else(|| anyhow::anyhow!("no HDMI output available"))?;

        self.niri.output_on(&hdmi_name)?;

        if matches!(mode, LayoutCastMode::Mirror) {
            if let Some(non_hdmi_name) = self
                .niri
                .connected_output_names()?
                .into_iter()
                .find(|name| !name.to_ascii_uppercase().contains("HDMI"))
            {
                self.niri.output_on(&non_hdmi_name)?;
            }
            outputs = self.niri.outputs_json()?;
        }

        let hdmi_output = outputs
            .iter()
            .find(|o| o.name == hdmi_name)
            .ok_or_else(|| anyhow::anyhow!("could not read HDMI output logical info"))?;

        let primary = outputs
            .iter()
            .find(|o| !o.name.to_ascii_uppercase().contains("HDMI"))
            .unwrap_or(hdmi_output);

        match mode {
            LayoutCastMode::ExtendRight => {
                self.niri.set_position(
                    &hdmi_output.name,
                    primary.logical.x + primary.logical.width,
                    primary.logical.y,
                )?;
                if primary.name != hdmi_output.name {
                    self.niri.output_on(&primary.name)?;
                }
                Ok(format!(
                    "cast mode set: extend-right ({} right of {})",
                    hdmi_output.name, primary.name
                ))
            }
            LayoutCastMode::ExtendLeft => {
                self.niri.set_position(
                    &hdmi_output.name,
                    primary.logical.x - hdmi_output.logical.width,
                    primary.logical.y,
                )?;
                if primary.name != hdmi_output.name {
                    self.niri.output_on(&primary.name)?;
                }
                Ok(format!(
                    "cast mode set: extend-left ({} left of {})",
                    hdmi_output.name, primary.name
                ))
            }
            LayoutCastMode::Mirror => {
                self.niri
                    .set_position(&hdmi_output.name, primary.logical.x, primary.logical.y)?;
                if primary.name != hdmi_output.name {
                    self.niri.set_position_auto(&primary.name)?;
                    self.niri.output_on(&primary.name)?;
                }
                Ok(format!(
                    "cast mode set: mirror ({} mirrored with {})",
                    hdmi_output.name, primary.name
                ))
            }
            LayoutCastMode::HdmiOnly => {
                for output in &outputs {
                    if output.name == hdmi_output.name {
                        self.niri.output_on(&output.name)?;
                    } else {
                        self.niri.output_off(&output.name)?;
                    }
                }
                Ok(format!("cast mode set: hdmi-only ({})", hdmi_output.name))
            }
        }
    }

    fn restore_all_outputs(&mut self) -> anyhow::Result<String> {
        let names = self.niri.connected_output_names()?;
        if names.is_empty() {
            return Err(anyhow::anyhow!("no connected outputs found"));
        }

        for name in &names {
            self.niri.output_on(name)?;
        }
        for name in &names {
            self.niri.set_position_auto(name)?;
        }

        Ok(format!("restored outputs: {}", names.join(", ")))
    }
}

#[derive(Debug, Clone, Copy)]
enum LayoutCastMode {
    ExtendRight,
    ExtendLeft,
    Mirror,
    HdmiOnly,
}

pub fn run() -> anyhow::Result<()> {
    enable_raw_mode().context("failed to enable raw mode")?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen).context("failed to enter alternate screen")?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("failed to initialize terminal")?;

    let mut app = App::new()?;
    app.refresh_discovery();

    let result = run_event_loop(&mut terminal, &mut app);

    disable_raw_mode().ok();
    execute!(terminal.backend_mut(), LeaveAlternateScreen).ok();
    terminal.show_cursor().ok();

    result
}

fn run_event_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    while app.running {
        terminal.draw(|frame| ui::render(frame, app))?;

        if event::poll(Duration::from_millis(150))? {
            if let Event::Key(key_event) = event::read()? {
                ui::handle_key(app, key_event);
            }
        }
    }
    Ok(())
}
