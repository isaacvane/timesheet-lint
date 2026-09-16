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
