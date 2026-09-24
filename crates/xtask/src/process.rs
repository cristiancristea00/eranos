//! Running the programs the gates call.

use std::process::{Command, ExitStatus, Stdio};

use crate::Failure;

/// The workspace root, where every step runs, whatever directory
/// `cargo xtask` was started from.
const WORKSPACE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// A program a gate runs, with its arguments and the environment it needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Step {
    arguments: Vec<String>,
    name: String,
    program: String,
    environment: Vec<Variable>,
}

/// An environment variable a step sets for its program.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Variable {
    name: String,
    value: String,
}

impl Step {
    /// A step that runs cargo with `arguments`, reported as `name`.
    pub(crate) fn cargo(arguments: &[&str], name: &str) -> Self {
        Self::new(arguments, name, "cargo")
    }

    /// A step that runs `program` with `arguments`, reported as `name`.
    pub(crate) fn new(arguments: &[&str], name: &str, program: &str) -> Self {
        Self {
            arguments: arguments
                .iter()
                .map(|&argument| argument.to_owned())
                .collect(),
            name: name.to_owned(),
            program: program.to_owned(),
            environment: Vec::new(),
        }
    }

    /// The same step, also setting the environment variable `name` to
    /// `value`.
    #[must_use]
    pub(crate) fn with_variable(mut self, name: &str, value: &str) -> Self {
        self.environment.push(Variable {
            name: name.to_owned(),
            value: value.to_owned(),
        });
        self
    }

    /// The command line as a shell would take it, for the heading a step
    /// prints before it runs.
    pub(crate) fn command_line(&self) -> String {
        self.environment
            .iter()
            .map(|variable| format!("{}='{}'", variable.name, variable.value))
            .chain([self.program.clone()])
            .chain(self.arguments.iter().cloned())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The process that runs the step: from the workspace root, with
    /// standard input closed, so no step can wait for a keyboard.
    fn command(&self) -> Command {
        let mut command = Command::new(&self.program);
        command
            .args(&self.arguments)
            .envs(
                self.environment
                    .iter()
                    .map(|variable| (&variable.name, &variable.value)),
            )
            .current_dir(WORKSPACE)
            .stdin(Stdio::null());
        command
    }

    /// Succeeds when `status` reports success, and fails as this step
    /// otherwise.
    fn check(&self, status: ExitStatus) -> Result<(), Failure> {
        if status.success() {
            Ok(())
        } else {
            Err(Failure::Step {
                status,
                step: self.name.clone(),
            })
        }
    }

    /// The failure of a step whose program could not be started.
    fn not_started(&self, source: std::io::Error) -> Failure {
        Failure::Start {
            source,
            step: self.name.clone(),
        }
    }
}

/// Runs steps. The gates take a runner, so their tests record the steps
/// instead of running them.
pub(crate) trait Runner {
    /// Runs `step` with its output shown.
    ///
    /// # Errors
    ///
    /// [`Failure::Start`] when its program cannot be started and
    /// [`Failure::Step`] when it does not succeed.
    fn run(&mut self, step: &Step) -> Result<(), Failure>;

    /// Runs `step` and returns its standard output; its standard error is
    /// shown.
    ///
    /// # Errors
    ///
    /// [`Failure::Start`] when its program cannot be started and
    /// [`Failure::Step`] when it does not succeed.
    fn output(&mut self, step: &Step) -> Result<String, Failure>;
}

/// Runs steps as processes.
#[derive(Debug)]
pub(crate) struct System;

impl Runner for System {
    fn run(&mut self, step: &Step) -> Result<(), Failure> {
        println!("\n==> {}: {}", step.name, step.command_line());
        let status = step
            .command()
            .status()
            .map_err(|source| step.not_started(source))?;
        step.check(status)
    }

    fn output(&mut self, step: &Step) -> Result<String, Failure> {
        let output = step
            .command()
            .stderr(Stdio::inherit())
            .output()
            .map_err(|source| step.not_started(source))?;
        step.check(output.status)?;
        // Cargo's JSON is UTF-8; anything else fails as the JSON it is not.
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

#[cfg(test)]
mod tests;
