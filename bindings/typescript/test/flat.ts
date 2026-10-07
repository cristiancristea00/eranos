// A group the tests share the shape of, and the loading of the module every test runs in.

import { readFile } from 'node:fs/promises';

import {
    type CategoryId,
    type Expense,
    type ExpenseId,
    type MemberId,
    type TransferId,
    CalendarDate,
    Currency,
    ExpenseGroup,
    Language,
    Money,
    init,
} from '../src/index.js';

/** The module, loaded once for every test file. */
export const initialised: Promise<void> = init(await readFile(new URL('../src/internal/eranos_web.wasm', import.meta.url)));

await initialised;

/** A group of two with a category, an expense, and a transfer, and the identifiers the group gave them. */
export class Flat {
    readonly euro = Currency.of('EUR');
    readonly day = CalendarDate.of(2026, 9, 23);
    readonly group = ExpenseGroup.create('Flat', this.euro, { description: 'Rent', language: Language.parse('pt-pt') });
    readonly ana: MemberId = this.group.addMemberSync({ firstName: 'Ana' });
    readonly bob: MemberId = this.group.addMemberSync({ firstName: 'Bob' });
    readonly food: CategoryId = this.group.addCategorySync({ name: 'Food' });
    readonly dinner: ExpenseId = this.group.addExpenseSync({
        name: 'Dinner',
        cost: { kind: 'local', amount: this.euros(3_000n) },
        payers: { kind: 'equal', members: [this.ana] },
        split: { kind: 'whole', split: { kind: 'equal', members: [this.ana, this.bob] } },
        date: this.day,
        category: this.food,
    });
    readonly repayment: TransferId = this.group.addTransferSync({
        from: this.bob,
        to: this.ana,
        cost: { kind: 'local', amount: this.euros(1_000n) },
        date: this.day,
    });

    /** `minorUnits` cents. */
    euros(minorUnits: bigint): Money {
        return new Money(minorUnits, this.euro);
    }

    /** An expense of 30.00 EUR, called `name`, that `payer` paid and `members` share equally. */
    expense(name: string, payer: MemberId, members: readonly MemberId[]): Expense {
        return {
            name,
            cost: { kind: 'local', amount: this.euros(3_000n) },
            payers: { kind: 'equal', members: [payer] },
            split: { kind: 'whole', split: { kind: 'equal', members } },
            date: this.day,
        };
    }
}
