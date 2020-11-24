use std::convert::From;
use std::fmt::{Display, Formatter, Result};
use time::OffsetDateTime;

#[derive(Debug, Copy, Clone)]
pub enum Month {
    /// [January](https://en.wikipedia.org/wiki/January) is the first month of the year
    January = 1,

    /// [February](https://en.wikipedia.org/wiki/February) is the second month of the year
    February = 2,

    /// [March](https://en.wikipedia.org/wiki/March) is the third month of the year
    March = 3,

    /// [April](https://en.wikipedia.org/wiki/April) is the fourth month of the year
    April = 4,

    /// [May](https://en.wikipedia.org/wiki/May) is the fifth month of the year
    May = 5,

    /// [June](https://en.wikipedia.org/wiki/June) is the sixth month of the year
    June = 6,

    /// [July](https://en.wikipedia.org/wiki/July) is the seventh month of the year
    July = 7,

    /// [August](https://en.wikipedia.org/wiki/August) is the eighth month of the year
    August = 8,

    /// [September](https://en.wikipedia.org/wiki/September) is the ninth month of the year
    September = 9,

    /// [October](https://en.wikipedia.org/wiki/October) is the tenth month of the year
    October = 10,

    /// [November](https://en.wikipedia.org/wiki/November) is the eleventh month of the year
    November = 11,

    /// [December](https://en.wikipedia.org/wiki/December) is the twelfth month of the year
    December = 12,
}

impl Display for Month {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str(match self {
            Month::January => "Jan",
            Month::February => "Feb",
            Month::March => "Mar",
            Month::April => "Apr",
            Month::May => "May",
            Month::June => "Jun",
            Month::July => "Jul",
            Month::August => "Aug",
            Month::September => "Sep",
            Month::October => "Oct",
            Month::November => "Nov",
            Month::December => "Dec",
        })
    }
}

impl From<u8> for Month {
    fn from(item: u8) -> Self {
        match item {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => Month::January,
        }
    }
}

pub(crate) fn rfc1123(date: &OffsetDateTime) -> String {
    let weekday = date.weekday().to_string();
    let month = Month::from(date.month());

    format!(
        "{}, {} {} {} {}:{}:{} GMT",
        &weekday[0..3],
        date.day(),
        month.to_string(),
        date.year(),
        date.hour(),
        date.minute(),
        date.second()
    )
}
