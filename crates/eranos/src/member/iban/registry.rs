//! IBAN lengths per country, from the SWIFT IBAN Registry release 103
//! (September 2026, <https://www.swift.com/standards/data-standards/iban-international-bank-account-number>).
//!
//! Territories the registry lists only under another country (for example
//! Guadeloupe under FR) use that country's code and have no entry. Keep the
//! lines sorted by country code: lookups binary search the table.

/// Country code and IBAN length, sorted by country code.
pub(super) const LENGTHS: &[(&str, u8)] = &[
    ("AD", 24), // Andorra
    ("AE", 23), // United Arab Emirates (The)
    ("AL", 28), // Albania
    ("AT", 20), // Austria
    ("AZ", 28), // Azerbaijan
    ("BA", 20), // Bosnia and Herzegovina
    ("BE", 16), // Belgium
    ("BG", 22), // Bulgaria
    ("BH", 22), // Bahrain
    ("BI", 27), // Burundi
    ("BR", 29), // Brazil
    ("BY", 28), // Belarus
    ("CH", 21), // Switzerland
    ("CR", 22), // Costa Rica
    ("CY", 28), // Cyprus
    ("CZ", 24), // Czechia
    ("DE", 22), // Germany
    ("DJ", 27), // Djibouti
    ("DK", 18), // Denmark
    ("DO", 28), // Dominican Republic
    ("EE", 20), // Estonia
    ("EG", 29), // Egypt
    ("ES", 24), // Spain
    ("FI", 18), // Finland
    ("FK", 18), // Falkland Islands (Malvinas)
    ("FO", 18), // Faroe Islands
    ("FR", 27), // France
    ("GB", 22), // United Kingdom
    ("GE", 22), // Georgia
    ("GI", 23), // Gibraltar
    ("GL", 18), // Greenland
    ("GR", 27), // Greece
    ("GT", 28), // Guatemala
    ("HN", 28), // Honduras
    ("HR", 21), // Croatia
    ("HU", 28), // Hungary
    ("IE", 22), // Ireland
    ("IL", 23), // Israel
    ("IQ", 23), // Iraq
    ("IS", 26), // Iceland
    ("IT", 27), // Italy
    ("JO", 30), // Jordan
    ("KW", 30), // Kuwait
    ("KZ", 20), // Kazakhstan
    ("LB", 28), // Lebanon
    ("LC", 32), // Saint Lucia
    ("LI", 21), // Liechtenstein
    ("LT", 20), // Lithuania
    ("LU", 20), // Luxembourg
    ("LV", 21), // Latvia
    ("LY", 25), // Libya
    ("MC", 27), // Monaco
    ("MD", 24), // Moldova, Republic of
    ("ME", 22), // Montenegro
    ("MK", 19), // North Macedonia
    ("MN", 20), // Mongolia
    ("MR", 27), // Mauritania
    ("MT", 31), // Malta
    ("MU", 30), // Mauritius
    ("NI", 28), // Nicaragua
    ("NL", 18), // Netherlands (The)
    ("NO", 15), // Norway
    ("OM", 23), // Oman
    ("PK", 24), // Pakistan
    ("PL", 28), // Poland
    ("PS", 29), // Palestine, State of
    ("PT", 25), // Portugal
    ("QA", 29), // Qatar
    ("RO", 24), // Romania
    ("RS", 22), // Serbia
    ("RU", 33), // Russian Federation
    ("SA", 24), // Saudi Arabia
    ("SC", 31), // Seychelles
    ("SD", 18), // Sudan
    ("SE", 24), // Sweden
    ("SI", 19), // Slovenia
    ("SK", 24), // Slovakia
    ("SM", 27), // San Marino
    ("SO", 23), // Somalia
    ("ST", 25), // Sao Tome and Principe
    ("SV", 28), // El Salvador
    ("TL", 23), // Timor-Leste
    ("TN", 24), // Tunisia
    ("TR", 26), // Turkiye
    ("UA", 29), // Ukraine
    ("VA", 22), // Holy See
    ("VG", 24), // Virgin Islands (British)
    ("XK", 20), // Kosovo
    ("YE", 30), // Yemen
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
