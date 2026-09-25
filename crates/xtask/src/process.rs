//! Running the programs the gates call.

use std::fs;
use std::io::ErrorKind;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use crate::Failure;

/// The workspace root, where every step runs unless it names a directory
/// under it, whatever directory `cargo xtask` was started from.
pub(crate) const WORKSPACE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

/// A program a gate runs, with its arguments and the environment it needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Step {
    arguments: Vec<String>,
    name: String,
    program: String,
    /// The directory the program runs in, under the workspace root; the root
    /// itself when absent.
    directory: Option<String>,
    environment: Vec<Variable>,
}

/// An environment variable a step sets for its program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Variable {
    name: String,
    value: String,
}

impl Variable {
    /// The variable `name`, set to `value`.
    pub(crate) fn new(name: &str, value: &str) -> Self {
        Self {
            name: name.to_owned(),
            value: value.to_owned(),
        }
    }
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
            directory: None,
            environment: Vec::new(),
        }
    }

    /// The same step, run in `directory`, under the workspace root.
    #[must_use]
    pub(crate) fn in_directory(mut self, directory: &str) -> Self {
        self.directory = Some(directory.to_owned());
        self
    }

    /// The same step, also setting the environment variable `name` to
    /// `value`.
    #[must_use]
    pub(crate) fn with_variable(mut self, name: &str, value: &str) -> Self {
        self.environment.push(Variable::new(name, value));
        self
    }

    /// The same step, also setting every variable of `environment`.
    #[must_use]
    pub(crate) fn with_environment(mut self, environment: &[Variable]) -> Self {
        self.environment.extend_from_slice(environment);
        self
    }

    /// The command line as a shell would take it, for the heading a step
    /// prints before it runs.
    pub(crate) fn command_line(&self) -> String {
        let command = self
            .environment
            .iter()
            .map(|variable| format!("{}='{}'", variable.name, variable.value))
            .chain([self.program.clone()])
            .chain(self.arguments.iter().cloned())
            .collect::<Vec<_>>()
            .join(" ");
        match &self.directory {
            Some(directory) => format!("cd {directory} && {command}"),
            None => command,
        }
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
            .current_dir(Path::new(WORKSPACE).join(self.directory.as_deref().unwrap_or_default()))
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

    /// Removes `directory` and everything in it; a missing directory is
    /// already clear.
    ///
    /// # Errors
    ///
    /// [`Failure::Clear`] when the directory is there and cannot be removed.
    fn clear(&mut self, directory: &Path) -> Result<(), Failure>;

    /// Creates `directory` and every directory above it that is missing; one
    /// that is there is left as it is.
    ///
    /// # Errors
    ///
    /// [`Failure::Create`] when a directory cannot be created.
    fn create(&mut self, directory: &Path) -> Result<(), Failure>;

    /// Writes `contents` to the file at `path`, replacing what it held.
    ///
    /// # Errors
    ///
    /// [`Failure::Write`] when the file cannot be written.
    fn write(&mut self, path: &Path, contents: &str) -> Result<(), Failure>;

    /// Whether the machine builds for Apple platforms, which only Xcode's
    /// tools on macOS do; the Swift package's gates run only where it does.
    fn builds_apple_platforms(&self) -> bool;

    /// The value of the environment variable `name`; `None` when it is not
    /// set or is empty.
    fn variable(&self, name: &str) -> Option<String>;
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

    fn clear(&mut self, directory: &Path) -> Result<(), Failure> {
        println!("\n==> clearing {}", directory.display());
        match fs::remove_dir_all(directory) {
            Err(error) if ErrorKind::NotFound != error.kind() => Err(Failure::Clear {
                directory: directory.display().to_string(),
                source: error,
            }),
            _ => Ok(()),
        }
    }

    fn create(&mut self, directory: &Path) -> Result<(), Failure> {
        println!("\n==> creating {}", directory.display());
        fs::create_dir_all(directory).map_err(|source| Failure::Create {
            directory: directory.display().to_string(),
            source,
        })
    }

    fn write(&mut self, path: &Path, contents: &str) -> Result<(), Failure> {
        fs::write(path, contents).map_err(|source| Failure::Write {
            path: path.display().to_string(),
            source,
        })
    }

    fn builds_apple_platforms(&self) -> bool {
        cfg!(target_os = "macos")
    }

    fn variable(&self, name: &str) -> Option<String> {
        std::env::var_os(name)
            .filter(|value| !value.is_empty())
            .map(|value| value.to_string_lossy().into_owned())
    }
}

#[cfg(test)]
mod tests;
