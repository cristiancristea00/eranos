//! The values of the C and C++ APIs: amounts, currencies, dates, times, lists of text, and the helpers that
//! parse and write values as the library does.

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
    use diplomat_runtime::{DiplomatStr, DiplomatWrite};

    /// A day of the calendar.
    #[derive(Debug, Clone, Copy)]
    pub struct EranosCalendarDate {
        /// The day of the month, from 1.
        pub day: i32,
        /// The month, from 1 for January.
        pub month: i32,
        /// The year.
        pub year: i32,
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

        /// Writes the date as the library does: "2026-09-23".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Calendar` for a day the calendar does not have.
        pub fn format(self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = eranos_bindings::calendar_date_format(self.into()).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
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

        /// Writes the time as the library does: "18:30".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Calendar` for a time the day does not have.
        pub fn format(self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = eranos_bindings::time_of_day_format(self.into()).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
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
        /// A new money of its required fields; every other field starts absent, empty, or at its
        /// default.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `InvalidText` for text that is not UTF-8.
        pub fn create(
            currency: &DiplomatStr,
            minor_units: i64,
        ) -> Result<Box<Self>, Box<EranosError>> {
            Ok(Box::new(Self(eranos_bindings::Money {
                currency: crate::text(currency)?,
                minor_units,
            })))
        }

        /// The currency, as an ISO 4217 code.
        pub fn currency(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.currency);
        }

        /// The amount in minor units: cents for the euro, none for the yen.
        #[must_use]
        pub const fn minor_units(&self) -> i64 {
            self.0.minor_units
        }

        /// The amount written `text` in the currency `currency`: "12.50", "12.5", or "-3".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for text that is not an amount of the currency, and of the kind
        /// `InvalidText` for text that is not UTF-8.
        pub fn parse(
            text: &DiplomatStr,
            currency: &DiplomatStr,
        ) -> Result<Box<Self>, Box<EranosError>> {
            eranos_bindings::money_parse(&crate::text(text)?, &crate::text(currency)?)
                .map(|money| Box::new(Self(money)))
                .map_err(crate::failed)
        }

        /// Writes the amount as the library does: "12.50 EUR".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a currency the library does not list.
        pub fn format(&self, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = eranos_bindings::money_format(&self.0).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
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

    /// A currency of the ISO 4217 list, with what the library knows of it.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosCurrencyDetails(pub(crate) eranos_bindings::CurrencyDetails);

    impl EranosCurrencyDetails {
        /// The ISO 4217 code.
        pub fn code(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.code);
        }

        /// How many decimal places its amounts have.
        #[must_use]
        pub const fn decimal_places(&self) -> i32 {
            self.0.decimal_places
        }

        /// The English name.
        pub fn name(&self, write: &mut DiplomatWrite) {
            crate::write_text(write, &self.0.name);
        }

        /// The ISO 4217 numeric code.
        #[must_use]
        pub const fn numeric_code(&self) -> i32 {
            self.0.numeric_code
        }

        /// The currency of the ISO 4217 code `code`.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a code the library does not list, and of the kind `InvalidText`
        /// for text that is not UTF-8.
        pub fn of(code: &DiplomatStr) -> Result<Box<Self>, Box<EranosError>> {
            eranos_bindings::currency_details(&crate::text(code)?)
                .map(|details| Box::new(Self(details)))
                .map_err(crate::failed)
        }
    }

    /// A list of currency details records.
    #[diplomat::opaque]
    #[derive(Debug)]
    pub struct EranosCurrencyDetailsList(pub(crate) Vec<eranos_bindings::CurrencyDetails>);

    impl EranosCurrencyDetailsList {
        /// How many records the list holds.
        #[must_use]
        pub const fn len(&self) -> usize {
            self.0.len()
        }

        /// Whether the list holds no record.
        #[must_use]
        pub const fn is_empty(&self) -> bool {
            self.0.is_empty()
        }

        /// A copy of the record at `index`, absent past the end.
        #[must_use]
        pub fn get(&self, index: usize) -> Option<Box<EranosCurrencyDetails>> {
            self.0
                .get(index)
                .map(|element| Box::new(EranosCurrencyDetails(element.clone())))
        }

        /// Every currency of the ISO 4217 list that the library knows, by code.
        #[must_use]
        pub fn all() -> Box<Self> {
            Box::new(Self(eranos_bindings::currencies()))
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

        /// Writes the percentage of `basis_points` basis points as the library does: "12.5%".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for a percentage out of range.
        pub fn format(
            basis_points: i32,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text = eranos_bindings::percentage_format(basis_points).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
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

        /// Writes the parts of `hundredths` hundredths as the library does: "0.5".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Money` for parts out of range.
        pub fn format(hundredths: i32, write: &mut DiplomatWrite) -> Result<(), Box<EranosError>> {
            let text = eranos_bindings::parts_format(hundredths).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
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

    /// The helpers of phone numbers, which cross in the E.164 form. It holds nothing, and nothing
    /// makes one.
    #[diplomat::opaque]
    #[derive(Debug, Clone, Copy)]
    pub struct EranosPhoneNumber;

    impl EranosPhoneNumber {
        /// Writes the phone number written `text` in the E.164 form: "+40712345678".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Contact` for text that is not a phone number, and of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn parse(
            text: &DiplomatStr,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text =
                eranos_bindings::phone_number_parse(&crate::text(text)?).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// Writes the phone number written `text` in the international form: "+40 712 345 678".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Contact` for text that is not a phone number, and of the
        /// kind `InvalidText` for text that is not UTF-8.
        pub fn international(
            text: &DiplomatStr,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text = eranos_bindings::phone_number_international(&crate::text(text)?)
                .map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }
    }

    /// The helpers of IBANs. It holds nothing, and nothing makes one.
    #[diplomat::opaque]
    #[derive(Debug, Clone, Copy)]
    pub struct EranosIban;

    impl EranosIban {
        /// Writes the IBAN written `text` in its electronic form, without spaces.
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Account` for text that is not an IBAN, and of the kind
        /// `InvalidText` for text that is not UTF-8.
        pub fn parse(
            text: &DiplomatStr,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text = eranos_bindings::iban_parse(&crate::text(text)?).map_err(crate::failed)?;
            crate::write_text(write, &text);
            Ok(())
        }

        /// Writes the country code of the IBAN written `text`: "RO".
        ///
        /// # Errors
        ///
        /// [`EranosError`] of the kind `Account` for text that is not an IBAN, and of the kind
        /// `InvalidText` for text that is not UTF-8.
        pub fn country_code(
            text: &DiplomatStr,
            write: &mut DiplomatWrite,
        ) -> Result<(), Box<EranosError>> {
            let text =
                eranos_bindings::iban_country_code(&crate::text(text)?).map_err(crate::failed)?;
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
        /// [`EranosError`] of the kind `Account` for a blank username, and of the kind
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
            day: date.day,
            month: date.month,
            year: date.year,
        }
    }
}

impl From<eranos_bindings::CalendarDate> for ffi::EranosCalendarDate {
    fn from(date: eranos_bindings::CalendarDate) -> Self {
        Self {
            day: date.day,
            month: date.month,
            year: date.year,
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
