//! The values of the C and C++ APIs: amounts, dates, times, lists of text, and the helpers that parse,
//! write, and check values as the library does.
//!
//! The objects of phone numbers and IBANs, which the library parses, are written with the records, in
//! the module `generated`.

/// The types and functions of the C and C++ APIs; C names each type as it is here, and C++ puts it in the
/// namespace `eranos::detail`.
#[expect(
    clippy::unnecessary_box_returns,
    reason = "the functions Diplomat writes hand out an opaque type only in a box"
)]
#[diplomat::bridge]
#[diplomat::attr(cpp, namespace = "eranos::detail")]
pub mod ffi {
    use crate::error::ffi::EranosError;
    use crate::generated::records::ffi::EranosCurrencyDetails;
    use diplomat_runtime::{DiplomatStr, DiplomatWrite};

    /// A day of the calendar.
    #[derive(Debug, Clone, Copy)]
    pub struct EranosCalendarDate {
        /// The year.
        pub year: i32,
        /// The month, from 1 for January.
        pub month: i32,
        /// The day of the month, from 1.
        pub day: i32,
    }

    impl EranosCalendarDate {
        /// The date written `text`: "2026-09-23".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Calendar` for text that is not a date, and of the kind `InvalidText` for text
        /// that is not UTF-8.
        pub fn parse(text: &DiplomatStr) -> Result<Self, Box<EranosError>> {
            eranos_bindings::calendar_date_parse(&crate::text(text)?)
                .map(Into::into)
                .map_err(crate::failed)
        }

        /// Writes the date as the library does: "2026-09-23". It writes the fields of a day the
        /// calendar does not have, which `check` refuses, the same way: "2026-02-30".
        pub fn format(self, write: &mut DiplomatWrite) {
            crate::write_text(write, &eranos_bindings::calendar_date_format(self.into()));
        }

        /// Checks that the date is a day of the calendar, between years 1 and 9999, as an entry's
        /// date must be.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Calendar` for a day the calendar does not have.
        pub fn check(self) -> Result<(), Box<EranosError>> {
            eranos_bindings::calendar_date_check(self.into()).map_err(crate::failed)
        }
    }

    /// A time of day, to the minute.
    #[derive(Debug, Clone, Copy)]
    pub struct EranosTimeOfDay {
        /// The hour, from 0 to 23.
        pub hour: i32,
        /// The minute, from 0 to 59.
        pub minute: i32,
    }

    impl EranosTimeOfDay {
        /// The time written `text`: "18:30".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Calendar` for text that is not a time, and of the kind `InvalidText` for text
        /// that is not UTF-8.
        pub fn parse(text: &DiplomatStr) -> Result<Self, Box<EranosError>> {
            eranos_bindings::time_of_day_parse(&crate::text(text)?)
                .map(Into::into)
                .map_err(crate::failed)
        }

        /// Writes the time as the library does: "18:30". It writes the fields of a time the day
        /// does not have, which `check` refuses, the same way: "24:00".
        pub fn format(self, write: &mut DiplomatWrite) {
            crate::write_text(write, &eranos_bindings::time_of_day_format(self.into()));
        }

        /// Checks that the time is a time of day, from 00:00 to 23:59, as an entry's time must be.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Calendar` for a time the day does not have.
        pub fn check(self) -> Result<(), Box<EranosError>> {
            eranos_bindings::time_of_day_check(self.into()).map_err(crate::failed)
        }
    }

    /// A list of text: identifiers, for the most part.
    #[diplomat::opaque_mut]
    #[derive(Debug)]
    pub struct EranosTextList(pub(crate) Vec<String>);

    impl EranosTextList {
        /// An empty list.
        #[must_use]
        pub fn create() -> Box<Self> {
            Box::new(Self(Vec::new()))
        }

        /// Adds `text` at the end.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn push(&mut self, text: &DiplomatStr) -> Result<(), Box<EranosError>> {
            self.0.push(crate::text(text)?);
            Ok(())
        }

