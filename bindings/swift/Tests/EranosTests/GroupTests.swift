// The group, in both forms of every call. A call that answered another way than the group holds,
// a change that did not take effect, or an asynchronous form that differed from its synchronous
// twin would show an app one group from a background task and another from its main thread.

@testable import Eranos
import Testing

/// A read that answered another way than the group holds would show an app entries, totals,
/// or plans the group does not have.
@Test
func everySynchronousReadAnswersWhatTheGroupHolds() throws {
    let flat = try Flat()
    let group = flat.group
    #expect(try !group.id().rawValue.isEmpty)
    let categories = try group.categories()
    #expect(categories.map(\.name) == ["Uncategorised", "Food"])
    #expect(try group.category(withID: flat.food) == categories[1])
    #expect(try group.currency() == flat.euro)
    let expenses = try group.expenses()
    #expect(expenses.map(\.name) == ["Dinner"])
    #expect(try group.expense(withID: flat.dinner) == expenses.first)
    let shares = try #require(try group.shares(of: flat.dinner))
    #expect(shares.total == flat.euros(3_000))
    #expect(shares.tip == flat.euros(0))
    #expect(shares.members.map(\.share) == [flat.euros(1_500), flat.euros(1_500)])
    #expect(try group.summary().totalSpent == flat.euros(3_000))
    let members = try group.members()
    #expect(members.map(\.firstName) == ["Ana", "Bob"])
    #expect(try group.member(withID: flat.ana) == members.first)
    #expect(try !group.isArchived(flat.ana))
    #expect(try group.name() == "Flat")
    #expect(try group.description() == "Rent")
    #expect(try group.language()?.description == "pt-PT")
    for strategy in SettlementStrategy.allCases {
        let settlement = try group.settlement(by: strategy)
        #expect(settlement.strategy == strategy)
        #expect(settlement.payments.map(\.amount) == [flat.euros(500)])
    }
    let transfers = try group.transfers()
    #expect(transfers.count == 1)
    #expect(try group.transfer(withID: flat.repayment) == transfers.first)
    #expect(try group.amount(of: flat.repayment) == flat.euros(1_000))
    let json = try group.save()
    #expect(try Group.load(json).save() == json)
    #expect(throws: EranosError.id(.invalidID, detail: "invalid identifier: expected a UUID")) {
        try group.member(withID: Member.ID(rawValue: "nobody"))
    }
}

/// An asynchronous read that answered another way than its synchronous twin would show an app
/// one group from a background task and another from its main thread.
@Test
func everyAsynchronousReadAnswersAsItsSynchronousTwin() async throws {
    let flat = try Flat()
    let group = flat.group
    let synchronous = try Snapshot(of: flat)
    #expect(try await group.id() == synchronous.id)
    #expect(try await group.categories() == synchronous.categories)
    #expect(try await group.category(withID: flat.food) == synchronous.categories.last)
    #expect(try await group.currency() == flat.euro)
    #expect(try await group.expenses() == synchronous.expenses)
    #expect(try await group.expense(withID: flat.dinner) == synchronous.expenses.first)
    #expect(try await group.shares(of: flat.dinner) == synchronous.shares)
    #expect(try await group.summary() == synchronous.summary)
    #expect(try await group.members() == synchronous.members)
    #expect(try await group.member(withID: flat.ana) == synchronous.members.first)
    #expect(try await !group.isArchived(flat.ana))
    #expect(try await group.name() == "Flat")
    #expect(try await group.description() == "Rent")
    #expect(try await group.language()?.description == "pt-PT")
    #expect(try await group.settlement(by: .greedy) == synchronous.settlement)
    #expect(try await group.transfers() == synchronous.transfers)
    #expect(try await group.transfer(withID: flat.repayment) == synchronous.transfers.first)
    #expect(try await group.amount(of: flat.repayment) == flat.euros(1_000))
    let json = try await group.save()
    #expect(json == synchronous.json)
    #expect(try await Group.load(json).save() == json)
}

/// What the synchronous reads answer, to compare the asynchronous ones with.
private struct Snapshot {
    let categories: [Category]
    let expenses: [Expense]
    let id: Group.ID
    let json: String
    let members: [Member]
    let settlement: Settlement
    let shares: ExpenseShares?
    let summary: Summary
    let transfers: [Transfer]

    init(of flat: Flat) throws {
        categories = try flat.group.categories()
        expenses = try flat.group.expenses()
        id = try flat.group.id()
        json = try flat.group.save()
        members = try flat.group.members()
        settlement = try flat.group.settlement(by: .greedy)
        shares = try flat.group.shares(of: flat.dinner)
        summary = try flat.group.summary()
        transfers = try flat.group.transfers()
    }
}

