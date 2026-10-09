//! What the JVM packages share: their Gradle runs, and how the JVM starts
//! the port of an example.

use crate::gates::Metadata;
use crate::process::Step;

/// The step that runs Gradle's `tasks` in `project`, a directory under the
/// workspace root, reported as `name`, with every deprecation warning an
/// error, so a build that the next Gradle refuses fails now.
#[must_use]
pub(crate) fn gradle(project: &str, tasks: &[&str], name: &str) -> Step {
    let mut arguments = vec!["gradlew", "--warning-mode=fail"];
    arguments.extend(tasks);
    Step::new(&arguments, name, "sh").in_directory(project)
}

/// The step that checks the format of the files of `project` that `files`
/// match with ktlint, reported as `name`.
#[must_use]
pub(crate) fn ktlint(project: &str, files: &[&str], name: &str) -> Step {
    let mut arguments = vec!["--relative"];
    arguments.extend(files);
    Step::new(&arguments, name, "ktlint").in_directory(project)
}

/// The step that applies ktlint's format to the files of `project` that
/// `files` match, reported as `name`.
#[must_use]
pub(crate) fn ktlint_format(project: &str, files: &[&str], name: &str) -> Step {
    let mut arguments = vec!["--format", "--relative"];
    arguments.extend(files);
    Step::new(&arguments, name, "ktlint").in_directory(project)
}

/// The step that runs Gradle's `tasks` as [`gradle`] does, with the tests
/// loading the host library from `metadata`'s target directory.
#[must_use]
pub(crate) fn gradle_on_library(
    project: &str,
    metadata: &Metadata,
    tasks: &[&str],
    name: &str,
) -> Step {
    let library = format!("-Peranos.library={}/debug", metadata.target_directory);
    let mut arguments = vec![library.as_str()];
    arguments.extend(tasks);
    gradle(project, &arguments, name)
}

/// Where a JVM language's ports are gathered, and how the JVM starts one.
pub(crate) struct Jvm {
    /// The class that holds a port's `main`, in the package
    /// `eranos.examples.<example>`, named without underscores.
    pub(crate) class: &'static str,
    /// The language, as a report that differs names it.
    pub(crate) language: &'static str,
    /// The system property that names the directory the library loads from.
    pub(crate) library_path: &'static str,
    /// The directory, from the workspace root, that holds the ports and
    /// every library they run with.
    pub(crate) ports: &'static str,
}

/// The step that runs the port of `example` in `jvm`'s language on the
/// `java` of the JDK at `jdk`, the library loading from `sequential`'s
/// target directory. The port calls native code from the class path, which
/// the launcher grants native access, and refuses it to any code the
/// command line does not grant it to, as the Java tests run. It writes its
/// report in UTF-8, as the Rust example does, whatever the machine's locale,
/// from which the JVM would otherwise take the encoding of its output.
#[must_use]
pub(crate) fn port(jvm: &Jvm, example: &str, jdk: &str, sequential: &Metadata) -> Step {
    let library = format!(
        "-D{}={}/debug",
        jvm.library_path, sequential.target_directory
    );
    let classes = format!("{}/*", jvm.ports);
    let class = format!("eranos.examples.{}.{}", example.replace('_', ""), jvm.class);
    let arguments = [
        "--enable-native-access=ALL-UNNAMED",
        "--illegal-native-access=deny",
        "-Dstdout.encoding=UTF-8",
        &library,
        "-cp",
        &classes,
        &class,
    ];
    Step::new(
        &arguments,
        &format!("the {} port of {example}", jvm.language),
        &format!("{jdk}/bin/java"),
    )
}

#[cfg(test)]
mod tests;
