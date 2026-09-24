use crate::model::{Date, Day, Entry, TimeOfDay, TimeSheet};

#[derive(Debug, Clone)]
pub struct ParseError {
    pub line: usize,
    pub message: String,
}

/// Parses a whole time sheet document.
///
/// Errors are collected rather than returned on the first failure, so a
/// caller sees every problem in the file in one pass instead of fixing and
/// re-running one mistake at a time.
pub fn parse(input: &str) -> Result<TimeSheet, Vec<ParseError>> {
    let mut days: Vec<Day> = Vec::new();
    let mut errors: Vec<ParseError> = Vec::new();

    for (idx, raw_line) in input.lines().enumerate() {
        let line_no = idx + 1;
        let line = raw_line.trim_end();
        if line.trim().is_empty() {
            continue;
        }

        if line.trim_start().starts_with('#') {
            match parse_date_header(line, line_no) {
                Ok(date) => days.push(Day { date, entries: Vec::new() }),
                Err(e) => errors.push(e),
            }
            continue;
        }

        match parse_entry(line, line_no) {
            Ok(entry) => match days.last_mut() {
                Some(day) => day.entries.push(entry),
                None => errors.push(ParseError {
                    line: line_no,
                    message: "entry appears before any '# YYYY-MM-DD' date header".to_string(),
                }),
            },
            Err(e) => errors.push(e),
        }
    }

    for day in &days {
        check_overlaps(day, &mut errors);
    }

    if errors.is_empty() {
        Ok(TimeSheet { days })
    } else {
        errors.sort_by_key(|e| e.line);
        Err(errors)
    }
}

fn parse_date_header(line: &str, line_no: usize) -> Result<Date, ParseError> {
    let body = line.trim_start().trim_start_matches('#').trim();
    let parts: Vec<&str> = body.split('-').collect();
    if parts.len() != 3 {
        return Err(ParseError {
            line: line_no,
            message: format!("expected a date header like '# 2026-09-15', found '{}'", line.trim()),
        });
    }

    let year: u16 = parts[0]
        .parse()
        .map_err(|_| ParseError { line: line_no, message: format!("invalid year '{}'", parts[0]) })?;
    let month: u8 = parts[1]
        .parse()
        .map_err(|_| ParseError { line: line_no, message: format!("invalid month '{}'", parts[1]) })?;
    let day: u8 = parts[2]
        .parse()
        .map_err(|_| ParseError { line: line_no, message: format!("invalid day '{}'", parts[2]) })?;

    let date = Date { year, month, day };
    if !date.is_valid() {
        return Err(ParseError { line: line_no, message: format!("'{}' is not a real calendar date", date) });
    }
    Ok(date)
}

fn parse_entry(line: &str, line_no: usize) -> Result<Entry, ParseError> {
    let trimmed = line.trim_start();
    let mut fields = trimmed.splitn(2, char::is_whitespace);
    let time_part = fields.next().unwrap_or("");
    let remainder = fields.next().unwrap_or("").trim_start();

    let mut rest_fields = remainder.splitn(2, char::is_whitespace);
    let project = rest_fields.next().unwrap_or("");
    let description = rest_fields.next().unwrap_or("").trim().to_string();

    if project.is_empty() {
        return Err(ParseError {
            line: line_no,
            message: format!("missing project field: '{}'", line.trim()),
        });
    }

    let (start_str, end_str) = time_part.split_once('-').ok_or_else(|| ParseError {
        line: line_no,
        message: format!("expected a time range 'HH:MM-HH:MM', found '{}'", time_part),
    })?;

    let start = parse_time(start_str, line_no)?;
    let end = parse_time(end_str, line_no)?;

    if end.minutes_since_midnight() <= start.minutes_since_midnight() {
        return Err(ParseError {
            line: line_no,
            message: format!("end time {} is not after start time {}", end, start),
        });
    }

    Ok(Entry { start, end, project: project.to_string(), description, line: line_no })
}

fn parse_time(s: &str, line_no: usize) -> Result<TimeOfDay, ParseError> {
    let (h, m) = s.split_once(':').ok_or_else(|| ParseError {
        line: line_no,
        message: format!("invalid time '{}', expected HH:MM", s),
    })?;
    let hour: u8 = h
        .parse()
        .map_err(|_| ParseError { line: line_no, message: format!("invalid hour in '{}'", s) })?;
    let minute: u8 = m
        .parse()
        .map_err(|_| ParseError { line: line_no, message: format!("invalid minute in '{}'", s) })?;

    let time = TimeOfDay { hour, minute };
    if !time.is_valid() {
        return Err(ParseError { line: line_no, message: format!("'{}' is not a valid time of day", s) });
    }
    Ok(time)
}

