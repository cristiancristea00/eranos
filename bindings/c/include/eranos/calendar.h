/// @file eranos/calendar.h
/// Days of the calendar and times of day, which the library checks when they are made.

#ifndef ERANOS_CALENDAR_H
#define ERANOS_CALENDAR_H

#include "EranosCalendarDate.h"
#include "EranosTimeOfDay.h"
#include "eranos/detail.h"
#include "eranos/text.h"

#include <stddef.h>

/// A time of day, to the minute. A plain struct of its hour and minute.
typedef struct EranosTimeOfDay EranosTimeOfDay;
/// A day of the calendar. A plain struct of its day, month, and year.
typedef struct EranosCalendarDate EranosCalendarDate;

/// The date written `text`: "2026-09-23".
///
/// @param text The text, as UTF-8 ended by a NUL; NULL reads as empty text
/// @param calendarDate Receives the calendar date on success
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_CALENDAR for text
///   that is not a date, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
static inline EranosError *eranosCalendarDateParse(char const *const text, EranosCalendarDate *const calendarDate) {
    EranosCalendarDate_parse_result const result = EranosCalendarDate_parse(eranosDetailView(text));
    *calendarDate = result.is_ok ? result.ok : (EranosCalendarDate){0};
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Writes the date as the library does: "2026-09-23".
///
/// @param calendarDate The calendar date
/// @param text Receives the text, replacing what it held
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_CALENDAR for a day
///   the calendar does not have
static inline EranosError *eranosCalendarDateFormat(EranosCalendarDate const calendarDate, EranosText *const text) {
    eranosDetailClear(text);
    EranosCalendarDate_format_result const result = EranosCalendarDate_format(calendarDate, text);
    eranosDetailFinish(text);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// The time written `text`: "18:30".
///
/// @param text The text, as UTF-8 ended by a NUL; NULL reads as empty text
/// @param timeOfDay Receives the time of day on success
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_CALENDAR for text
///   that is not a time, and of the kind ERANOS_ERROR_KIND_INVALID_TEXT for text that is not UTF-8
static inline EranosError *eranosTimeOfDayParse(char const *const text, EranosTimeOfDay *const timeOfDay) {
    EranosTimeOfDay_parse_result const result = EranosTimeOfDay_parse(eranosDetailView(text));
    *timeOfDay = result.is_ok ? result.ok : (EranosTimeOfDay){0};
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

/// Writes the time as the library does: "18:30".
///
/// @param timeOfDay The time of day
/// @param text Receives the text, replacing what it held
/// @return NULL on success, or the error, which the caller destroys with eranosErrorDestroy: one of the kind ERANOS_ERROR_KIND_CALENDAR for a time
///   the day does not have
static inline EranosError *eranosTimeOfDayFormat(EranosTimeOfDay const timeOfDay, EranosText *const text) {
    eranosDetailClear(text);
    EranosTimeOfDay_format_result const result = EranosTimeOfDay_format(timeOfDay, text);
    eranosDetailFinish(text);
    return eranosDetailErrorOf(result.is_ok, &result.err);
}

#endif // ERANOS_CALENDAR_H
