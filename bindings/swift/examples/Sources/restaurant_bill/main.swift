// An itemised restaurant bill: every line is split among the people who had it, quantities multiply the unit price, and
// the tip is shared either in proportion to what each person ordered or equally. The port of the Rust example
// `restaurant_bill`, which prints the same report.

import Eranos
import Report

struct LineRow: TableRow {
    static let headers = ["Item", "Shared by", "Quantity", "Unit price", "Line total"]

    let name: String
    let participants: String
    let quantity: Quantity
    let unitPrice: Money
    let lineTotal: Money

    var cells: [String] {
        [name, participants, "\(quantity)", "\(unitPrice)", "\(lineTotal)"]
    }
}

struct TipRow: TableRow {
    static let headers = ["Member", "Ordered", "Tip by consumption", "Pays", "Tip shared equally", "Pays"]

    let name: String
    let subtotal: Money
    let proportionalTip: Money
    let proportionalTotal: Money
    let equalTip: Money
    let equalTotal: Money

    var cells: [String] {
        [name, "\(subtotal)", "\(proportionalTip)", "\(proportionalTotal)", "\(equalTip)", "\(equalTotal)"]
    }
}

let euro = try CurrencyCode(code: "EUR")

/// The amount written `text`, in euros.
func eur(_ text: String) throws(EranosError) -> Money {
    try Money(parsing: text, in: euro)
}

/// Stops the example unless `found` is the amount written `expected`, in euros.
func expect(_ expected: String, _ found: Money?) throws(EranosError) {
    let amount = try eur(expected)
    precondition(amount == found)
}

/// The receipt: every line with the members who had it.
func receipt(_ ana: Member.ID, _ bob: Member.ID, _ carol: Member.ID) throws(EranosError) -> [ExpenseItem] {
    [
        ExpenseItem(name: "Ciorbă", unitPrice: try eur("8.00"), split: .equal([ana])),
        ExpenseItem(name: "Sarmale", unitPrice: try eur("14.00"), split: .equal([bob, carol]), quantity: try Quantity(whole: 2)),
        ExpenseItem(
            name: "Papanași",
            unitPrice: try eur("9.00"),
            split: .parts([MemberParts(member: ana, parts: try Parts(hundredths: 100)), MemberParts(member: carol, parts: try Parts(hundredths: 200))])
        ),
        ExpenseItem(name: "Lemonade", unitPrice: try eur("3.00"), split: .equal([ana, bob, carol]), quantity: try Quantity(whole: 3))
    ]
}

/// How `expense` would divide if it were added to `group`, computed on a copy loaded from the group's saved form, so
/// the group itself is unchanged.
func preview(_ group: ExpenseGroup, _ expense: Expense) throws(EranosError) -> ExpenseShares? {
    let copy = try ExpenseGroup.load(try group.save())
    return try copy.shares(of: try copy.add(expense))
}

/// What `member` consumed of the expense that `shares` divides.
func consumed(_ shares: ExpenseShares?, _ member: Member.ID) -> Money? {
    shares?.members.first { share in share.member == member }?.share
}

func main() throws(EranosError) {
    let group = try ExpenseGroup(name: "Dinner at Caru' cu Bere", currency: euro)
    let ana = try group.add(Member(firstName: "Ana"))
    let bob = try group.add(Member(firstName: "Bob"))
    let carol = try group.add(Member(firstName: "Carol"))
    let day = try CalendarDate(year: 2026, month: 9, day: 23)

    title("\(try group.name()): an itemised bill")
    print("""
        Three friends share a Romanian dinner. Instead of splitting the whole bill evenly,
        every line of the receipt is split among the people who had it.
        """)

    section("1. The receipt")
    printTable(
        try receipt(ana, bob, carol).map { item throws(EranosError) in
            LineRow(
                name: item.name,
                participants: try names(group, item.split.members),
                quantity: item.quantity,
                unitPrice: item.unitPrice,
                lineTotal: try item.lineTotal()
            )
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
            name: "Dinner",
            cost: .local(try eur("54.00")),
            payers: .equal([ana]),
            split: .itemised(try receipt(ana, bob, carol)),
            date: day,
            tip: TipRule(value: .percentage(try Percentage(parsing: "10")), distribution: distribution)
        )
    }
    let untipped = Expense(name: "Dinner", cost: .local(try eur("54.00")), payers: .equal([ana]), split: .itemised(try receipt(ana, bob, carol)), date: day)
    let ordered = try preview(group, untipped)
    let proportional = try group.add(try bill(.proportional))
    let byConsumption = try group.shares(of: proportional)
    let even = try preview(group, try bill(.equal))
    print("""
        Ana paid the bill plus a 10 % tip (\(byConsumption?.tip.description ?? "?")). A tip can be shared in proportion to what
        each person ordered, or equally; both are shown:
        """)
    var rows: [TipRow] = []
    for member in [ana, bob, carol] {
        guard let subtotal = consumed(ordered, member),
              let proportionalTotal = consumed(byConsumption, member),
              let equalTotal = consumed(even, member) else {
            preconditionFailure("every member ordered")
        }
        rows.append(
            TipRow(
                name: try names(group, [member]),
                subtotal: subtotal,
                proportionalTip: try proportionalTotal.subtracting(subtotal),
                proportionalTotal: proportionalTotal,
                equalTip: try equalTotal.subtracting(subtotal),
                equalTotal: equalTotal
            )
        )
    }
    printTable(rows, numbersFrom: 1)
    print("""
        By consumption, Carol, who ordered the most, adds the largest part of the tip;
        shared equally, everyone adds the same 1.80.
        """)

    section("3. A total that does not match is refused")
    do {
        _ = try group.add(Expense(name: "Dinner", cost: .local(try eur("55.00")), payers: .equal([ana]), split: .itemised(try receipt(ana, bob, carol)), date: day))
        preconditionFailure("the mismatch must be refused")
    } catch {
        print("""
            An amount of 55.00 EUR with these lines is refused when the expense is built:
              error: \(error.detail)
            """)
    }

    // Subtotals: Ana 8 + 3 + 3 = 14.00, Bob 14 + 3 = 17.00, Carol 14 + 6 + 3 = 23.00. A proportional tip adds 10 % to each;
    // an equal one 1.80 to everyone.
    try expect("15.40", consumed(byConsumption, ana))
    try expect("18.70", consumed(byConsumption, bob))
    try expect("25.30", consumed(byConsumption, carol))
    try expect("24.80", consumed(even, carol))
    try expect("59.40", even?.total)
}

try main()
