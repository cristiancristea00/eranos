// Every feature of the library in one story: a group and its details, members with contact details and accounts,
// categories, an expense for every way of dividing one, another currency, edits and refusals, members who come and go,
// the summary, a new group currency, settling up, saving and loading, the values on their own, and the group shared
// between tasks. Every section checks its key numbers. The port of the Rust example `grand_tour`, which prints the
// same report.

import Eranos

func main() async throws(EranosError) {
    print("Lisbon and Porto: a grand tour of eranos")
    print("════════════════════════════════════════")
    print("""
        Ana, Bob, Carol, Dan, and Carol's daughter Eva spend a week in Portugal. Each
        section below uses another part of the library, from creating the group to saving
        it, and checks its key numbers.
        """)
    let group = try theGroup()
    let people = try members(group)
    let categories = try addCategories(group)
    let seafood = try expenses(group, people, categories)
    try anotherCurrency(group, people, categories)
    try edits(group, people, categories, seafood)
    try comingsAndGoings(group, people)
    try summary(group, people, categories)
    try removingACategory(group, categories)
    try newCurrency(group, people)
    try settlingUp(group)
    try savingAndLoading(group, people, categories)
    try values(group, people)
    try await sharing(group, people)
}

try await main()
