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

/**
 * A phone number, valid in its country's numbering plan.
 *
 * A number holds its E.164 form and its international form as the library writes them, so it prints without a call into
 * the library.
 */
export class PhoneNumber {
    private constructor(
        /** The number in E.164 form: "+40712345678". */
        readonly e164: string,
        /** The number in international form, grouped as its country groups it: "+40 712 345 678". */
        readonly international: string,
    ) {}

    /**
     * The number written `text`, starting with `+` and the country code.
     *
     * @param text - The number's text
     * @returns The number
     * @throws {@link ContactError} with the failure `invalidPhoneNumber`
     */
    static parse(text: string): PhoneNumber {
        return PhoneNumber.converted(calling(() => internal.phoneNumberParse(text)));
    }

    /**
     * The number the generated code holds as `phone`.
     *
     * @internal
     */
    static converted(phone: internal.PhoneNumber): PhoneNumber {
        return new PhoneNumber(phone.e164, phone.international);
    }

    /**
     * The number as the generated code holds it.
     *
     * @internal
     */
    generated(): internal.PhoneNumber {
        return internal.PhoneNumber.create({ e164: this.e164, international: this.international });
    }

    /**
     * Whether `other` is the same number, in both of its forms.
     *
     * @param other - The number to compare
     * @returns Whether the two are equal
     */
    equals(other: PhoneNumber): boolean {
        return this.e164 === other.e164 && this.international === other.international;
    }

    /**
     * The number grouped as its country groups it: its international form.
     *
     * @returns The text: "+40 712 345 678"
     */
    toString(): string {
        return this.international;
    }
}

/**
 * An international bank account number, checked against the SWIFT IBAN registry.
 *
 * An IBAN holds its text and its country code as the library writes them, so it gives its country without a call into the
 * library.
 */
export class Iban {
    private constructor(
        /** The IBAN without spaces, in capitals. */
        private readonly text: string,
        /** The two-letter code of the IBAN's country: "RO". */
        readonly countryCode: string,
    ) {}

    /**
     * The IBAN written `text`, with or without spaces, in any letter case.
     *
     * @param text - The IBAN's text
     * @returns The IBAN
     * @throws {@link PaymentAccountError} with the failure `invalidIban` and the check it failed
     */
    static parse(text: string): Iban {
        return Iban.converted(calling(() => internal.ibanParse(text)));
    }

    /**
     * The IBAN the generated code holds as `iban`.
     *
     * @internal
     */
    static converted(iban: internal.Iban): Iban {
        return new Iban(iban.iban, iban.countryCode);
    }

    /**
     * The IBAN as the generated code holds it.
     *
     * @internal
     */
    generated(): internal.Iban {
        return internal.Iban.create({ iban: this.text, countryCode: this.countryCode });
    }

    /**
     * Whether `other` is the same IBAN, with the same country code.
     *
     * @param other - The IBAN to compare
     * @returns Whether the two are equal
     */
    equals(other: Iban): boolean {
        return this.text === other.text && this.countryCode === other.countryCode;
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
     * @throws {@link PaymentAccountError} with the failure `emptyUsername` for blank text
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