/// A change to categories or members that did not take effect, or returned another entry than
/// the one it replaced or removed, would leave an app showing a group that is not there.
@Test
func synchronousChangesToCategoriesAndMembersTakeEffect() throws {
    let flat = try Flat()
    let group = flat.group
    let travel = try group.add(Category(name: "Travel"))
    var trips = try #require(try group.category(withID: travel))
    trips.name = "Trips"
    #expect(try group.replace(trips).name == "Travel")
    #expect(try group.remove(travel).name == "Trips")

    let carol = try group.add(Member(firstName: "Carol"))
    var popa = try #require(try group.member(withID: carol))
    popa.lastName = "Popa"
    #expect(try group.replace(popa).lastName == nil)
    try group.archive(carol)
    #expect(try group.isArchived(carol))
    try group.unarchive(carol)
    #expect(try !group.isArchived(carol))
    #expect(try group.remove(carol).lastName == "Popa")
    #expect(throws: EranosError.self) {
        try group.remove(flat.ana)
    }
}

/// A change to expenses, transfers, or the group's own details that did not take effect would
/// leave an app showing a group that is not there.
@Test
func synchronousChangesToEntriesAndDetailsTakeEffect() throws {
    let flat = try Flat()
    let group = flat.group
    let lunch = try group.add(flat.expense("Lunch", paidBy: flat.bob, sharedBy: [flat.ana, flat.bob]))
    var lateLunch = try #require(try group.expense(withID: lunch))
    lateLunch.name = "Late lunch"
    #expect(try group.replace(lateLunch).name == "Lunch")
    #expect(try group.remove(lunch).name == "Late lunch")

    let payment = try group.add(Transfer(cost: .local(flat.euros(500)), date: flat.day, from: flat.ana, to: flat.bob))
    var larger = try #require(try group.transfer(withID: payment))
    larger.cost = .local(flat.euros(700))
    #expect(try group.replace(larger).cost == .local(flat.euros(500)))
    #expect(try group.remove(payment).cost == .local(flat.euros(700)))

    try group.rename(to: "Home")
    #expect(try group.name() == "Home")
    try group.changeDescription(to: nil)
    #expect(try group.description() == nil)
    try group.changeLanguage(to: Language("ro"))
    #expect(try group.language() == Language("ro"))
    try group.changeLanguage(to: nil)
    #expect(try group.language() == nil)
    let lei = try Currency(code: "RON")
    try group.changeCurrency(to: lei, rates: [flat.euro: ExchangeRate(parsing: "5")])
    #expect(try group.summary().totalSpent == Money(currency: lei, minorUnits: 15_000))
}

/// An asynchronous change that did not take effect, or took another effect than its synchronous
/// twin, would leave the group in another state than the app asked for.
@Test
func asynchronousChangesTakeEffectAsTheirSynchronousTwinsDo() async throws {
    let flat = try Flat()
    let group = flat.group
    let travel = try await group.add(Category(name: "Travel"))
    var trips = try #require(try await group.category(withID: travel))
    trips.name = "Trips"
    #expect(try await group.replace(trips).name == "Travel")
    #expect(try await group.remove(travel).name == "Trips")

    let carol = try await group.add(Member(firstName: "Carol"))
    var popa = try #require(try await group.member(withID: carol))
    popa.lastName = "Popa"
    #expect(try await group.replace(popa).lastName == nil)
    try await group.archive(carol)
    #expect(try await group.isArchived(carol))
    try await group.unarchive(carol)
    #expect(try await group.remove(carol).lastName == "Popa")

    let lunch = try await group.add(flat.expense("Lunch", paidBy: flat.bob, sharedBy: [flat.ana, flat.bob]))
    var lateLunch = try #require(try await group.expense(withID: lunch))
    lateLunch.name = "Late lunch"
    #expect(try await group.replace(lateLunch).name == "Lunch")
    #expect(try await group.remove(lunch).name == "Late lunch")

    let payment = try await group.add(Transfer(cost: .local(flat.euros(500)), date: flat.day, from: flat.ana, to: flat.bob))
    var larger = try #require(try await group.transfer(withID: payment))
    larger.cost = .local(flat.euros(700))
    #expect(try await group.replace(larger).cost == .local(flat.euros(500)))
    #expect(try await group.remove(payment).cost == .local(flat.euros(700)))

    try await group.rename(to: "Home")
    try await group.changeDescription(to: "Bills")
    try await group.changeLanguage(to: Language("ro"))
    #expect(try await [group.name(), group.description(), group.language()?.description] == ["Home", "Bills", "ro"])
    try await group.changeDescription(to: nil)
    #expect(try await group.description() == nil)
    let lei = try Currency(code: "RON")
    try await group.changeCurrency(to: lei, rates: [flat.euro: ExchangeRate(parsing: "5")])
    #expect(try await group.currency() == lei)
}

