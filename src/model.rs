use std::fmt;

/// A full time sheet: an ordered list of days, each with its own entries.
#[derive(Debug, Clone)]
pub struct TimeSheet {
    pub days: Vec<Day>,
}

#[derive(Debug, Clone)]
pub struct Day {
    pub date: Date,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub struct Entry {
    pub start: TimeOfDay,
    pub end: TimeOfDay,
    pub project: String,
    pub description: String,
    /// Source line, kept around so validation errors can point back at it.
    pub line: usize,
}

impl Entry {
    pub fn duration_minutes(&self) -> u32 {
        self.end.minutes_since_midnight() - self.start.minutes_since_midnight()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Date {
    pub year: u16,
    pub month: u8,
    pub day: u8,
}

impl Date {
    pub fn is_valid(&self) -> bool {
        self.month >= 1 && self.month <= 12 && self.day >= 1 && self.day as u32 <= days_in_month(self.year, self.month)
    }
}

impl fmt::Display for Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

fn days_in_month(year: u16, month: u8) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeOfDay {
    pub hour: u8,
    pub minute: u8,
}

impl TimeOfDay {
    pub fn is_valid(&self) -> bool {
        self.hour < 24 && self.minute < 60
    }

    pub fn minutes_since_midnight(&self) -> u32 {
        self.hour as u32 * 60 + self.minute as u32
    }
}

impl fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:02}:{:02}", self.hour, self.minute)
    }
}
