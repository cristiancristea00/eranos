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
    #expect(shares.members.map(\.share) == [flat.euros(1_500), flat.euros(1_500)])
    #expect(shares.total == flat.euros(3_000))
    #expect(shares.tip == flat.euros(0))
    #expect(try group.summary().totalSpent == flat.euros(3_000))
    let members = try group.members()
    #expect(members.map(\.firstName) == ["Ana", "Bob"])
    #expect(try group.member(withID: flat.ana) == members.first)
    #expect(try !group.isArchived(flat.ana))
    #expect(try "Flat" == group.name())
    #expect(try "Rent" == group.description())
    #expect(try "pt-PT" == group.language()?.description)
    for strategy in SettlementStrategy.allCases {
        let settlement = try group.settlement(by: strategy)
        // Two balances are well within the adaptive strategy's limit, so it settles them exactly.
        let computedBy: SettlementStrategy = .adaptive == strategy ? .minimumTransfers : strategy
        #expect(settlement.strategy == computedBy)
        #expect(settlement.payments.map(\.amount) == [flat.euros(500)])
    }
    let transfers = try group.transfers()
    #expect(1 == transfers.count)
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
    #expect(try await "Flat" == group.name())
    #expect(try await "Rent" == group.description())
    #expect(try await "pt-PT" == group.language()?.description)
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
    let id: Group.ID
    let categories: [Category]
    let expenses: [Expense]
    let json: String
    let members: [Member]
    let settlement: Settlement
    let summary: Summary
    let transfers: [Transfer]
    let shares: ExpenseShares?

    init(of flat: Flat) throws {
        id = try flat.group.id()
        categories = try flat.group.categories()
        expenses = try flat.group.expenses()
        json = try flat.group.save()
        members = try flat.group.members()
        settlement = try flat.group.settlement(by: .greedy)
        summary = try flat.group.summary()
        transfers = try flat.group.transfers()
        shares = try flat.group.shares(of: flat.dinner)
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
    #expect(try "Travel" == group.replace(trips).name)
    #expect(try "Trips" == group.remove(travel).name)

    let carol = try group.add(Member(firstName: "Carol"))
    var popa = try #require(try group.member(withID: carol))
    popa.lastName = "Popa"
    #expect(try nil == group.replace(popa).lastName)
    try group.archive(carol)
    #expect(try group.isArchived(carol))
    try group.unarchive(carol)
    #expect(try !group.isArchived(carol))
    #expect(try "Popa" == group.remove(carol).lastName)
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
    #expect(try "Lunch" == group.replace(lateLunch).name)
    #expect(try "Late lunch" == group.remove(lunch).name)

    let payment = try group.add(Transfer(from: flat.ana, to: flat.bob, cost: .local(flat.euros(500)), date: flat.day))
    var larger = try #require(try group.transfer(withID: payment))
    larger.cost = .local(flat.euros(700))
    #expect(try group.replace(larger).cost == .local(flat.euros(500)))
    #expect(try group.remove(payment).cost == .local(flat.euros(700)))

    try group.rename(to: "Home")
    #expect(try "Home" == group.name())
    try group.changeDescription(to: nil)
    #expect(try nil == group.description())
    try group.changeLanguage(to: Language("ro"))
    #expect(try group.language() == Language("ro"))
    try group.changeLanguage(to: nil)
    #expect(try nil == group.language())
    let lei = try Currency(code: "RON")
    try group.changeCurrency(to: lei, rates: [flat.euro: ExchangeRate(parsing: "5")])
    #expect(try group.summary().totalSpent == Money(minorUnits: 15_000, currency: lei))
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
    #expect(try await "Travel" == group.replace(trips).name)
    #expect(try await "Trips" == group.remove(travel).name)

    let carol = try await group.add(Member(firstName: "Carol"))
    var popa = try #require(try await group.member(withID: carol))
    popa.lastName = "Popa"
    #expect(try await nil == group.replace(popa).lastName)
    try await group.archive(carol)
    #expect(try await group.isArchived(carol))
    try await group.unarchive(carol)
    #expect(try await "Popa" == group.remove(carol).lastName)

    let lunch = try await group.add(flat.expense("Lunch", paidBy: flat.bob, sharedBy: [flat.ana, flat.bob]))
    var lateLunch = try #require(try await group.expense(withID: lunch))
    lateLunch.name = "Late lunch"
    #expect(try await "Lunch" == group.replace(lateLunch).name)
    #expect(try await "Late lunch" == group.remove(lunch).name)

    let payment = try await group.add(Transfer(from: flat.ana, to: flat.bob, cost: .local(flat.euros(500)), date: flat.day))
    var larger = try #require(try await group.transfer(withID: payment))
    larger.cost = .local(flat.euros(700))
    #expect(try await group.replace(larger).cost == .local(flat.euros(500)))
    #expect(try await group.remove(payment).cost == .local(flat.euros(700)))

    try await group.rename(to: "Home")
    try await group.changeDescription(to: "Bills")
    try await group.changeLanguage(to: Language("ro"))
    #expect(try await "Home" == group.name())
    #expect(try await "Bills" == group.description())
    #expect(try await "ro" == group.language()?.description)
    try await group.changeDescription(to: nil)
    #expect(try await nil == group.description())
    let lei = try Currency(code: "RON")
    try await group.changeCurrency(to: lei, rates: [flat.euro: ExchangeRate(parsing: "5")])
    #expect(try await group.currency() == lei)
}

/// A new group that dropped what it starts with, or a saved one that loaded from text that is
/// not the saved form, would start an app from another group than the user made.
@Test
func aGroupStartsWithWhatItIsGivenAndLoadsOnlyItsSavedForm() async throws {
    let euro = try Currency(code: "EUR")
    let plain = try Group(name: "Trip", currency: euro)
    #expect(try await nil == plain.description())
    #expect(try await nil == plain.language())
    #expect(throws: EranosError.group(.emptyName, detail: "the group name must not be empty")) {
        try Group(name: " ", currency: euro)
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
    #expect(try "Flat" == reader.name())
    #expect(try "Rent" == reader.description())
    #expect(try "pt-PT" == reader.language()?.description)
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
    #expect(try "Home" == group.withRead { reader throws(EranosError) in try reader.name() })
}

/// An asynchronous read handle that answered another way than the synchronous one, or held the
/// lock after its closure returned, would stop every later change.
@Test
func anAsynchronousReadHandleReleasesTheLockWhenItsClosureReturns() async throws {
    let flat = try Flat()
    let group = flat.group
    let reader = try await group.read()
    #expect(try "Flat" == reader.name())
    reader.release()
    let name = try await group.withRead { reader async throws(EranosError) in try reader.name() }
    #expect("Flat" == name)
    try await group.rename(to: "Home")
    #expect(try await "Home" == group.name())
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
    #expect(try await 2 == group.members().count)
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
