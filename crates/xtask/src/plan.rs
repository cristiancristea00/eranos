//! What a package's gates do, as data. Each package describes its gates as
//! actions, each in the phase it belongs to: the gates run every action in
//! order, and a command for one package's tests, linters, or packages runs
//! its preparing actions and the phase it names.

use std::collections::VecDeque;
use std::path::PathBuf;

use crate::Failure;
use crate::process::{Runner, Step};

/// The part of a package's gates that an action belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Phase {
    /// The documentation, built with warnings as errors.
    Docs,
    /// The formatters' checks and the linters.
    Lint,
    /// The packages an app consumes.
    Package,
    /// What the other phases read: the targets, the libraries, and the
    /// generated sources.
    Prepare,
    /// The tests and their coverage.
    Test,
}

/// A continuation of a read: the actions that follow from the step's
/// standard output.
type Continuation = Box<dyn FnOnce(String) -> Result<Vec<Action>, Failure>>;

/// What an action does.
pub(crate) enum Work {
    /// Removes a directory and everything in it; a missing one is already
    /// clear.
    Clear(PathBuf),
    /// Creates a directory and every directory above it that is missing.
    Create(PathBuf),
    /// Runs a step and hands its standard output to the continuation, whose
    /// actions run next.
    Read(Step, Continuation),
    /// Runs a step.
    Run(Step),
    /// Writes a file, replacing what it held.
    Write(PathBuf, String),
}

/// One piece of a package's gates.
pub(crate) struct Action {
    /// The phase it belongs to.
    pub(crate) phase: Phase,
    /// What it does.
    pub(crate) work: Work,
}

impl Action {
    /// The action that removes `directory` in `phase`.
    pub(crate) const fn clear(phase: Phase, directory: PathBuf) -> Self {
        Self {
            phase,
            work: Work::Clear(directory),
        }
    }

    /// The action that creates `directory` in `phase`.
    pub(crate) const fn create(phase: Phase, directory: PathBuf) -> Self {
        Self {
            phase,
            work: Work::Create(directory),
        }
    }

    /// The action that runs `step` in `phase` and hands its output to
    /// `then`, whose actions run next.
    pub(crate) fn read(
        phase: Phase,
        step: Step,
        then: impl FnOnce(String) -> Result<Vec<Self>, Failure> + 'static,
    ) -> Self {
        Self {
            phase,
            work: Work::Read(step, Box::new(then)),
        }
    }

    /// The action that runs `step` in `phase`.
    pub(crate) const fn run(phase: Phase, step: Step) -> Self {
        Self {
            phase,
            work: Work::Run(step),
        }
    }

    /// The actions that run each of `steps` in `phase`, in order.
    pub(crate) fn runs(phase: Phase, steps: Vec<Step>) -> Vec<Self> {
        steps
            .into_iter()
            .map(|step| Self::run(phase, step))
            .collect()
    }

    /// The action that writes `contents` to the file at `path` in `phase`.
    pub(crate) const fn write(phase: Phase, path: PathBuf, contents: String) -> Self {
        Self {
            phase,
            work: Work::Write(path, contents),
        }
    }
}

/// Runs, with `runner`, every action of `actions` whose phase `selected`
/// takes, in order, the actions a read gives right after it, stopping at the
/// first that fails. `selected` is a trait object, so every caller runs the
/// same instantiation: a closure's type is its own, and the coverage report
/// keeps the best single instantiation of a generic function, which would
/// leave out a branch that only one caller's selection takes.
///
/// # Errors
///
/// The failure of the first action that fails.
pub(crate) fn execute(
    runner: &mut impl Runner,
    actions: Vec<Action>,
    selected: &dyn Fn(Phase) -> bool,
) -> Result<(), Failure> {
    let mut queue = VecDeque::from(actions);
    while let Some(action) = queue.pop_front() {
        if !selected(action.phase) {
            continue;
        }
        match action.work {
            Work::Clear(directory) => runner.clear(&directory)?,
            Work::Create(directory) => runner.create(&directory)?,
            Work::Read(step, then) => {
                let output = runner.output(&step)?;
                for next in then(output)?.into_iter().rev() {
                    queue.push_front(next);
                }
            }
            Work::Run(step) => runner.run(&step)?,
            Work::Write(path, contents) => runner.write(&path, &contents)?,
        }
    }
    Ok(())
}

/// The steps `actions` run or read, in order, without the actions a read's
/// continuation gives: what a test compares a package's description with.
#[cfg(test)]
pub(crate) fn steps(actions: &[Action]) -> Vec<Step> {
    actions
        .iter()
        .filter_map(|action| match &action.work {
            Work::Read(step, _) | Work::Run(step) => Some(step.clone()),
            Work::Clear(_) | Work::Create(_) | Work::Write(..) => None,
        })
        .collect()
}

#[cfg(test)]
mod tests;
