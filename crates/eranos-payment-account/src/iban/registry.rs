//! IBAN lengths and account-number formats per country, from the SWIFT IBAN
//! Registry release 103 (September 2026,
//! <https://www.swift.com/standards/data-standards/iban-international-bank-account-number>).
//!
//! Territories the registry lists only under another country (for example
//! Guadeloupe under FR) use that country's code and have no entry. Keep the
//! lines sorted by country code: lookups binary search the table.

/// A country's entry in the registry.
pub(super) struct Country {
    /// The ISO 3166 code the IBAN starts with, such as `"DE"`.
    pub(super) code: &'static str,
    /// The number of characters, the four of the prefix included.
    pub(super) length: u8,
    /// The BBAN structure of the characters after the prefix: segments of
    /// a length, `!`, and a class, where `n` is digits, `a` uppercase
    /// letters, and `c` either, such as `"8!n10!n"` for eight digits, then
    /// ten.
    pub(super) structure: &'static str,
}

/// Every country in the registry, sorted by code.
// One line per country, so a registry update diffs line by line.
#[rustfmt::skip]
pub(super) const COUNTRIES: &[Country] = &[
    Country { code: "AD", length: 24, structure: "4!n4!n12!c" }, // Andorra
    Country { code: "AE", length: 23, structure: "3!n16!n" }, // United Arab Emirates (The)
    Country { code: "AL", length: 28, structure: "8!n16!c" }, // Albania
    Country { code: "AT", length: 20, structure: "5!n11!n" }, // Austria
    Country { code: "AZ", length: 28, structure: "4!a20!c" }, // Azerbaijan
    Country { code: "BA", length: 20, structure: "3!n3!n8!n2!n" }, // Bosnia and Herzegovina
    Country { code: "BE", length: 16, structure: "3!n7!n2!n" }, // Belgium
    Country { code: "BG", length: 22, structure: "4!a4!n2!n8!c" }, // Bulgaria
    Country { code: "BH", length: 22, structure: "4!a14!c" }, // Bahrain
    Country { code: "BI", length: 27, structure: "5!n5!n11!n2!n" }, // Burundi
    Country { code: "BR", length: 29, structure: "8!c5!n10!n1!c1!c" }, // Brazil
    Country { code: "BY", length: 28, structure: "4!c4!n16!c" }, // Belarus
    Country { code: "CH", length: 21, structure: "5!n12!c" }, // Switzerland
    Country { code: "CR", length: 22, structure: "4!n14!n" }, // Costa Rica
    Country { code: "CY", length: 28, structure: "3!n5!n16!c" }, // Cyprus
    Country { code: "CZ", length: 24, structure: "4!n16!n" }, // Czechia
    Country { code: "DE", length: 22, structure: "8!n10!n" }, // Germany
    Country { code: "DJ", length: 27, structure: "5!n5!n11!n2!n" }, // Djibouti
    Country { code: "DK", length: 18, structure: "4!n9!n1!n" }, // Denmark
    Country { code: "DO", length: 28, structure: "4!c20!n" }, // Dominican Republic
    Country { code: "EE", length: 20, structure: "2!n14!n" }, // Estonia
    Country { code: "EG", length: 29, structure: "4!n4!n17!n" }, // Egypt
    Country { code: "ES", length: 24, structure: "4!n4!n1!n1!n10!n" }, // Spain
    Country { code: "FI", length: 18, structure: "3!n11!n" }, // Finland
    Country { code: "FK", length: 18, structure: "2!a12!n" }, // Falkland Islands (Malvinas)
    Country { code: "FO", length: 18, structure: "4!n9!n1!n" }, // Faroe Islands
    Country { code: "FR", length: 27, structure: "5!n5!n11!c2!n" }, // France
    Country { code: "GB", length: 22, structure: "4!a6!n8!n" }, // United Kingdom
    Country { code: "GE", length: 22, structure: "2!a16!n" }, // Georgia
    Country { code: "GI", length: 23, structure: "4!a15!c" }, // Gibraltar
    Country { code: "GL", length: 18, structure: "4!n9!n1!n" }, // Greenland
    Country { code: "GR", length: 27, structure: "3!n4!n16!c" }, // Greece
    Country { code: "GT", length: 28, structure: "4!c20!c" }, // Guatemala
    Country { code: "HN", length: 28, structure: "4!a20!n" }, // Honduras
    Country { code: "HR", length: 21, structure: "7!n10!n" }, // Croatia
    Country { code: "HU", length: 28, structure: "3!n4!n1!n15!n1!n" }, // Hungary
    Country { code: "IE", length: 22, structure: "4!a6!n8!n" }, // Ireland
    Country { code: "IL", length: 23, structure: "3!n3!n13!n" }, // Israel
    Country { code: "IQ", length: 23, structure: "4!a3!n12!n" }, // Iraq
    Country { code: "IS", length: 26, structure: "4!n2!n6!n10!n" }, // Iceland
    Country { code: "IT", length: 27, structure: "1!a5!n5!n12!c" }, // Italy
    Country { code: "JO", length: 30, structure: "4!a4!n18!c" }, // Jordan
    Country { code: "KW", length: 30, structure: "4!a22!c" }, // Kuwait
    Country { code: "KZ", length: 20, structure: "3!n13!c" }, // Kazakhstan
    Country { code: "LB", length: 28, structure: "4!n20!c" }, // Lebanon
    Country { code: "LC", length: 32, structure: "4!a24!c" }, // Saint Lucia
    Country { code: "LI", length: 21, structure: "5!n12!c" }, // Liechtenstein
    Country { code: "LT", length: 20, structure: "5!n11!n" }, // Lithuania
    Country { code: "LU", length: 20, structure: "3!n13!c" }, // Luxembourg
    Country { code: "LV", length: 21, structure: "4!a13!c" }, // Latvia
    Country { code: "LY", length: 25, structure: "3!n3!n15!n" }, // Libya
    Country { code: "MC", length: 27, structure: "5!n5!n11!c2!n" }, // Monaco
    Country { code: "MD", length: 24, structure: "2!c18!c" }, // Moldova, Republic of
    Country { code: "ME", length: 22, structure: "3!n13!n2!n" }, // Montenegro
    Country { code: "MK", length: 19, structure: "3!n10!c2!n" }, // North Macedonia
    Country { code: "MN", length: 20, structure: "4!n12!n" }, // Mongolia
    Country { code: "MR", length: 27, structure: "5!n5!n11!n2!n" }, // Mauritania
    Country { code: "MT", length: 31, structure: "4!a5!n18!c" }, // Malta
    Country { code: "MU", length: 30, structure: "4!a2!n2!n12!n3!n3!a" }, // Mauritius
    Country { code: "NI", length: 28, structure: "4!a20!n" }, // Nicaragua
    Country { code: "NL", length: 18, structure: "4!a10!n" }, // Netherlands (The)
    Country { code: "NO", length: 15, structure: "4!n6!n1!n" }, // Norway
    Country { code: "OM", length: 23, structure: "3!n16!c" }, // Oman
    Country { code: "PK", length: 24, structure: "4!a16!c" }, // Pakistan
    Country { code: "PL", length: 28, structure: "8!n16!n" }, // Poland
    Country { code: "PS", length: 29, structure: "4!a21!c" }, // Palestine, State of
    Country { code: "PT", length: 25, structure: "4!n4!n11!n2!n" }, // Portugal
    Country { code: "QA", length: 29, structure: "4!a21!c" }, // Qatar
    Country { code: "RO", length: 24, structure: "4!a16!c" }, // Romania
    Country { code: "RS", length: 22, structure: "3!n13!n2!n" }, // Serbia
    Country { code: "RU", length: 33, structure: "9!n5!n15!c" }, // Russian Federation
    Country { code: "SA", length: 24, structure: "2!n18!c" }, // Saudi Arabia
    Country { code: "SC", length: 31, structure: "4!a2!n2!n16!n3!a" }, // Seychelles
    Country { code: "SD", length: 18, structure: "2!n12!n" }, // Sudan
    Country { code: "SE", length: 24, structure: "3!n16!n1!n" }, // Sweden
    Country { code: "SI", length: 19, structure: "5!n8!n2!n" }, // Slovenia
    Country { code: "SK", length: 24, structure: "4!n6!n10!n" }, // Slovakia
    Country { code: "SM", length: 27, structure: "1!a5!n5!n12!c" }, // San Marino
    Country { code: "SO", length: 23, structure: "4!n3!n12!n" }, // Somalia
    Country { code: "ST", length: 25, structure: "4!n4!n11!n2!n" }, // Sao Tome and Principe
    Country { code: "SV", length: 28, structure: "4!a20!n" }, // El Salvador
    Country { code: "TL", length: 23, structure: "3!n14!n2!n" }, // Timor-Leste
    Country { code: "TN", length: 24, structure: "2!n3!n13!n2!n" }, // Tunisia
    Country { code: "TR", length: 26, structure: "5!n1!n16!c" }, // Turkiye
    Country { code: "UA", length: 29, structure: "6!n19!c" }, // Ukraine
    Country { code: "VA", length: 22, structure: "3!n15!n" }, // Holy See
    Country { code: "VG", length: 24, structure: "4!a16!n" }, // Virgin Islands (British)
    Country { code: "XK", length: 20, structure: "4!n10!n2!n" }, // Kosovo
    Country { code: "YE", length: 30, structure: "4!a4!n18!c" }, // Yemen
];

