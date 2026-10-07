// Section 12 of the tour: the group saved as JSON, loaded again, and refused when its file was changed.

import Eranos
import Report

/// A change made to a saved group, and the text it leaves.
struct Tampering {
    let change: String
    let text: String
}

/// What loading the saved form `text` refuses: the error the replay of its entries reports.
func refusal(_ text: String) -> EranosError {
    do {
        _ = try ExpenseGroup.load(text)
        preconditionFailure("the change must be refused")
    } catch {
        return error
    }
}

func savingAndLoading(_ group: ExpenseGroup, _ people: People, _ categories: Categories) throws(EranosError) {
    section("12. Saving and loading")
    let saved = try group.save()
    print("""
        The group saves to \(saved.utf8.count) bytes of JSON: its details and entries, never the totals.
        Ana's accounts are saved in clear text, as the library holds them:
        """)
    guard let ana = try group.member(withID: people.ana) else {
        preconditionFailure("Ana is a member")
    }
    for account in ana.accounts {
        let details = switch account.details {
        case let .iban(iban):
            iban.description
        case let .username(username):
            username.description
        }
        precondition(nil != saved.firstRange(of: details), "\(details) is saved as it is")
        print("  \(account.name): \(details)")
    }
    let loaded = try ExpenseGroup.load(saved)
    print("""
        Loading replays every entry through the same checks as adding it and rebuilds the
        totals: the loaded group equals the saved one. Reading a SavedExpenseGroup and
        loading it with into_expense_group reports a refusal as a typed error, not as text:
        """)
    let tampered = [
        Tampering(change: "\"format\": 3", text: saved.replacing("\"format\": 2", with: "\"format\": 3", maxReplacements: 1)),
        Tampering(change: "a blank group name", text: saved.replacing("\"name\": \"Lisbon and Porto\"", with: "\"name\": \" \"", maxReplacements: 1)),
        Tampering(change: "Lodging renamed FOOD", text: saved.replacing("\"name\": \"Lodging\"", with: "\"name\": \"FOOD\"", maxReplacements: 1))
    ]
    var rows: [LoadRow] = []
    var failures: [ExpenseGroupFailure] = []
    for tampering in tampered {
        precondition(tampering.text != saved, "\(tampering.change) changes the file")
        let error = refusal(tampering.text)
        rows.append(LoadRow(change: tampering.change, message: error.detail))
        guard case let .expenseGroup(failure, _) = error else {
            preconditionFailure("the replay refuses \(tampering.change) with the group's failure")
        }
        failures.append(failure)
    }
    printTable(rows, numbersFrom: 2)

    expect(saved, try loaded.save(), "the loaded group saves as the saved one")
    expect(try group.summary(), try loaded.summary(), "the loaded group's totals")
    precondition([.unsupportedFormat(format: 3), .emptyName, .duplicateCategoryName(categories.food)] == failures, "the replay refuses every change")
}
