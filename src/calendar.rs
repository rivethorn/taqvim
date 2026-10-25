use chrono::{Datelike, NaiveDate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PDate {
    pub year: i32,
    pub month: u32, // 1..12
    pub day: u32,
}

impl PDate {
    pub fn new(y: i32, m: u32, d: u32) -> Self {
        Self { year: y, month: m, day: d }
    }
}

pub const MONTH_NAMES: [&str; 12] = [
    "Farvardin", "Ordibehesht", "Khordad", "Tir", "Mordad", "Shahrivar",
    "Mehr", "Aban", "Azar", "Dey", "Bahman", "Esfand",
];

pub const WEEKDAYS: [&str; 7] = ["Sat", "Sun", "Mon", "Tue", "Wed", "Thu", "Fri"];

pub fn is_leap(jy: i32) -> bool {
    // 33-year cycle
    let r = jy.rem_euclid(33);
    matches!(r, 1 | 5 | 9 | 13 | 17 | 22 | 26 | 30)
}

pub fn month_days(jy: i32, jm: u32) -> u32 {
    if jm <= 6 {
        31
    } else if jm <= 11 {
        30
    } else {
        if is_leap(jy) { 30 } else { 29 }
    }
}

// Gregorian -> Persian using standard algorithm
pub fn gregorian_to_persian(gy: i32, gm: u32, gd: u32) -> PDate {
    let g_d_m = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let gy2 = if gm > 2 { gy + 1 } else { gy };
    let days = 355666
        + 365 * gy
        + (gy2 + 3) / 4
        - (gy2 + 99) / 100
        + (gy2 + 399) / 400
        + gd as i32
        + g_d_m[(gm - 1) as usize];
    let mut jy = -1595 + 33 * (days / 12053);
    let mut rem = days % 12053;
    jy += 4 * (rem / 1461);
    rem %= 1461;
    if rem > 365 {
        jy += (rem - 1) / 365;
        rem = (rem - 1) % 365;
    }
    let (jm, jd) = if rem < 186 {
        (1 + rem / 31, 1 + rem % 31)
    } else {
        let r = rem - 186;
        (7 + r / 30, 1 + r % 30)
    };
    PDate { year: jy, month: jm as u32, day: jd as u32 }
}

pub fn jdn_from_gregorian(y: i32, m: u32, d: u32) -> i64 {
    let y = y as i64;
    let m = m as i64;
    let d = d as i64;
    let a = (14 - m) / 12;
    let yy = y + 4800 - a;
    let mm = m + 12 * a - 3;
    d + (153 * mm + 2) / 5 + 365 * yy + yy / 4 - yy / 100 + yy / 400 - 32045
}

pub fn gregorian_from_jdn(jdn: i64) -> NaiveDate {
    // standard
    let l = jdn + 68569;
    let n = (4 * l) / 146097;
    let l = l - (146097 * n + 3) / 4;
    let i = (4000 * (l + 1)) / 1461001;
    let l = l - (1461 * i) / 4 + 31;
    let j = (80 * l) / 2447;
    let d = l - (2447 * j) / 80;
    let l = j / 11;
    let m = j + 2 - 12 * l;
    let y = 100 * (n - 49) + i + l;
    NaiveDate::from_ymd_opt(y as i32, m as u32, d as u32).unwrap()
}

pub fn persian_to_gregorian(p: PDate) -> NaiveDate {
    // binary search over JDN using gregorian_to_persian
    // Range: JDN for year p.year-1 .. p.year+1
    let target = p;
    // binary search around
    let mut lo = jdn_from_gregorian(p.year + 620, 1, 1) - 400;
    let mut hi = jdn_from_gregorian(p.year + 623, 1, 1) + 400;
    // narrow by iterative
    while lo < hi {
        let mid = (lo + hi) / 2;
        let gd = gregorian_from_jdn(mid);
        let pp = gregorian_to_persian(gd.year(), gd.month(), gd.day());
        if (pp.year, pp.month, pp.day) < (target.year, target.month, target.day) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    gregorian_from_jdn(lo)
}

pub fn today_persian() -> PDate {
    let now = chrono::Local::now().date_naive();
    gregorian_to_persian(now.year(), now.month(), now.day())
}

pub fn weekday_persian(p: PDate) -> usize {
    // Persian weekdays: Saturday=0? We'll map Gregorian weekday.
    let gd = persian_to_gregorian(p);
    // chrono weekday: Mon=0..Sun=6. We want Sat=0.
    let cw = gd.weekday().num_days_from_monday(); // Mon=0
    // Saturday: offset. Monday=0, Sat = -2 mod 7 = 5? Let's compute: Sat -> 5? Mon0 Tue1 Wed2 Thu3 Fri4 Sat5 Sun6
    // We want Sat=0 Sun=1 ... Fri=6. So index = (cw + 2) % 7? cw Sat=5 => (5+2)%7=0. Sun6=>1. Mon0=>2. Yes.
    ((cw + 2) % 7) as usize
}

pub fn format_persian(p: PDate) -> String {
    format!("{} {} {}", p.day, MONTH_NAMES[(p.month - 1) as usize], p.year)
}

pub fn weekday_name(p: PDate) -> &'static str {
    WEEKDAYS[weekday_persian(p)]
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nowruz_2024() {
        let p = gregorian_to_persian(2024, 3, 20);
        assert_eq!((p.year, p.month, p.day), (1403, 1, 1));
    }
    #[test]
    fn roundtrip() {
        let p = PDate::new(1403, 7, 15);
        let g = persian_to_gregorian(p);
        let p2 = gregorian_to_persian(g.year(), g.month(), g.day());
        assert_eq!(p, p2);
    }
    #[test]
    fn weekday() {
        let p = PDate::new(1403, 1, 1);
        assert_eq!(weekday_name(p), "Wed");
    }
}

