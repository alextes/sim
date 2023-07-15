use std::time::Duration;

pub fn get_load_indicator_from_duration(duration: Duration) -> char {
    match duration {
        num if num <= Duration::from_millis(10) => '0',
        num if num > Duration::from_millis(10) && num <= Duration::from_millis(20) => '1',
        num if num > Duration::from_millis(20) && num <= Duration::from_millis(30) => '2',
        num if num > Duration::from_millis(30) && num <= Duration::from_millis(40) => '3',
        num if num > Duration::from_millis(40) && num <= Duration::from_millis(50) => '4',
        num if num > Duration::from_millis(50) && num <= Duration::from_millis(60) => '5',
        num if num > Duration::from_millis(60) && num <= Duration::from_millis(70) => '6',
        num if num > Duration::from_millis(70) && num <= Duration::from_millis(80) => '7',
        num if num > Duration::from_millis(80) && num <= Duration::from_millis(90) => '8',
        num if num > Duration::from_millis(90) && num <= Duration::from_millis(100) => '9',
        _ => '?',
    }
}
