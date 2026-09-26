// Saving and loading a group as JSON. The saved form holds the entries only; loading replays them through the group's
// checks and recomputes every total. The port of the Rust example `persistence`, which prints the same report.

#include "eranos.h"
#include "report.h"

#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

/// The balance of @p member in @p group, as the library writes it, or ? for a member without totals.
static Line balanceOf(EranosGroup const *const group, Id const member) {
    EranosSummary *summary = NULL;
    must(eranosGroupSummary(group, &summary));
    EranosMemberSummary *const totals = totalsOf(summary, member.text);
    Line const line = NULL == totals ? lineOf("%s", "?") : balanceLine(eranosMemberSummaryBalance(totals));
    eranosMemberSummaryDestroy(totals);
    eranosSummaryDestroy(summary);
    return line;
}

/// The lunch that Bob paid, with a tip of 10 % shared in proportion to what each consumed.
static EranosExpense *lunchOf(Members const *const both, Id const bob, EranosCalendarDate const day) {
    EranosExpense *const expense = expenseOf(localCost(amountOf("30.00", "EUR")), day, "Lunch", soleSplit(bob), equalSplit(both));
    int32_t basisPoints = 0;
    must(eranosPercentageParse("10", &basisPoints));
    EranosTipValue *const value = eranosTipValueCreatePercentage(basisPoints);
    EranosTip *const tip = eranosTipCreate(value, ERANOS_TIP_DISTRIBUTION_PROPORTIONAL);
    eranosExpenseSetTip(expense, tip);
    eranosTipDestroy(tip);
    eranosTipValueDestroy(value);
    return expense;
}

int main(void) {
    EranosCalendarDate const day = {.day = 23, .month = 9, .year = 2026};
    EranosNewGroup *details = NULL;
    must(eranosNewGroupCreate("EUR", "Road trip", &details));
    EranosGroup *group = NULL;
    must(eranosGroupCreate(details, &group));
    eranosNewGroupDestroy(details);
    Id const ana = addMember(group, "Ana");
    Id const bob = addMember(group, "Bob");
    Members const both = {.count = 2, .ids = {ana, bob}};
    addExpense(group, expenseOf(localCost(amountOf("70.00", "EUR")), day, "Fuel", soleSplit(ana), equalSplit(&both)));
    addExpense(group, lunchOf(&both, bob, day));

    title("Road trip: saving and loading a group");
    printLine("With the `serde` feature a group serialises, and so does every type it stores.\n"
              "It saves only what was entered: its details, members, categories, expenses,\n"
              "and transfers. The totals, balances, and settlements are recomputed when it is\n"
              "loaded.");

    section("1. The saved form");
    // Account details would be in clear text: store this in the platform's secure storage (Keychain, Keystore, or an
    // encrypted database).
    EranosText *const saved = eranosTextCreate();
    must(eranosGroupSave(group, saved));
    printLine(eranosTextData(saved));
    printLine("Amounts are minor units with their currency, dates are YYYY-MM-DD, and the\n"
              "`format` number lets a future release refuse a file it cannot read instead of\n"
              "dropping fields.");

    section("2. Loading it back");
    EranosGroup *loaded = NULL;
    must(eranosGroupLoad(eranosTextData(saved), &loaded));
    printLine("Loading replays every entry through the same checks as adding it, then rebuilds\n"
              "the totals. The loaded group equals the saved one, and so do its balances:");
    Table *const table = tableCreate(3, (char const *const[]){"Member", "Balance before saving", "Balance after loading"}, 1);
    Id const members[] = {ana, bob};
    for (size_t index = 0; 2 > index; ++index) {
        Line const before = balanceOf(group, members[index]);
        Line const after = balanceOf(loaded, members[index]);
        check(0 == strcmp(before.text, after.text), "the loaded group has the saved one's balances");
        tableAdd(table, (Line const[]){firstName(group, members[index].text), before, after});
    }
    tablePrint(table);
    EranosSettlement *plan = NULL;
    must(eranosGroupSettlement(loaded, ERANOS_SETTLEMENT_STRATEGY_GREEDY, &plan));
    EranosPaymentList *const payments = eranosSettlementPayments(plan);
    printFormatted("Payments needed to settle the loaded group: %zu.", eranosPaymentListLength(payments));

    section("3. A corrupted file is refused");
    printLine("Here the fuel's payer was edited to an identifier no member has. Reading the file\n"
              "as a SavedGroup and loading it with into_group replays every entry through the\n"
              "same checks as adding it, and refuses the file instead of producing balances that\n"
              "do not add up:");
    // The fuel is the first entry that names Ana: the expenses come before the members in the saved form.
    char *const corrupted = replaceFirst(eranosTextData(saved), ana.text, "0199a000-0000-7000-8000-00000000ffff");
    EranosGroup *refusedGroup = NULL;
    EranosError *const refused = eranosGroupLoad(corrupted, &refusedGroup);
    check(NULL != refused && NULL == refusedGroup, "the file is refused");
    Line const message = messageLine(refused);
    printFormatted("  error: %s", message.text);

    EranosText *const resaved = eranosTextCreate();
    must(eranosGroupSave(loaded, resaved));
    check(0 == strcmp(eranosTextData(resaved), eranosTextData(saved)), "the loaded group saves as the group did");
    check(1 == eranosPaymentListLength(payments), "one payment settles the loaded group");
    eranosTextDestroy(resaved);
    free(corrupted);
    eranosPaymentListDestroy(payments);
    eranosSettlementDestroy(plan);
    eranosGroupDestroy(loaded);
    eranosTextDestroy(saved);
    eranosGroupDestroy(group);
    return EXIT_SUCCESS;
}
