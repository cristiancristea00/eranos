package eranos;

import eranos.internal.Eranos;

/** A language the apps show a group in: a BCP 47 tag, such as "pt-PT". */
public final class Language {
    private final String text;

    private Language(String text) {
        this.text = text;
    }

    /**
     * The language of the tag {@code text}, in any letter case: "pt-pt".
     *
     * @param text the tag
     * @return the language
     * @throws EranosException.Language with {@link LanguageFailure.InvalidLanguage} for text that
     *     is not a BCP 47 tag
     */
    public static Language parse(String text) throws EranosException {
        return Converting.call(() -> new Language(Eranos.languageParse(text)));
    }

    /**
     * The language the library wrote as {@code text}.
     *
     * @param text the tag
     * @return the language
     */
    static Language known(String text) {
        return new Language(text);
    }

    @Override
    public boolean equals(Object other) {
        return other instanceof Language language && text.equals(language.text);
    }

    @Override
    public int hashCode() {
        return text.hashCode();
    }

    /**
     * The tag in its canonical case: "pt-PT".
     *
     * @return the text
     */
    @Override
    public String toString() {
        return text;
    }
}