        /// How many texts the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no text.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// Writes the text at `index`, if the list holds one there; absent past the end.
        pub fn get(&self, index: usize, write: &mut DiplomatWrite) -> Option<()> {
            crate::write_optional(write, self.0.get(index).map(String::as_str))
        }
    }

    /// An amount: a whole number of minor units of a currency, cents for the euro.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosMoney(pub(crate) eranos_bindings::Money);

    impl EranosMoney {
        /// `minor_units` of `currency`, whose details come from the library: from
        /// `EranosCurrencyDetails::of`, the list of every currency, or another amount, so the
        /// amount needs no lookup and no check of its own.
        #[must_use]
        pub fn create(minor_units: i64, currency: &EranosCurrencyDetails) -> Box<Self> {
            Box::new(Self(eranos_bindings::Money {
                minor_units,
                currency: currency.0.clone(),
            }))
        }

        /// The amount in minor units: cents for the euro, none for the yen.
        #[must_use]
        pub const fn minor_units(&self) -> i64 {
            self.0.minor_units
        }

        /// The currency, with what ISO 4217 says of it.
        #[must_use]
        pub fn currency(&self) -> Box<EranosCurrencyDetails> {
            Box::new(EranosCurrencyDetails(self.0.currency.clone()))
        }

        /// The amount written `text` in `currency`, whose details come from the library, as for
        /// `create`: "12.50", "12.5", or "-3".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for text that is not an amount of the currency, and of the kind
        /// `InvalidText` for text that is not UTF-8.
        pub fn parse(
            text: &DiplomatStr,
            currency: &EranosCurrencyDetails,
        ) -> Result<Box<Self>, Box<EranosError>> {
            eranos_bindings::money_parse(&crate::text(text)?, &currency.0.code)
                .map(|money| Box::new(Self(money)))
                .map_err(crate::failed)
        }

        /// The amount of the double `value` in `currency`, whose details come from the library, as
        /// for `create`: 1.005 in EUR is 1.01 EUR. The library writes the double in its shortest
        /// decimal form, rounds it half away from zero to the currency's decimal places, and parses
        /// it, so every binding gets the same amount from the same double.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a value that is NaN or infinite, or an amount
        /// out of range.
        pub fn from_double(
            value: f64,
            currency: &EranosCurrencyDetails,
        ) -> Result<Box<Self>, Box<EranosError>> {
            eranos_bindings::money_from_f64(value, &currency.0.code)
                .map(|money| Box::new(Self(money)))
                .map_err(crate::failed)
        }

        /// Writes the amount as the library does: "12.50 EUR".
        pub fn format(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &eranos_bindings::money_format(&self.0));
        }

        /// This amount plus `other`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for amounts in two currencies, or a sum out of range.
        pub fn add(&self, other: &Self) -> Result<Box<Self>, Box<EranosError>> {
            eranos_bindings::money_add(&self.0, &other.0)
                .map(|money| Box::new(Self(money)))
                .map_err(crate::failed)
        }

        /// This amount minus `other`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for amounts in two currencies, or a difference out of range.
        pub fn subtract(&self, other: &Self) -> Result<Box<Self>, Box<EranosError>> {
            eranos_bindings::money_subtract(&self.0, &other.0)
                .map(|money| Box::new(Self(money)))
                .map_err(crate::failed)
        }
    }

    /// The helpers of exchange rates, which cross as decimals. It holds nothing, and nothing makes
    /// one.
    #[diplomat::opaque]
    #[derive(Debug, Clone, Copy)]
    pub struct EranosExchangeRate;

    impl EranosExchangeRate {
        /// Writes the exchange rate written `text` as the library writes it: "4.97".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for text that is not a positive decimal, and of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn parse(
            text: &DiplomatStr,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text =
                eranos_bindings::exchange_rate_parse(&crate::text(text)?).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// Writes the exchange rate of the double `value` as the library writes it: 4.9765 is
        /// "4.9765". The library writes the double in its shortest decimal form, rounds it half away
        /// from zero to 18 decimal places, and parses it, so 25.5 is "25.5".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a value that is NaN or infinite, or that is not a
        /// positive rate at 18 decimal places.
        pub fn from_double(value: f64, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = eranos_bindings::exchange_rate_from_f64(value).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// The amount `amount` converted at the rate `rate` into the currency `currency`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a rate that is not one, a currency the library
        /// does not list, or an amount out of range, and of the kind `InvalidText` for text that is
        /// not UTF-8.
        pub fn convert(
            rate: &DiplomatStr,
            amount: &EranosMoney,
            currency: &DiplomatStr,
        ) -> Result<Box<EranosMoney>, Box<EranosError>> {
            eranos_bindings::exchange_rate_convert(
                &crate::text(rate)?,
                &amount.0,
                &crate::text(currency)?,
            )
            .map(|money| Box::new(EranosMoney(money)))
            .map_err(crate::failed)
        }
    }

    /// The helpers of percentages, which cross in basis points. It holds nothing, and nothing makes
    /// one.
    #[diplomat::opaque]
    #[derive(Debug, Clone, Copy)]
    pub struct EranosPercentage;

    impl EranosPercentage {
        /// The percentage written `text`, in basis points: "12.5%" is 1250.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for text that is not a percentage, and of the kind
        /// `InvalidText` for text that is not UTF-8.
        pub fn parse(text: &DiplomatStr) -> Result<i32, Box<EranosError>> {
            eranos_bindings::percentage_parse(&crate::text(text)?).map_err(crate::failed)
        }

        /// The percentage of the double `value`, a number of per cent, in basis points: 12.345 is
        /// 1235. The library writes the double in its shortest decimal form, rounds it half away
        /// from zero to two decimal places, and parses it.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a value that is NaN or infinite, negative, or out
        /// of range.
        pub fn from_double(value: f64) -> Result<i32, Box<EranosError>> {
            eranos_bindings::percentage_from_f64(value).map_err(crate::failed)
        }

        /// Writes the percentage of `basis_points` basis points as the library does: "12.5%". It
        /// writes a negative number, which `check` refuses, the same way after a minus sign:
        /// "-12.5%".
        pub fn format(basis_points: i32, write: &mut DiplomatWrite) {
            crate::write_text(write, &eranos_bindings::percentage_format(basis_points));
        }

        /// Checks that `basis_points` is a percentage, as an entry that holds one requires.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a negative number.
        pub fn check(basis_points: i32) -> Result<(), Box<EranosError>> {
            eranos_bindings::percentage_check(basis_points).map_err(crate::failed)
        }
    }

    /// The helpers of parts, which cross in hundredths. It holds nothing, and nothing makes one.
    #[diplomat::opaque]
    #[derive(Debug, Clone, Copy)]
    pub struct EranosParts;

    impl EranosParts {
        /// The parts written `text`, in hundredths: "0.5" is 50.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for text that is not a number of parts, and of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn parse(text: &DiplomatStr) -> Result<i32, Box<EranosError>> {
            eranos_bindings::parts_parse(&crate::text(text)?).map_err(crate::failed)
        }

        /// The parts of the double `value`, in hundredths: 1.005 is 101. The library writes the
        /// double in its shortest decimal form, rounds it half away from zero to two decimal places,
        /// and parses it.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a value that is NaN or infinite, rounds to zero
        /// or below, or is out of range.
        pub fn from_double(value: f64) -> Result<i32, Box<EranosError>> {
            eranos_bindings::parts_from_f64(value).map_err(crate::failed)
        }

        /// Writes the parts of `hundredths` hundredths as the library does: "0.5". It writes zero
        /// and a negative number, which `check` refuses, the same way: "0" and "-0.5".
        pub fn format(hundredths: i32, write: &mut DiplomatWrite) {
            crate::write_text(write, &eranos_bindings::parts_format(hundredths));
        }

        /// Checks that `hundredths` is a number of parts, as a split of parts requires.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a number that is not positive.
        pub fn check(hundredths: i32) -> Result<(), Box<EranosError>> {
            eranos_bindings::parts_check(hundredths).map_err(crate::failed)
        }
    }

    /// The helpers of quantities, which cross in ten-thousandths. It holds nothing, and nothing
    /// makes one.
    #[diplomat::opaque]
    #[derive(Debug, Clone, Copy)]
    pub struct EranosQuantity;

    impl EranosQuantity {
        /// The quantity written `text`, in ten-thousandths: "0.452" is 4520.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for text that is not a positive decimal with at most
        /// four decimal places, or a quantity out of range, and of the kind `InvalidText` for text
        /// that is not UTF-8.
        pub fn parse(text: &DiplomatStr) -> Result<i64, Box<EranosError>> {
            eranos_bindings::quantity_parse(&crate::text(text)?).map_err(crate::failed)
        }

        /// The quantity of `units` whole units, in ten-thousandths: 3 is 30000.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a number that is not positive, or out of range.
        pub fn from_whole(units: i64) -> Result<i64, Box<EranosError>> {
            eranos_bindings::quantity_from_whole(units).map_err(crate::failed)
        }

        /// The quantity of the double `value`, in ten-thousandths: 0.45205 is 4521. The library
        /// writes the double in its shortest decimal form, rounds it half away from zero to four
        /// decimal places, and parses it.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a value that is NaN or infinite, rounds to zero
        /// or below, or is out of range.
        pub fn from_double(value: f64) -> Result<i64, Box<EranosError>> {
            eranos_bindings::quantity_from_f64(value).map_err(crate::failed)
        }

        /// Writes the quantity of `ten_thousandths` ten-thousandths as the library does: "0.452".
        /// It writes zero and a negative number, which `check` refuses, the same way: "0" and
        /// "-0.452".
        pub fn format(ten_thousandths: i64, write: &mut DiplomatWrite) {
            crate::write_text(write, &eranos_bindings::quantity_format(ten_thousandths));
        }

        /// Checks that `ten_thousandths` is a quantity, as an item requires.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a number that is not positive.
        pub fn check(ten_thousandths: i64) -> Result<(), Box<EranosError>> {
            eranos_bindings::quantity_check(ten_thousandths).map_err(crate::failed)
        }
    }

    /// The helpers of identifiers, whose rules every kind of identifier shares. It holds nothing,
    /// and nothing makes one.
    #[diplomat::opaque]
    #[derive(Debug, Clone, Copy)]
    pub struct EranosIdentifier;

    impl EranosIdentifier {
        /// Writes a new identifier, which any entry or group can take.
        ///
        /// It is a UUID v7 as lowercase hyphenated text. Every kind of identifier draws from the
        /// one generator of the library.
        pub fn generate(write: &mut DiplomatWrite) {
            crate::write_text(write, &eranos_bindings::identifier_generate());
        }

        /// Writes the identifier written `text`, as it is: not trimmed, and in its case.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Id` for text that is not an identifier, with the first rule
        /// it breaks, and of the kind `InvalidText` for text that is not UTF-8.
        pub fn parse(
            text: &DiplomatStr,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text =
                eranos_bindings::identifier_parse(&crate::text(text)?).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// Writes the identifier of the built-in category, Uncategorised, which every group has and
        /// every expense is in until it is given another: the text of the nil UUID,
        /// `00000000-0000-0000-0000-000000000000`.
        pub fn uncategorised(write: &mut DiplomatWrite) {
            crate::write_text(write, &eranos_bindings::category_id_uncategorised());
        }
    }

    /// The helpers of language tags. It holds nothing, and nothing makes one.
    #[diplomat::opaque]
    #[derive(Debug, Clone, Copy)]
    pub struct EranosLanguage;

    impl EranosLanguage {
        /// Writes the language written `text` as a canonical BCP 47 tag: "pt-PT".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Language` for text that is not a language tag, and of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn parse(
            text: &DiplomatStr,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text =
                eranos_bindings::language_parse(&crate::text(text)?).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }
    }

    /// The helpers of email addresses. It holds nothing, and nothing makes one.
    #[diplomat::opaque]
    #[derive(Debug, Clone, Copy)]
    pub struct EranosEmail;

    impl EranosEmail {
        /// Writes the email address written `text`, checked and trimmed.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Contact` for text that is not an email address, and of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn parse(
            text: &DiplomatStr,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text = eranos_bindings::email_parse(&crate::text(text)?).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }
    }

    /// The helpers of the usernames of payment apps. It holds nothing, and nothing makes one.
    #[diplomat::opaque]
    #[derive(Debug, Clone, Copy)]
    pub struct EranosUsername;

    impl EranosUsername {
        /// Writes the username written `text`, trimmed.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `PaymentAccount` for a blank username, and of the kind
        /// `InvalidText` for text that is not UTF-8.
        pub fn parse(
            text: &DiplomatStr,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text =
                eranos_bindings::username_parse(&crate::text(text)?).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }
    }
}

impl From<ffi::EranosCalendarDate> for eranos_bindings::CalendarDate {
    fn from(date: ffi::EranosCalendarDate) -> Self {
        Self {
            year: date.year,
            month: date.month,
            day: date.day,
        }
    }
}

impl From<eranos_bindings::CalendarDate> for ffi::EranosCalendarDate {
    fn from(date: eranos_bindings::CalendarDate) -> Self {
        Self {
            year: date.year,
            month: date.month,
            day: date.day,
        }
    }
}

impl From<ffi::EranosTimeOfDay> for eranos_bindings::TimeOfDay {
    fn from(time: ffi::EranosTimeOfDay) -> Self {
        Self {
            hour: time.hour,
            minute: time.minute,
        }
    }
}

impl From<eranos_bindings::TimeOfDay> for ffi::EranosTimeOfDay {
    fn from(time: eranos_bindings::TimeOfDay) -> Self {
        Self {
            hour: time.hour,
            minute: time.minute,
        }
    }
}
