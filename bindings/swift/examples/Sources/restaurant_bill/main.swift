// An itemised restaurant bill: every line is split among the people who had it, quantities multiply the unit price, and
// the tip is shared either in proportion to what each person ordered or equally. The port of the Rust example
// `restaurant_bill`, which prints the same report.

import Eranos
import Report

let euro = try Currency(code: "EUR")

/// The amount written `text`, in euros.
func eur(_ text: String) throws(EranosError) -> Money {
    try Money(parsing: text, in: euro)
}

/// The receipt: every line with the members who had it.
func receipt(_ ana: Member.ID, _ bob: Member.ID, _ carol: Member.ID) throws(EranosError) -> [Item] {
    [
        Item(name: "Ciorbă", split: .equal([ana]), unitPrice: try eur("8.00")),
        Item(name: "Sarmale", split: .equal([bob, carol]), unitPrice: try eur("14.00"), quantity: 2),
        Item(
            name: "Papanași",
            split: .parts([MemberParts(member: ana, parts: try Parts(hundredths: 100)), MemberParts(member: carol, parts: try Parts(hundredths: 200))]),
            unitPrice: try eur("9.00")
        ),
        Item(name: "Lemonade", split: .equal([ana, bob, carol]), unitPrice: try eur("3.00"), quantity: 3)
    ]
}

/// How `expense` would divide if it were added to `group`, computed on a copy loaded from the group's saved form, so
/// the group itself is unchanged.
func preview(_ group: Group, _ expense: Expense) throws(EranosError) -> ExpenseShares? {
    let copy = try Group.load(try group.save())
    return try copy.shares(of: try copy.add(expense))
}

/// What `member` consumed of the expense that `shares` divides.
func consumed(_ shares: ExpenseShares?, _ member: Member.ID) -> Money? {
    shares?.members.first { share in share.member == member }?.share
}

func main() throws(EranosError) {
    let group = try Group(currency: euro, name: "Dinner at Caru' cu Bere")
    let ana = try group.add(Member(firstName: "Ana"))
    let bob = try group.add(Member(firstName: "Bob"))
    let carol = try group.add(Member(firstName: "Carol"))
    let day = try CalendarDate(day: 23, month: 9, year: 2026)

    print("\(try group.name()): an itemised bill")
    print(String(repeating: "═", count: try group.name().count + 19))
    print("""
        Three friends share a Romanian dinner. Instead of splitting the whole bill evenly,
        every line of the receipt is split among the people who had it.
        """)

    section("1. The receipt")
    printTable(
        ["Item", "Shared by", "Quantity", "Unit price", "Line total"],
        try receipt(ana, bob, carol).map { item throws(EranosError) in
            let total = Money(currency: item.unitPrice.currency, minorUnits: item.unitPrice.minorUnits * Int64(item.quantity))
            return [item.name, try names(group, item.split.members), "\(item.quantity)", "\(item.unitPrice)", "\(total)"]
        },
        numbersFrom: 2
    )
    print("""
        The papanași are split by parts: Carol ate two of the three. The line totals add up
        to 54.00 EUR, and the expense's amount must match them exactly.
        """)

    section("2. Who ordered what, and the tip")
    // The dinner, its tip of 10 % shared by `distribution`.
    let bill = { (distribution: TipDistribution) throws(EranosError) -> Expense in
        Expense(
            cost: .local(try eur("54.00")),
            date: day,
            name: "Dinner",
            payers: .equal([ana]),
            split: .itemised(try receipt(ana, bob, carol)),
            tip: Tip(value: .percentage(try Percentage(parsing: "10")), distribution: distribution)
        )
    }
    let untipped = Expense(cost: .local(try eur("54.00")), date: day, name: "Dinner", payers: .equal([ana]), split: .itemised(try receipt(ana, bob, carol)))
    let ordered = try preview(group, untipped)
    let proportional = try group.add(try bill(.proportional))
    let byConsumption = try group.shares(of: proportional)
    let even = try preview(group, try bill(.equal))
    print("""
        Ana paid the bill plus a 10 % tip (\(byConsumption?.tip.description ?? "?")). A tip can be shared in proportion to what
        each person ordered, or equally; both are shown:
        """)
    var rows: [[String]] = []
    for (member, firstName) in [(ana, "Ana"), (bob, "Bob"), (carol, "Carol")] {
        guard let subtotal = consumed(ordered, member),
              let proportionalTotal = consumed(byConsumption, member),
              let equalTotal = consumed(even, member) else {
            preconditionFailure("every member ordered")
        }
        rows.append([
            firstName,
            "\(subtotal)",
            "\(try proportionalTotal.subtracting(subtotal))",
            "\(proportionalTotal)",
            "\(try equalTotal.subtracting(subtotal))",
            "\(equalTotal)"
        ])
    }
    printTable(["Member", "Ordered", "Tip by consumption", "Pays", "Tip shared equally", "Pays"], rows, numbersFrom: 1)
    print("""
        By consumption, Carol, who ordered the most, adds the largest part of the tip;
        shared equally, everyone adds the same 1.80.
        """)

    section("3. A total that does not match is refused")
    do {
        _ = try group.add(Expense(cost: .local(try eur("55.00")), date: day, name: "Dinner", payers: .equal([ana]), split: .itemised(try receipt(ana, bob, carol))))
        preconditionFailure("the mismatch must be refused")
    } catch {
        print("""
            An amount of 55.00 EUR with these lines is refused when the expense is built:
              error: \(error.detail)
            """)
    }

    // Subtotals: Ana 8 + 3 + 3 = 14.00, Bob 14 + 3 = 17.00, Carol 14 + 6 + 3 = 23.00. A proportional tip adds 10 % to each;
    // an equal one 1.80 to everyone.
    let expected = try ["15.40", "18.70", "25.30", "24.80", "59.40"].map { text throws(EranosError) in try eur(text) }
    precondition(expected[0] == consumed(byConsumption, ana))
    precondition(expected[1] == consumed(byConsumption, bob))
    precondition(expected[2] == consumed(byConsumption, carol))
    precondition(expected[3] == consumed(even, carol))
    precondition(expected[4] == even?.total)
}

try main()