/// The registry's own example IBAN for every country, in the same order.
#[cfg(test)]
pub(super) const EXAMPLES: &[&str] = &[
    "AD1200012030200359100100",
    "AE070331234567890123456",
    "AL47212110090000000235698741",
    "AT611904300234573201",
    "AZ21NABZ00000000137010001944",
    "BA391290079401028494",
    "BE68539007547034",
    "BG80BNBG96611020345678",
    "BH67BMAG00001299123456",
    "BI4210000100010000332045181",
    "BR6699999A03000010009795493C1",
    "BY13NBRB3600900000002Z00AB00",
    "CH9300762011623852957",
    "CR05015202001026284066",
    "CY17002001280000001200527600",
    "CZ6508000000192000145399",
    "DE89370400440532013000",
    "DJ2100010000000154000100186",
    "DK5000400440116243",
    "DO28BAGR00000001212453611324",
    "EE382200221020145685",
    "EG380019000500000000263180002",
    "ES9121000418450200051332",
    "FI2112345600000785",
    "FK88SC123456789012",
    "FO6264600001631634",
    "FR1420041010050500013M02606",
    "GB29NWBK60161331926819",
    "GE29NB0000000101904917",
    "GI75NWBK000000007099453",
    "GL8964710001000206",
    "GR1601101250000000012300695",
    "GT82TRAJ01020000001210029690",
    "HN88CABF00000000000250005469",
    "HR1210010051863000160",
    "HU42117730161111101800000000",
    "IE29AIBK93115212345678",
    "IL620108000000099999999",
    "IQ98NBIQ850123456789012",
    "IS140159260076545510730339",
    "IT60X0542811101000000123456",
    "JO94CBJO0010000000000131000302",
    "KW81CBKU0000000000001234560101",
    "KZ86125KZT5004100100",
    "LB62099900000001001901229114",
    "LC55HEMM000100010012001200023015",
    "LI21088100002324013AA",
    "LT121000011101001000",
    "LU280019400644750000",
    "LV80BANK0000435195001",
    "LY83002048000020100120361",
    "MC5811222000010123456789030",
    "MD24AG000225100013104168",
    "ME25505000012345678951",
    "MK07250120000058984",
    "MN121234123456789123",
    "MR1300020001010000123456753",
    "MT84MALT011000012345MTLCAST001S",
    "MU17BOMM0101101030300200000MUR",
    "NI45BAPR00000013000003558124",
    "NL91ABNA0417164300",
    "NO9386011117947",
    "OM810180000001299123456",
    "PK36SCBL0000001123456702",
    "PL61109010140000071219812874",
    "PS92PALS000000000400123456702",
    "PT50000201231234567890154",
    "QA58DOHB00001234567890ABCDEFG",
    "RO49AAAA1B31007593840000",
    "RS35260005601001611379",
    "RU0304452522540817810538091310419",
    "SA0380000000608010167519",
    "SC18SSCB11010000000000001497USD",
    "SD2129010501234001",
    "SE4550000000058398257466",
    "SI56263300012039086",
    "SK3112000000198742637541",
    "SM86U0322509800000000270100",
    "SO211000001001000100141",
    "ST23000100010051845310146",
    "SV62CENR00000000000000700025",
    "TL380080012345678910157",
    "TN5910006035183598478831",
    "TR330006100519786457841326",
    "UA213223130000026007233566001",
    "VA59001123000012345678",
    "VG96VPVG0000012345678901",
    "XK051212012345678906",
    "YE15CBYE0001018861234567891234",
];
