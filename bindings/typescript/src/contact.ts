/**
 * The ways to reach and pay a member, and the language a group is shown in.
 *
 * @packageDocumentation
 */

import { calling } from './converting.js';
import * as internal from './internal/eranos.js';

/** An email address, checked by the library's rules. */
export class Email {
    private constructor(
        /** The address, as entered but without surrounding whitespace. */
        private readonly address: string,
    ) {}

    /**
     * The address written `text`.
     *
     * @param text - The address's text
     * @returns The address
     * @throws {@link ContactError} with the failure `invalidEmail` for text that is not a local part and a domain around an
     *   `@`
     */
    static parse(text: string): Email {
        return new Email(calling(() => internal.emailParse(text)));
    }

    /**
     * The address the library wrote as `text`.
     *
     * @internal
     */
    static known(text: string): Email {
        return new Email(text);
    }

    /**
     * Whether `other` is the same address.
     *
     * @param other - The address to compare
     * @returns Whether the two are equal
     */
    equals(other: Email): boolean {
        return this.address === other.address;
    }

    /**
     * The address, as entered but without surrounding whitespace.
     *
     * @returns The text
     */
    toString(): string {
        return this.address;
    }
}

/** A phone number, valid in its country's numbering plan. */
export class PhoneNumber {
    private constructor(
        /** The number in E.164 form: "+40712345678". */
        readonly e164: string,
    ) {}

    /**
     * The number written `text`, starting with `+` and the country code.
     *
     * @param text - The number's text
     * @returns The number
     * @throws {@link ContactError} with the failure `invalidPhoneNumber`
     */
    static parse(text: string): PhoneNumber {
        return new PhoneNumber(calling(() => internal.phoneNumberParse(text)));
    }

    /**
     * The number the library wrote as `e164`.
     *
     * @internal
     */
    static known(e164: string): PhoneNumber {
        return new PhoneNumber(e164);
    }

    /**
     * Whether `other` is the same number.
     *
     * @param other - The number to compare
     * @returns Whether the two are equal
     */
    equals(other: PhoneNumber): boolean {
        return this.e164 === other.e164;
    }

    /**
     * The number grouped as its country groups it.
     *
     * @returns The text: "+40 712 345 678"
     */
    toString(): string {
        try {
            return internal.phoneNumberInternational(this.e164);
        } catch {
            return this.e164;
        }
    }
}

/** An international bank account number, checked against the SWIFT IBAN registry. */
export class Iban {
    private constructor(
        /** The IBAN without spaces, in capitals. */
        private readonly text: string,
    ) {}

    /**
     * The IBAN written `text`, with or without spaces, in any letter case.
     *
     * @param text - The IBAN's text
     * @returns The IBAN
     * @throws {@link AccountError} with the failure `invalidIban` and the check it failed
     */
    static parse(text: string): Iban {
        return new Iban(calling(() => internal.ibanParse(text)));
    }

    /**
     * The IBAN the library wrote as `text`.
     *
     * @internal
     */
    static known(text: string): Iban {
        return new Iban(text);
    }

    /** The two-letter country code: "RO". */
    get countryCode(): string {
        try {
            return internal.ibanCountryCode(this.text);
        } catch {
            return this.text.slice(0, 2);
        }
    }

    /**
     * Whether `other` is the same IBAN.
     *
     * @param other - The IBAN to compare
     * @returns Whether the two are equal
     */
    equals(other: Iban): boolean {
        return this.text === other.text;
    }

    /**
     * The IBAN without spaces, in capitals.
     *
     * @returns The text: "RO49AAAA1B31007593840000"
     */
    toString(): string {
        return this.text;
    }
}

/** A username in a payment app. */
export class Username {
    private constructor(
        /** The username, as entered but without surrounding whitespace. */
        private readonly text: string,
    ) {}

    /**
     * The username written `text`.
     *
     * @param text - The username's text
     * @returns The username
     * @throws {@link AccountError} with the failure `emptyUsername` for blank text
     */
    static parse(text: string): Username {
        return new Username(calling(() => internal.usernameParse(text)));
    }

    /**
     * The username the library wrote as `text`.
     *
     * @internal
     */
    static known(text: string): Username {
        return new Username(text);
    }

    /**
     * Whether `other` is the same username.
     *
     * @param other - The username to compare
     * @returns Whether the two are equal
     */
    equals(other: Username): boolean {
        return this.text === other.text;
    }

    /**
     * The username, as entered but without surrounding whitespace.
     *
     * @returns The text
     */
    toString(): string {
        return this.text;
    }
}

/** A language the apps show a group in: a BCP 47 tag, such as "pt-PT". */
export class Language {
    private constructor(
        /** The tag in its canonical case. */
        private readonly tag: string,
    ) {}

    /**
     * The language of the tag `text`, in any letter case: "pt-pt".
     *
     * @param text - The tag
     * @returns The language
     * @throws {@link LanguageError} with the failure `invalidLanguage` for text that is not a BCP 47 tag
     */
    static parse(text: string): Language {
        return new Language(calling(() => internal.languageParse(text)));
    }

    /**
     * The language the library wrote as `text`.
     *
     * @internal
     */
    static known(text: string): Language {
        return new Language(text);
    }

    /**
     * Whether `other` is the same language.
     *
     * @param other - The language to compare
     * @returns Whether the two are equal
     */
    equals(other: Language): boolean {
        return this.tag === other.tag;
    }

    /**
     * The tag in its canonical case.
     *
     * @returns The text: "pt-PT"
     */
    toString(): string {
        return this.tag;
    }
}
