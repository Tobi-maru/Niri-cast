use std::process::{Child, Command, Stdio};

use anyhow::{anyhow, Context};

#[derive(Debug, Default)]
pub struct WlMirrorAdapter {
    child: Option<Child>,
}

impl WlMirrorAdapter {
    pub fn start(&mut self, source_output: &str, fullscreen_output: &str) -> anyhow::Result<()> {
        self.stop()?;

        let child = Command::new("wl-mirror")
            .args([
                "--fullscreen-output",
                fullscreen_output,
                "--fullscreen",
                source_output,
            ])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .context("failed to start `wl-mirror`")?;

        self.child = Some(child);
        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        let mut child = match self.child.take() {
            Some(child) => child,
            None => return Ok(()),
        };

        if child.try_wait()?.is_some() {
            return Ok(());
        }

        child.kill().context("failed to stop wl-mirror process")?;
        let status = child.wait().context("failed to wait for wl-mirror to exit")?;
        if status.success() {
            Ok(())
        } else {
            Err(anyhow!("wl-mirror exited with non-zero status while stopping"))
        }
    }
}
