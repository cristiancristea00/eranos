//! The languages: each named once on the command line, and read back from
//! its name.

use super::*;

/// A command line whose language read as another would check the wrong
/// package: every name reads back as its language, and a name no language
/// has reads as none.
#[test]
fn every_language_reads_back_from_its_name() {
    for language in Language::ALL {
        assert_eq!(Language::parse(language.name()), Some(language));
        assert_eq!(language.to_string(), language.name());
    }
    assert_eq!(Language::parse("typescript"), None);
    assert_eq!(Language::parse("all"), None);
}

/// A command that skipped the host library and its generated sources would
/// test a package against stale bindings, and one that built them for every
/// language would spend a minute on a package that never reads them: the
/// packages over eranos-uniffi read them, and the others build their own.
#[test]
fn the_packages_over_eranos_uniffi_read_its_host_library() {
    let readers: Vec<Language> = Language::ALL
        .into_iter()
        .filter(|language| language.reads_uniffi())
        .collect();
    assert_eq!(
        readers,
        [
            Language::Java,
            Language::Kotlin,
            Language::Python,
            Language::Swift
        ]
    );
}
