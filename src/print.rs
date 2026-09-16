use crate::model::TimeSheet;
use crate::parser::ParseError;

/// Human-readable rendering: one block per day, entries aligned into
/// columns, a running total per day and a grand total at the end.
pub fn pretty_print(sheet: &TimeSheet) -> String {
    let mut out = String::new();
    let mut grand_total = 0u32;

    let project_width = sheet
        .days
        .iter()
        .flat_map(|d| &d.entries)
        .map(|e| e.project.len())
        .max()
        .unwrap_or(0);

    for day in &sheet.days {
        let day_total: u32 = day.entries.iter().map(|e| e.duration_minutes()).sum();
        grand_total += day_total;

        out.push_str(&format!("{}\n", day.date));
        for entry in &day.entries {
            out.push_str(&format!(
                "  {}-{}  {:width$}  {}\n",
                entry.start,
                entry.end,
                entry.project,
                entry.description,
                width = project_width
            ));
        }
        out.push_str(&format!("  total: {}\n\n", format_duration(day_total)));
    }

    out.push_str(&format!("grand total: {}\n", format_duration(grand_total)));
    out
}

fn format_duration(minutes: u32) -> String {
    format!("{}h{:02}m", minutes / 60, minutes % 60)
}

/// JSON rendering of a successfully parsed sheet. Hand-rolled since this
/// crate has no dependencies to pull in a serializer.
pub fn to_json(sheet: &TimeSheet) -> String {
    let mut out = String::from("{\"days\":[");
    let mut grand_total = 0u32;

    for (i, day) in sheet.days.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let day_total: u32 = day.entries.iter().map(|e| e.duration_minutes()).sum();
        grand_total += day_total;

        out.push_str(&format!(
            "{{\"date\":\"{}\",\"total_minutes\":{},\"entries\":[",
            day.date, day_total
        ));
        for (j, entry) in day.entries.iter().enumerate() {
            if j > 0 {
                out.push(',');
            }
            out.push_str(&format!(
                "{{\"start\":\"{}\",\"end\":\"{}\",\"project\":{},\"description\":{},\"duration_minutes\":{}}}",
                entry.start,
                entry.end,
                json_string(&entry.project),
                json_string(&entry.description),
                entry.duration_minutes()
            ));
        }
        out.push_str("]}");
    }

    out.push_str(&format!("],\"grand_total_minutes\":{}}}", grand_total));
    out
}

/// JSON rendering for the failure path, so a caller in --json mode always
/// gets JSON back, never a plain-text error dump.
pub fn errors_to_json(errors: &[ParseError]) -> String {
    let mut out = String::from("{\"errors\":[");
    for (i, e) in errors.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&format!("{{\"line\":{},\"message\":{}}}", e.line, json_string(&e.message)));
    }
    out.push_str("]}");
    out
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
