//! The languages the commands name: the workspace's Rust and the package of
//! each binding.

use std::fmt;

/// A language whose checks a command runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Language {
    /// The C and C++ packages, over eranos-capi, which build together, and
    /// which a command line calls `c`.
    Capi,
    /// The Java package.
    Java,
    /// The Kotlin Multiplatform package.
    Kmp,
    /// The Kotlin package, for Android and the JVM.
    Kotlin,
    /// The Python package.
    Python,
    /// The workspace's crates.
    Rust,
    /// The Swift package, which builds only where the machine builds for
    /// Apple platforms.
    Swift,
    /// The TypeScript package, over eranos-web.
    Web,
}

impl Language {
    /// Every language.
    pub(crate) const ALL: [Self; 8] = [
        Self::Capi,
        Self::Java,
        Self::Kmp,
        Self::Kotlin,
        Self::Python,
        Self::Rust,
        Self::Swift,
        Self::Web,
    ];

    /// The language named `name` on a command line, or none for a name no
    /// language has.
    #[must_use]
    pub(crate) fn parse(name: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|language| language.name() == name)
    }

    /// The language's name on a command line.
    #[must_use]
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Capi => "c",
            Self::Java => "java",
            Self::Kmp => "kmp",
            Self::Kotlin => "kotlin",
            Self::Python => "python",
            Self::Rust => "rust",
            Self::Swift => "swift",
            Self::Web => "web",
        }
    }

    /// Whether its package's checks read the host library of eranos-uniffi
    /// and the sources generated from it, which a command builds first.
    #[must_use]
    pub(crate) const fn reads_uniffi(self) -> bool {
        match self {
            Self::Capi | Self::Kmp | Self::Rust | Self::Web => false,
            Self::Java | Self::Kotlin | Self::Python | Self::Swift => true,
        }
    }
}

impl fmt::Display for Language {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

#[cfg(test)]
mod tests;
