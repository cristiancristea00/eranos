// Section 12 of the tour: the group saved as JSON, loaded again, and refused when its file was changed.

import Eranos
import Report

/// What loading the saved form `text` refuses: the error the replay of its entries reports.
func refusal(_ text: String) -> EranosError {
    do {
        _ = try Group.load(text)
        preconditionFailure("the change must be refused")
    } catch {
        return error
    }
}

func savingAndLoading(_ group: Group, _ people: People, _ categories: Categories) throws(EranosError) {
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
    let loaded = try Group.load(saved)
    print("""
        Loading replays every entry through the same checks as adding it and rebuilds the
        totals: the loaded group equals the saved one. Reading a SavedGroup and loading it
        with into_group reports a refusal as a typed error instead of a message:
        """)
    let tampered = [
        ("\"format\": 3", saved.replacing("\"format\": 2", with: "\"format\": 3", maxReplacements: 1)),
        ("a blank group name", saved.replacing("\"name\": \"Lisbon and Porto\"", with: "\"name\": \" \"", maxReplacements: 1)),
        ("Lodging renamed FOOD", saved.replacing("\"name\": \"Lodging\"", with: "\"name\": \"FOOD\"", maxReplacements: 1))
    ]
    var rows: [[String]] = []
    var errors: [EranosError] = []
    for (change, text) in tampered {
        precondition(text != saved, "\(change) changes the file")
        let error = refusal(text)
        rows.append([change, error.detail])
        errors.append(error)
    }
    printTable(["Change to the file", "Message"], rows, numbersFrom: 2)

    expect(saved, try loaded.save(), "the loaded group saves as the saved one")
    expect(try group.summary(), try loaded.summary(), "the loaded group's totals")
    guard 3 == errors.count,
          case .group(.unsupportedFormat(format: 3), _) = errors[0],
          case .group(.emptyName, _) = errors[1],
          case .group(.duplicateCategoryName(categories.food), _) = errors[2] else {
        preconditionFailure("the replay refuses every change")
    }
}
