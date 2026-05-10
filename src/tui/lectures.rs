/* -----------------------------------------------------------------------------
 * tui/lectures.rs
 * Week view calendar for lectures with navigation.
 * -------------------------------------------------------------------------- */

use std::collections::BTreeMap;

use super::Screen;
use crate::{data, display::color, json, terminal, util};

type Lecture = json::Value;

struct DayLectures {
    date: String,
    lectures: Vec<Lecture>,
}

/* --- Render Loop ---------------------------------------------------------- */

pub fn render() -> Screen {
    let mut week_offset: i64 = 0;
    let mut scroll_offset = 0;
    let mut cached_week: Option<i64> = None;
    let mut cached_days: Vec<DayLectures> = Vec::new();
    let mut current_week_label = String::new();

    loop {
        let (from_date, to_date) = util::get_week_bounds(week_offset);

        if Some(week_offset) != cached_week {
            let all_lectures = match data::fetch_lectures(
                Some(from_date.as_str()),
                Some(to_date.as_str()),
                None,
            ) {
                Ok(c) => c,
                Err(e) => {
                    println!("{} {}.", color::red("ERR"), e);
                    println!("{} Press any key.", color::dim("HINT"));
                    terminal::getch();
                    return Screen::Menu;
                }
            };

            cached_days = group_lectures_by_day(&all_lectures, week_offset);
            cached_week = Some(week_offset);
            current_week_label = util::format_week_label(week_offset);
        }

        terminal::clear();
        println!("{}", color::bold(&format!("  Lectures - {}", current_week_label)));
        println!("  {}\n", color::dim(&"-".repeat(40)));

        let height = terminal::get_height().max(6);
        let max_visible = height - 6;

        let total_slots: usize = cached_days.iter().map(|d| d.lectures.len()).sum();

        if total_slots == 0 {
            println!("  No lectures this week.");
            println!("\n  {} Press any key.", color::dim("HINT"));
            terminal::getch();
            return Screen::Menu;
        }

        if scroll_offset >= total_slots {
            scroll_offset = total_slots.saturating_sub(1);
        }

        render_week_grid(&cached_days, scroll_offset, max_visible, total_slots);

        println!("\n  {}", color::dim("← →: week nav | g: jump to week | Enter: details | Esc: back"));

        match terminal::getch().as_str() {
            "\x1b[A" => {
                if scroll_offset > 0 {
                    scroll_offset -= 1;
                }
            }
            "\x1b[B" => {
                if scroll_offset + 1 < total_slots {
                    scroll_offset += 1;
                }
            }
            "\x1b[D" => {
                week_offset -= 1;
                cached_week = None;
                scroll_offset = 0;
            }
            "\x1b[C" => {
                week_offset += 1;
                cached_week = None;
                scroll_offset = 0;
            }
            "g" => {
                print!("  {} ", color::dim("Weeks back (positive number):"));
                let _ = std::io::Write::flush(&mut std::io::stdout()); // best-effort: prompt
                let mut input = String::new();
                if std::io::stdin().read_line(&mut input).is_ok() {
                    if let Ok(n) = input.trim().parse::<i64>() {
                        week_offset = -n;
                        cached_week = None;
                        scroll_offset = 0;
                    }
                }
            }
            "\r" | "\n" => {
                show_lecture_details(&cached_days, scroll_offset);
            }
            "\x1b" => return Screen::Menu,
            "q" => return Screen::Quit,
            _ => {}
        }
    }
}

/* --- Grouping ------------------------------------------------------------- */

fn group_lectures_by_day(lectures: &[Lecture], week_offset: i64) -> Vec<DayLectures> {
    let mut day_map: BTreeMap<String, Vec<Lecture>> = BTreeMap::new();

    for lecture_val in lectures {
        let starts = lecture_val["startsAt"].as_str().unwrap_or("");
        let date = starts.split('T').next().unwrap_or("").to_string();
        if date.is_empty() {
            continue;
        }
        day_map.entry(date.clone()).or_default().push(lecture_val.clone());
    }

    for (_, lecs) in day_map.iter_mut() {
        lecs.sort_by(|a, b| {
            let a_start = a["startsAt"].as_str().unwrap_or("");
            let b_start = b["startsAt"].as_str().unwrap_or("");
            a_start.cmp(b_start)
        });
    }

    let (week_start, _) = util::get_week_bounds(week_offset);
    let start_parts: Vec<&str> = week_start.split('-').collect();
    let start_year: i64 = start_parts.get(0).unwrap_or(&"2025").parse().unwrap_or(2025);
    let start_month: u32 = start_parts.get(1).unwrap_or(&"01").parse().unwrap_or(1);
    let start_day: u32 = start_parts.last().unwrap_or(&"1").parse().unwrap_or(1);

    let mut result = Vec::new();
    for day_num in 0..5 {
        let day_of_month = start_day + day_num as u32;
        let date = format!("{:04}-{:02}-{:02}", start_year, start_month, day_of_month);
        let lectures = day_map.remove(&date).unwrap_or_default();
        result.push(DayLectures { date, lectures });
    }

    result
}

