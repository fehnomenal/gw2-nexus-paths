pub fn format_categories(num: usize) -> String {
    format!("{num} categor{}", if num == 1 { "y" } else { "ies" })
}

pub fn format_points(num: usize) -> String {
    format!("{num} point{}", if num == 1 { "" } else { "s" })
}

pub fn format_trails(num: usize) -> String {
    format!("{num} route{}", if num == 1 { "" } else { "s" })
}
