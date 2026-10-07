// Every feature of the library in one story: a group and its details, members with contact details and accounts,
// categories, an expense for every way of dividing one, another currency, edits and refusals, members who come and go,
// the summary, a new group currency, settling up, saving and loading, the values on their own, and the group shared
// between tasks. Every section checks its key numbers. The port of the Rust example `grand_tour`, which prints the same
// report.

import { anotherCurrency, comingsAndGoings, edits } from './grand_tour/edits.js';
import { expenses } from './grand_tour/expenses.js';
import { addCategories, members, theGroup } from './grand_tour/members.js';
import { savingAndLoading } from './grand_tour/saving.js';
import { sharing } from './grand_tour/sharing.js';
import { newCurrency, removingACategory, settlingUp, summary } from './grand_tour/totals.js';
import { values } from './grand_tour/values.js';
import { load, printLines } from './report.js';

await load();
printLines('Lisbon and Porto: a grand tour of eranos', '════════════════════════════════════════');
printLines(
    "Ana, Bob, Carol, Dan, and Carol's daughter Eva spend a week in Portugal. Each",
    'section below uses another part of the library, from creating the group to saving',
    'it, and checks its key numbers.',
);
using group = theGroup();
const people = members(group);
const categories = addCategories(group);
const seafood = expenses(group, people, categories);
anotherCurrency(group, people, categories);
edits(group, people, categories, seafood);
comingsAndGoings(group, people);
summary(group, people, categories);
removingACategory(group, categories);
newCurrency(group, people);
settlingUp(group);
savingAndLoading(group, people, categories);
values(people);
await sharing(group, people);