/* --- Grid Rendering ------------------------------------------------------- */

fn render_week_grid(days: &[DayLectures], scroll_offset: usize, max_visible: usize, total_slots: usize) {
    let mut all_slots: Vec<(usize, &Lecture)> = Vec::new();
    for (day_idx, day) in days.iter().enumerate() {
        for lecture_val in &day.lectures {
            all_slots.push((day_idx, lecture_val));
        }
    }

    let needs_scroll = total_slots > max_visible;
    let visible_start = if needs_scroll { scroll_offset } else { 0 };
    let visible_end = if needs_scroll {
        (scroll_offset + max_visible).min(all_slots.len())
    } else {
        all_slots.len()
    };

    if needs_scroll && scroll_offset > 0 {
        println!("  {} {}\n", color::dim("^"), color::dim(&format!("{} more", scroll_offset)));
    }

    let mut current_day = -1;

    for (idx, (day_idx, lecture_val)) in all_slots[visible_start..visible_end].iter().enumerate() {
        if *day_idx as i32 != current_day {
            current_day = *day_idx as i32;
            if idx > 0 {
                println!();
            }
            let date = &days[*day_idx].date;
            println!("  {}", color::bold(date));
        }

        let starts = lecture_val["startsAt"].as_str().unwrap_or("");
        let ends = lecture_val["endsAt"].as_str().unwrap_or("");
        let start_time = starts.split('T').nth(1).unwrap_or("").split(':').take(2).collect::<Vec<_>>().join(":");
        let end_time = ends.split('T').nth(1).unwrap_or("").split(':').take(2).collect::<Vec<_>>().join(":");
        let course = lecture_val["courseName"].as_str().unwrap_or("-");
        let room = lecture_val["place"]["name"].as_str().unwrap_or("-");

        let is_selected = (visible_start + idx) == scroll_offset;
        let prefix = if is_selected { ">" } else { " " };
        let line = format!("    {} {}  {}  {}", prefix, color::dim(&format!("{}-{}", start_time, end_time)), color::cyan(&util::trunc(course, 20)), color::dim(&util::trunc(room, 8)));

        if is_selected {
            println!("{}", color::cyan(&line));
        } else {
            println!("{}", line);
        }
    }

    if needs_scroll && visible_end < total_slots {
        let remaining = total_slots - visible_end;
        println!("\n  {} {}", color::dim("v"), color::dim(&format!("{} more", remaining)));
    }
}

/* --- Details View --------------------------------------------------------- */

fn show_lecture_details(days: &[DayLectures], slot_index: usize) {
    let mut all_lectures: Vec<&Lecture> = Vec::new();
    for day in days {
        for lecture_val in &day.lectures {
            all_lectures.push(lecture_val);
        }
    }

    if slot_index >= all_lectures.len() {
        return;
    }

    let lecture_val = all_lectures[slot_index];

    terminal::clear();
    println!("{}", color::bold("  Lecture Details"));
    println!("  {}\n", color::dim("==============="));

    let starts = lecture_val["startsAt"].as_str().unwrap_or("");
    let ends = lecture_val["endsAt"].as_str().unwrap_or("");
    let date = starts.split('T').next().unwrap_or("-");
    let start_time = starts.split('T').nth(1).unwrap_or("").split(':').take(2).collect::<Vec<_>>().join(":");
    let end_time = ends.split('T').nth(1).unwrap_or("").split(':').take(2).collect::<Vec<_>>().join(":");

    println!("  {}: {}", color::bold("Course"), color::cyan(lecture_val["courseName"].as_str().unwrap_or("-")));
    println!("  {}: {} {}-{}", color::bold("Date"), color::dim(date), color::dim(&start_time), color::dim(&end_time));
    println!("  {}: {}", color::bold("Room"), color::dim(lecture_val["place"]["name"].as_str().unwrap_or("-")));

    let teacher_id = lecture_val["teacherId"].as_i64().unwrap_or(0);
    if teacher_id > 0 {
        println!("  {}: {}", color::bold("Teacher ID"), color::dim(&teacher_id.to_string()));
    }

    let lec_type = lecture_val["type"].as_str().unwrap_or("-");
    if !lec_type.is_empty() {
        println!("  {}: {}", color::bold("Type"), color::dim(lec_type));
    }

    let description_text = lecture_val["description"].as_str().unwrap_or("");
    if !description_text.is_empty() {
        println!("\n  {}:", color::bold("Description"));
        for line in util::trunc(description_text, 200).lines() {
            println!("    {}", line);
        }
    }

    let virtual_classrooms = lecture_val["virtualClassrooms"].as_array().unwrap_or(&[]);
    if !virtual_classrooms.is_empty() {
        println!("\n  {}:", color::bold("Virtual Classrooms"));
        for vc in virtual_classrooms {
            let name = vc["name"].as_str().unwrap_or("-");
            let url = vc["url"].as_str().unwrap_or("-");
            println!("    - {} ({})", color::cyan(name), color::dim(url));
        }
    }

    println!("\n  {} Press any key.", color::dim("HINT"));
    terminal::getch();
}