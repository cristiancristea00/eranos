/**
 * The identifiers of the entities: each is text of at most 64 bytes of UTF-8, not empty, without control characters, and
 * without whitespace at either end, compared byte for byte, so case matters. An app may give a new entry or a new group
 * an identifier of its own, such as the key of its own store; without one, the library generates a UUID v7 in lowercase
 * hyphenated form. Each is branded so that one entity's identifier cannot stand where another's is expected. An identifier
 * converts to its text and back, for storing it: {@link parseMemberId} and its siblings check the text, and
 * {@link memberId} and its siblings take it unchecked, for the library to check when a call carries it.
 *
 * @packageDocumentation
 */

import { calling } from './converting.js';
import * as internal from './internal/eranos.js';

declare const groupBrand: unique symbol;
declare const memberBrand: unique symbol;
declare const categoryBrand: unique symbol;
declare const expenseBrand: unique symbol;
declare const transferBrand: unique symbol;

/** The identifier of a group: text, which is a UUID v7 in lowercase hyphenated form when the library generated it. */
export type ExpenseGroupId = string & {
    /** What sets the identifier of a group apart from other text and other identifiers: no value has it. */
    readonly [groupBrand]: never;
};

/** The identifier of a member: text, which is a UUID v7 in lowercase hyphenated form when the library generated it. */
export type MemberId = string & {
    /** What sets the identifier of a member apart from other text and other identifiers: no value has it. */
    readonly [memberBrand]: never;
};

/** The identifier of a category: text, which is a UUID v7 in lowercase hyphenated form when the library generated it. */
export type CategoryId = string & {
    /** What sets the identifier of a category apart from other text and other identifiers: no value has it. */
    readonly [categoryBrand]: never;
};

/** The identifier of an expense: text, which is a UUID v7 in lowercase hyphenated form when the library generated it. */
export type ExpenseId = string & {
    /** What sets the identifier of an expense apart from other text and other identifiers: no value has it. */
    readonly [expenseBrand]: never;
};

/** The identifier of a transfer: text, which is a UUID v7 in lowercase hyphenated form when the library generated it. */
export type TransferId = string & {
    /** What sets the identifier of a transfer apart from other text and other identifiers: no value has it. */
    readonly [transferBrand]: never;
};

/**
 * The group identifier written `text`, unchecked: a call that carries it checks it, and {@link parseGroupId} checks it
 * at once.
 *
 * @param text - The identifier's text
 * @returns The identifier
 */
export function groupId(text: string): ExpenseGroupId {
    return text as ExpenseGroupId;
}

/**
 * The group identifier written `text`, as it is: not trimmed, and in its case, checked by the rules every kind of
 * identifier shares.
 *
 * @param text - The identifier's text
 * @returns The identifier
 * @throws {@link IdError} with the failure `empty`, `tooLong` for more than 64 bytes of UTF-8, `controlCharacter`, or
 *   `surroundingWhitespace`: the first rule the text breaks
 */
export function parseGroupId(text: string): ExpenseGroupId {
    return groupId(calling(() => internal.identifierParse(text)));
}

/**
 * The member identifier written `text`, unchecked: a call that carries it checks it, and {@link parseMemberId} checks
 * it at once.
 *
 * @param text - The identifier's text
 * @returns The identifier
 */
export function memberId(text: string): MemberId {
    return text as MemberId;
}

/**
 * The member identifier written `text`, as it is: not trimmed, and in its case, checked by the rules every kind of
 * identifier shares.
 *
 * @param text - The identifier's text
 * @returns The identifier
 * @throws {@link IdError} with the failure `empty`, `tooLong` for more than 64 bytes of UTF-8, `controlCharacter`, or
 *   `surroundingWhitespace`: the first rule the text breaks
 */
export function parseMemberId(text: string): MemberId {
    return memberId(calling(() => internal.identifierParse(text)));
}

/**
 * The category identifier written `text`, unchecked: a call that carries it checks it, and {@link parseCategoryId}
 * checks it at once.
 *
 * @param text - The identifier's text
 * @returns The identifier
 */
export function categoryId(text: string): CategoryId {
    return text as CategoryId;
}

/**
 * The category identifier written `text`, as it is: not trimmed, and in its case, checked by the rules every kind of
 * identifier shares.
 *
 * @param text - The identifier's text
 * @returns The identifier
 * @throws {@link IdError} with the failure `empty`, `tooLong` for more than 64 bytes of UTF-8, `controlCharacter`, or
 *   `surroundingWhitespace`: the first rule the text breaks
 */
export function parseCategoryId(text: string): CategoryId {
    return categoryId(calling(() => internal.identifierParse(text)));
}

/**
 * The expense identifier written `text`, unchecked: a call that carries it checks it, and {@link parseExpenseId} checks
 * it at once.
 *
 * @param text - The identifier's text
 * @returns The identifier
 */
export function expenseId(text: string): ExpenseId {
    return text as ExpenseId;
}

/**
 * The expense identifier written `text`, as it is: not trimmed, and in its case, checked by the rules every kind of
 * identifier shares.
 *
 * @param text - The identifier's text
 * @returns The identifier
 * @throws {@link IdError} with the failure `empty`, `tooLong` for more than 64 bytes of UTF-8, `controlCharacter`, or
 *   `surroundingWhitespace`: the first rule the text breaks
 */
export function parseExpenseId(text: string): ExpenseId {
    return expenseId(calling(() => internal.identifierParse(text)));
}

/**
 * The transfer identifier written `text`, unchecked: a call that carries it checks it, and {@link parseTransferId}
 * checks it at once.
 *
 * @param text - The identifier's text
 * @returns The identifier
 */
export function transferId(text: string): TransferId {
    return text as TransferId;
}

/**
 * The transfer identifier written `text`, as it is: not trimmed, and in its case, checked by the rules every kind of
 * identifier shares.
 *
 * @param text - The identifier's text
 * @returns The identifier
 * @throws {@link IdError} with the failure `empty`, `tooLong` for more than 64 bytes of UTF-8, `controlCharacter`, or
 *   `surroundingWhitespace`: the first rule the text breaks
 */
export function parseTransferId(text: string): TransferId {
    return transferId(calling(() => internal.identifierParse(text)));
}
