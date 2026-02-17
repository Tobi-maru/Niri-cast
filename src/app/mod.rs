use std::time::Duration;

use anyhow::Context;
use crossterm::event::{self, Event};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::adapters::{
    audio::{AudioAdapter, AudioSink},
    niri::NiriAdapter,
    portal::PortalAdapter,
    system::SystemAdapter,
    wl_mirror::WlMirrorAdapter,
};
use crate::diagnostics::{run_troubleshooting, TroubleshootReport};
use crate::profiles::{ProfileStore, TvProfile};
use crate::ui;

pub struct App {
    pub selected_tab: usize,
    pub running: bool,
    pub log_lines: Vec<String>,
    pub last_outputs: Vec<String>,
    pub audio_sinks: Vec<AudioSink>,
    pub selected_audio_sink: usize,
    pub diagnostics: Option<TroubleshootReport>,
    pub profile_store: ProfileStore,
    pub niri: NiriAdapter,
    pub audio: AudioAdapter,
    pub system: SystemAdapter,
    pub portal: PortalAdapter,
    pub wl_mirror: WlMirrorAdapter,
}

impl App {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            selected_tab: 0,
            running: true,
            log_lines: vec!["niri-cast started".to_string()],
            last_outputs: Vec::new(),
            audio_sinks: Vec::new(),
            selected_audio_sink: 0,
            diagnostics: None,
            profile_store: ProfileStore::new()?,
            niri: NiriAdapter::default(),
            audio: AudioAdapter::default(),
            system: SystemAdapter::default(),
            portal: PortalAdapter::default(),
            wl_mirror: WlMirrorAdapter::default(),
        })
    }

    pub fn shutdown(&mut self) {
        if let Err(err) = self.wl_mirror.stop() {
            self.log(format!("failed to stop wl-mirror: {err}"));
        }
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
        self.refresh_outputs();
        self.refresh_audio_sinks();
        self.log(format!(
            "discovered {} outputs, {} sinks",
            self.last_outputs.len(),
            self.audio_sinks.len()
        ));
    }

    fn refresh_outputs(&mut self) {
        self.last_outputs = self.niri.list_outputs().unwrap_or_default();
    }

    fn refresh_audio_sinks(&mut self) {
        self.audio_sinks = self.audio.list_sink_objects().unwrap_or_default();
        if self.audio_sinks.is_empty() {
            self.selected_audio_sink = 0;
        } else if self.selected_audio_sink >= self.audio_sinks.len() {
            self.selected_audio_sink = self.audio_sinks.len() - 1;
        }
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
        self.refresh_audio_sinks();
    }

    pub fn select_next_audio_sink(&mut self) {
        if self.audio_sinks.is_empty() {
            self.log("no audio sinks discovered");
            return;
        }
        self.selected_audio_sink = (self.selected_audio_sink + 1) % self.audio_sinks.len();
        let sink = &self.audio_sinks[self.selected_audio_sink];
        self.log(format!("selected sink: {}. {}", sink.id, sink.name));
    }

    pub fn select_prev_audio_sink(&mut self) {
        if self.audio_sinks.is_empty() {
            self.log("no audio sinks discovered");
            return;
        }
        if self.selected_audio_sink == 0 {
            self.selected_audio_sink = self.audio_sinks.len() - 1;
        } else {
            self.selected_audio_sink -= 1;
        }
        let sink = &self.audio_sinks[self.selected_audio_sink];
        self.log(format!("selected sink: {}. {}", sink.id, sink.name));
    }

    pub fn apply_selected_audio_sink(&mut self) {
        if self.audio_sinks.is_empty() {
            self.log("no audio sinks available to apply");
            return;
        }

        let sink = self.audio_sinks[self.selected_audio_sink].clone();
        match self.audio.set_default_and_move_streams_by_id(&sink.id) {
            Ok(moved) => self.log(format!(
                "set default audio sink: {}. {} (moved {} active stream(s))",
                sink.id, sink.name, moved
            )),
            Err(err) => self.log(format!("failed to switch audio sink: {err}")),
        }
        self.refresh_audio_sinks();
    }

    pub fn switch_to_laptop_audio(&mut self) {
        self.refresh_audio_sinks();
        let target = self
            .audio_sinks
            .iter()
            .find(|sink| !sink.is_hdmi)
            .cloned();
        match target {
            Some(sink) => {
                match self.audio.set_default_and_move_streams_by_id(&sink.id) {
                    Ok(moved) => self.log(format!(
                        "switched to laptop audio: {} (moved {} active stream(s))",
                        sink.name, moved
                    )),
                    Err(err) => self.log(format!("failed to switch to laptop audio: {err}")),
                }
                self.refresh_audio_sinks();
            }
            None => {
                match self.audio.try_switch_card_profile_to_laptop() {
                    Ok(true) => {
                        self.refresh_audio_sinks();
                        if let Some(sink) = self
                            .audio_sinks
                            .iter()
                            .find(|sink| !sink.is_hdmi)
                            .cloned()
                        {
                            match self.audio.set_default_and_move_streams_by_id(&sink.id) {
                                Ok(moved) => self.log(format!(
                                    "switched to laptop audio: {} (moved {} active stream(s))",
                                    sink.name, moved
                                )),
                                Err(err) => {
                                    self.log(format!("failed to set laptop audio sink: {err}"))
                                }
                            }
                            self.refresh_audio_sinks();
                        } else {
                            self.log("switched profile, but no laptop sink was exposed");
                        }
                    }
                    Ok(false) => self.log(
                        "no non-HDMI sink available; could not switch card profile to laptop audio",
                    ),
                    Err(err) => self.log(format!("laptop profile switch failed: {err}")),
                }
            }
        }
    }

    pub fn switch_to_tv_audio(&mut self) {
        self.refresh_audio_sinks();
        let target = self.audio_sinks.iter().find(|sink| sink.is_hdmi).cloned();
        match target {
            Some(sink) => {
                match self.audio.set_default_and_move_streams_by_id(&sink.id) {
                    Ok(moved) => self.log(format!(
                        "switched to TV audio: {} (moved {} active stream(s))",
                        sink.name, moved
                    )),
                    Err(err) => self.log(format!("failed to switch to TV audio: {err}")),
                }
                self.refresh_audio_sinks();
            }
            None => {
                match self.audio.try_switch_card_profile_to_tv() {
                    Ok(true) => {
                        self.refresh_audio_sinks();
                        if let Some(sink) = self.audio_sinks.iter().find(|sink| sink.is_hdmi).cloned()
                        {
                            match self.audio.set_default_and_move_streams_by_id(&sink.id) {
                                Ok(moved) => self.log(format!(
                                    "switched to TV audio: {} (moved {} active stream(s))",
                                    sink.name, moved
                                )),
                                Err(err) => self.log(format!("failed to set TV audio sink: {err}")),
                            }
                            self.refresh_audio_sinks();
                        } else {
                            self.log("switched profile, but no HDMI sink was exposed");
                        }
                    }
                    Ok(false) => {
                        self.log("no HDMI sink available; could not switch card profile to TV audio")
                    }
                    Err(err) => self.log(format!("TV profile switch failed: {err}")),
                }
            }
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
        if !matches!(mode, LayoutCastMode::Mirror) {
            self.wl_mirror.stop()?;
        }

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

        let non_hdmi_primary = outputs
            .iter()
            .find(|o| !o.name.to_ascii_uppercase().contains("HDMI"));

        match mode {
            LayoutCastMode::ExtendRight => {
                let primary = non_hdmi_primary.unwrap_or(hdmi_output);
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
                let primary = non_hdmi_primary.unwrap_or(hdmi_output);
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
                let primary = non_hdmi_primary
                    .ok_or_else(|| anyhow::anyhow!("no non-HDMI source output available"))?;
                self.niri.output_on(&primary.name)?;
                self.wl_mirror.start(&primary.name, &hdmi_output.name)?;
                Ok(format!(
                    "cast mode set: wl-mirror (source={}, target={})",
                    primary.name, hdmi_output.name
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
    app.shutdown();

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

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                ui::handle_key(app, key_event);
            }
        }
    }
    Ok(())
}