/// Flags entries within the same day whose ranges overlap. Original entry
/// order is left untouched; only a scratch index list is sorted.
fn check_overlaps(day: &Day, errors: &mut Vec<ParseError>) {
    let mut order: Vec<usize> = (0..day.entries.len()).collect();
    order.sort_by_key(|&i| day.entries[i].start.minutes_since_midnight());

    for pair in order.windows(2) {
        let earlier = &day.entries[pair[0]];
        let later = &day.entries[pair[1]];
        if earlier.end.minutes_since_midnight() > later.start.minutes_since_midnight() {
            errors.push(ParseError {
                line: later.line,
                message: format!(
                    "{}-{} on {} overlaps with {}-{} at line {}",
                    later.start, later.end, day.date, earlier.start, earlier.end, earlier.line
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn messages(input: &str) -> Vec<String> {
        match parse(input) {
            Ok(_) => Vec::new(),
            Err(errors) => errors.into_iter().map(|e| e.message).collect(),
        }
    }

    #[test]
    fn parses_a_simple_day() {
        let sheet = parse("# 2026-09-15\n09:00-10:00 acme standup\n").unwrap();
        assert_eq!(sheet.days.len(), 1);
        assert_eq!(sheet.days[0].date, Date { year: 2026, month: 9, day: 15 });
        assert_eq!(sheet.days[0].entries.len(), 1);
        assert_eq!(sheet.days[0].entries[0].duration_minutes(), 60);
    }

    #[test]
    fn blank_lines_are_ignored() {
        let sheet = parse("\n# 2026-09-15\n\n09:00-10:00 acme standup\n\n").unwrap();
        assert_eq!(sheet.days.len(), 1);
        assert_eq!(sheet.days[0].entries.len(), 1);
    }

    #[test]
    fn description_is_optional() {
        let sheet = parse("# 2026-09-15\n09:00-10:00 acme\n").unwrap();
        assert_eq!(sheet.days[0].entries[0].description, "");
    }

    #[test]
    fn rejects_malformed_date_header() {
        let errs = messages("# 2026-09\n09:00-10:00 acme standup\n");
        assert_eq!(errs.len(), 1);
        assert!(errs[0].contains("expected a date header"), "{}", errs[0]);
    }

    #[test]
    fn rejects_calendar_date_that_does_not_exist() {
        let errs = messages("# 2026-02-30\n09:00-10:00 acme standup\n");
        assert!(errs[0].contains("not a real calendar date"), "{}", errs[0]);
    }

    #[test]
    fn accepts_leap_day_on_leap_year() {
        assert!(parse("# 2024-02-29\n09:00-10:00 acme standup\n").is_ok());
    }

    #[test]
    fn rejects_leap_day_on_non_leap_year() {
        let errs = messages("# 2025-02-29\n09:00-10:00 acme standup\n");
        assert!(errs[0].contains("not a real calendar date"), "{}", errs[0]);
    }

    #[test]
    fn rejects_month_and_day_out_of_range() {
        let errs = messages("# 2026-13-01\n09:00-10:00 acme standup\n");
        assert!(errs[0].contains("not a real calendar date"), "{}", errs[0]);
    }

    #[test]
    fn rejects_entry_before_any_date_header() {
        let errs = messages("09:00-10:00 acme standup\n");
        assert!(errs[0].contains("before any"), "{}", errs[0]);
    }

    #[test]
    fn rejects_missing_time_range() {
        let errs = messages("# 2026-09-15\nacme standup\n");
        assert!(errs[0].contains("expected a time range"), "{}", errs[0]);
    }

    #[test]
    fn rejects_invalid_hour_and_minute() {
        let errs = messages("# 2026-09-15\n24:00-25:00 acme standup\n");
        assert!(errs[0].contains("not a valid time of day"), "{}", errs[0]);
    }

    #[test]
    fn rejects_end_before_or_equal_to_start() {
        let errs = messages("# 2026-09-15\n10:00-09:00 acme standup\n");
        assert!(errs[0].contains("is not after start time"), "{}", errs[0]);

        let errs = messages("# 2026-09-15\n10:00-10:00 acme standup\n");
        assert!(errs[0].contains("is not after start time"), "{}", errs[0]);
    }

    #[test]
    fn rejects_missing_project_field() {
        let errs = messages("# 2026-09-15\n09:00-10:00\n");
        assert!(errs[0].contains("missing project field"), "{}", errs[0]);
    }

    #[test]
    fn detects_overlap_regardless_of_entry_order() {
        let errs = messages(
            "# 2026-09-15\n13:00-14:00 acme later\n09:00-13:30 acme earlier\n",
        );
        assert_eq!(errs.len(), 1);
        assert!(errs[0].contains("overlaps with"), "{}", errs[0]);
    }

    #[test]
    fn back_to_back_entries_do_not_overlap() {
        assert!(parse("# 2026-09-15\n09:00-10:00 acme a\n10:00-11:00 acme b\n").is_ok());
    }

    #[test]
    fn collects_multiple_errors_in_one_pass() {
        let errs = messages("# 2026-13-01\n25:00-26:00 acme standup\n");
        assert_eq!(errs.len(), 2);
    }

    #[test]
    fn errors_are_sorted_by_line() {
        let input = "# 2026-09-15\n10:00-09:00 acme bad\n# 2026-13-01\n";
        match parse(input) {
            Ok(_) => panic!("expected errors"),
            Err(errors) => {
                let lines: Vec<usize> = errors.iter().map(|e| e.line).collect();
                let mut sorted = lines.clone();
                sorted.sort();
                assert_eq!(lines, sorted);
            }
        }
    }
}