/// A new group that dropped what it starts with, or a saved one that loaded from text that is
/// not the saved form, would start an app from another group than the user made.
@Test
func aGroupStartsWithWhatItIsGivenAndLoadsOnlyItsSavedForm() async throws {
    let euro = try Currency(code: "EUR")
    let plain = try Group(currency: euro, name: "Trip")
    #expect(try await plain.description() == nil)
    #expect(try await plain.language() == nil)
    #expect(throws: EranosError.group(.emptyName, detail: "the group name must not be empty")) {
        try Group(currency: euro, name: " ")
    }
    #expect(throws: EranosError.invalidJSON(line: 1, column: 1, detail: "the saved group ends early, at line 1, column 1")) {
        try Group.load("{")
    }
    await #expect(throws: EranosError.self) {
        try await Group.load("{")
    }
}

/// A read handle that answered another way than the group, kept reading after its release, or
/// kept the lock would show a report of a group that may have changed since.
@Test
func aReadHandleReadsAsTheGroupDoesUntilItIsReleased() throws {
    let flat = try Flat()
    let group = flat.group
    let snapshot = try Snapshot(of: flat)
    let reader = try group.read()
    #expect(try reader.id() == snapshot.id)
    #expect(try reader.categories() == snapshot.categories)
    #expect(try reader.category(withID: flat.food) == snapshot.categories.last)
    #expect(try reader.currency() == flat.euro)
    #expect(try reader.expenses() == snapshot.expenses)
    #expect(try reader.expense(withID: flat.dinner) == snapshot.expenses.first)
    #expect(try reader.shares(of: flat.dinner) == snapshot.shares)
    #expect(try reader.summary() == snapshot.summary)
    #expect(try reader.members() == snapshot.members)
    #expect(try reader.member(withID: flat.ana) == snapshot.members.first)
    #expect(try !reader.isArchived(flat.ana))
    #expect(try reader.name() == "Flat")
    #expect(try reader.description() == "Rent")
    #expect(try reader.language()?.description == "pt-PT")
    #expect(try reader.settlement(by: .greedy) == snapshot.settlement)
    #expect(try reader.transfers() == snapshot.transfers)
    #expect(try reader.transfer(withID: flat.repayment) == snapshot.transfers.first)
    #expect(try reader.amount(of: flat.repayment) == flat.euros(1_000))
    #expect(try reader.save() == snapshot.json)
    reader.release()
    reader.release()
    #expect(throws: EranosError.released(detail: "the read handle was released")) {
        try reader.name()
    }
    try group.rename(to: "Home")
    #expect(try group.withRead { reader throws(EranosError) in try reader.name() } == "Home")
}

/// An asynchronous read handle that answered another way than the synchronous one, or held the
/// lock after its closure returned, would stop every later change.
@Test
func anAsynchronousReadHandleReleasesTheLockWhenItsClosureReturns() async throws {
    let flat = try Flat()
    let group = flat.group
    let reader = try await group.read()
    #expect(try reader.name() == "Flat")
    reader.release()
    let name = try await group.withRead { reader async throws(EranosError) in try reader.name() }
    #expect(name == "Flat")
    try await group.rename(to: "Home")
    #expect(try await group.name() == "Home")
}

/// A cancelled task whose call still ran would change the group after the app gave the change
/// up; one whose call was never cancelled would keep waiting for a lock it no longer needs.
@Test
func cancellingATaskCancelsItsCallAndLeavesTheGroupAsItWas() async throws {
    let flat = try Flat()
    let group = flat.group
    let (gate, opener) = AsyncStream<Void>.makeStream()
    let task = Task {
        // Waits until the task is cancelled, so that the call starts cancelled.
        for await _ in gate {}
        return try await group.add(Member(firstName: "Carol"))
    }
    task.cancel()
    opener.finish()
    let result = await task.result
    #expect(throws: EranosError.cancelled(detail: "the call was cancelled")) {
        try result.get()
    }
    #expect(try await group.members().count == 2)
}

/// A failure the library does not describe, such as a defect in the bindings, that escaped the
/// typed error would crash an app that catches only it.
@Test
func aFailureTheLibraryDoesNotDescribeIsUnexpected() async {
    #expect(throws: EranosError.unexpected(detail: "Unexpected()")) {
        try converting { () throws -> Int in throw Unexpected() }
    }
    await #expect(throws: EranosError.unexpected(detail: "Unexpected()")) {
        try await converting { _ async throws -> Int in throw Unexpected() }
    }
}

/// A failure that is not the library's.
private struct Unexpected: Error {}
