use super::OutputFormat;
use super::cli_output_value::CliOutputValue;
use super::facet_cli_output::FacetCliOutput;
use eyre::Context;
use facet::Facet;
use std::io::IsTerminal;
use std::io::Write;
use std::io::{self};

pub struct CliOutput {
    value: Option<Box<dyn CliOutputValue>>,
    pub elapsed_duration_warn_threshold: Option<std::time::Duration>,
}

impl core::fmt::Debug for CliOutput {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CliOutput")
            .field("has_value", &self.value.is_some())
            .field(
                "elapsed_duration_warn_threshold",
                &self.elapsed_duration_warn_threshold,
            )
            .finish()
    }
}

impl CliOutput {
    #[must_use]
    pub const fn none(elapsed_duration_warn_threshold: Option<std::time::Duration>) -> Self {
        Self {
            value: None,
            elapsed_duration_warn_threshold,
        }
    }

    #[must_use]
    pub fn facet<T>(value: T, elapsed_duration_warn_threshold: Option<std::time::Duration>) -> Self
    where
        T: Facet<'static> + 'static,
    {
        Self {
            value: Some(Box::new(FacetCliOutput { value })),
            elapsed_duration_warn_threshold,
        }
    }

    /// # Errors
    ///
    /// This function will return an error if the selected output format cannot be rendered
    /// or if the rendered output cannot be written to stdout.
    pub fn emit(self, requested_format: Option<OutputFormat>) -> eyre::Result<()> {
        let Some(output) = self.value else {
            return Ok(());
        };

        let stdout_is_terminal = io::stdout().is_terminal();
        let format = requested_format.unwrap_or(if stdout_is_terminal {
            OutputFormat::Text
        } else {
            OutputFormat::Json
        });
        let Some(rendered) = output.render(format, stdout_is_terminal)? else {
            return Ok(());
        };

        let mut stdout = io::stdout().lock();
        stdout
            .write_all(rendered.as_bytes())
            .wrap_err("failed to write command output")?;
        if !rendered.ends_with('\n') {
            stdout
                .write_all(b"\n")
                .wrap_err("failed to terminate command output")?;
        }
        stdout.flush().wrap_err("failed to flush command output")?;
        Ok(())
    }
}

#[cfg(test)]
mod budget_tests {
    use super::*;
    #[derive(Facet)]
    struct Fixture {
        count: u32,
    }
    #[test]
    fn budget_is_not_serialized() {
        for threshold in [None, Some(std::time::Duration::from_secs(1))] {
            let output = CliOutput::facet(Fixture { count: 7 }, threshold);
            assert_eq!(output.elapsed_duration_warn_threshold, threshold);
            let json = output
                .value
                .unwrap()
                .render(OutputFormat::Json, false)
                .unwrap()
                .unwrap();
            assert_eq!(json, "{\n  \"count\": 7\n}");
        }
        assert!(CliOutput::none(None).value.is_none());
    }
}
