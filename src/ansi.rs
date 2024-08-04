macro_rules! ansi {
    ($expr:expr) => {
        concat!("\x1B[", $expr)
    };
}

pub const ERASE_SCREEN: &str = ansi!("2J");
pub const HOME_CURSOR: &str = ansi!("H");
pub const ENABLE_ALT_SCREEN: &str = ansi!("?1049h");
pub const DISABLE_ALT_SCREEN: &str = ansi!("?1049l");
pub const HIDE_CURSOR: &str = ansi!("?25l");
pub const SHOW_CURSOR: &str = ansi!("?25h");

macro_rules! colour {
    ($expr:expr) => {
        concat!(ansi!($expr), "m")
    }
}

pub const GREY_FG: &str = colour!("90");
pub const WHITE_BG: &str = colour!("107");
pub const RED_BG: &str = colour!("41");
pub const RESET: &str = colour!("0");

macro_rules! blue_bg {
    ($expr:expr) => {
        colour!(concat!("48;5;", $expr))
    };
}

// pub const BLUE19_BG: &str = blue_bg!("19");
// pub const BLUE21_BG: &str = blue_bg!("21");
// pub const BLUE27_BG: &str = blue_bg!("27");
// pub const BLUE33_BG: &str = blue_bg!("33");
// pub const BLUE39_BG: &str = blue_bg!("39");
// pub const BLUE69_BG: &str = blue_bg!("69");
// pub const BLUE75_BG: &str = blue_bg!("75");

pub fn blue(val: u8) -> String {
    format!("\x1B[48;2;{0};{0};255m", val)
}
