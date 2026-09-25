/**
 * The identifiers of the entities: each a hyphenated UUID, branded so that one entity's identifier cannot stand where
 * another's is expected. An identifier converts to its text and back, for storing it; the library checks the text when a
 * call carries it.
 *
 * @packageDocumentation
 */

declare const groupBrand: unique symbol;
declare const memberBrand: unique symbol;
declare const categoryBrand: unique symbol;
declare const expenseBrand: unique symbol;
declare const transferBrand: unique symbol;

/** The identifier of a group: a hyphenated UUID the library gave it. */
export type GroupId = string & {
    /** What sets the identifier of a group apart from other text and other identifiers: no value has it. */
    readonly [groupBrand]: never;
};

/** The identifier of a member: a hyphenated UUID the group gave them. */
export type MemberId = string & {
    /** What sets the identifier of a member apart from other text and other identifiers: no value has it. */
    readonly [memberBrand]: never;
};

/** The identifier of a category: a hyphenated UUID the group gave it. */
export type CategoryId = string & {
    /** What sets the identifier of a category apart from other text and other identifiers: no value has it. */
    readonly [categoryBrand]: never;
};

/** The identifier of an expense: a hyphenated UUID the group gave it. */
export type ExpenseId = string & {
    /** What sets the identifier of an expense apart from other text and other identifiers: no value has it. */
    readonly [expenseBrand]: never;
};

/** The identifier of a transfer: a hyphenated UUID the group gave it. */
export type TransferId = string & {
    /** What sets the identifier of a transfer apart from other text and other identifiers: no value has it. */
    readonly [transferBrand]: never;
};

/**
 * The group identifier written `text`.
 *
 * @param text - The identifier's text
 * @returns The identifier
 */
export function groupId(text: string): GroupId {
    return text as GroupId;
}

/**
 * The member identifier written `text`.
 *
 * @param text - The identifier's text
 * @returns The identifier
 */
export function memberId(text: string): MemberId {
    return text as MemberId;
}

/**
 * The category identifier written `text`.
 *
 * @param text - The identifier's text
 * @returns The identifier
 */
export function categoryId(text: string): CategoryId {
    return text as CategoryId;
}

/**
 * The expense identifier written `text`.
 *
 * @param text - The identifier's text
 * @returns The identifier
 */
export function expenseId(text: string): ExpenseId {
    return text as ExpenseId;
}

/**
 * The transfer identifier written `text`.
 *
 * @param text - The identifier's text
 * @returns The identifier
 */
export function transferId(text: string): TransferId {
    return text as TransferId;
}
