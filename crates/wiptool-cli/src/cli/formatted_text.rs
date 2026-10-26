use std::fmt::Display;

#[allow(dead_code)]
#[derive(Debug, Default, Clone, Copy)]
#[repr(u8)]
pub enum ANSIColor {
    BLACK = 0,
    RED = 1,
    GREEN = 2,
    YELLOW = 3,
    BLUE = 4,
    PURPLE = 5,
    CYAN = 6,
    #[default]
    WHITE = 7,
}

impl ANSIColor {
    fn code(&self) -> u8 {
        return self.clone() as u8;
    }
}

#[derive(Debug)]
pub enum Color {
    ANSI(ANSIColor),
    #[allow(dead_code)]
    TrueColor(u8, u8, u8),
}

impl Color {
    fn foreground_seq(&self) -> String {
        return match self {
            Color::ANSI(ansi) => format!("3{}", ansi.code()),
            _ => todo!(),
        };
    }
}

impl From<ANSIColor> for Color {
    fn from(value: ANSIColor) -> Self {
        return Self::ANSI(value);
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(value: (u8, u8, u8)) -> Self {
        return Self::TrueColor(value.0, value.1, value.2);
    }
}

#[derive(Debug, Default)]
pub struct FormattedString {
    string: String,
    color: Option<Color>,
    bold: bool,
    italics: bool,
}

impl FormattedString {
    pub fn new<S: ToString>(string: S) -> Self {
        FormattedString {
            string: string.to_string(),
            color: None,
            bold: false,
            italics: false,
        }
    }

    pub fn color<I: Into<Color>>(mut self, color: I) -> Self {
        self.color = Some(color.into());
        return self;
    }

    pub fn bold(mut self) -> Self {
        self.bold = true;
        return self;
    }

    #[allow(dead_code)]
    pub fn italics(mut self) -> Self {
        self.italics = true;
        return self;
    }
}

impl Display for FormattedString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted = self.color.is_some() || self.bold || self.italics;
        if formatted {
            write!(f, "\x1b[")?;
        }
        if self.bold {
            write!(f, "1")?;
            if self.italics || self.color.is_some() {
                write!(f, ";")?;
            }
        }
        if self.italics {
            write!(f, "3")?;
            if self.color.is_some() {
                write!(f, ";")?;
            }
        }
        if let Some(color) = &self.color {
            write!(f, "{}", color.foreground_seq())?;
        }
        if formatted {
            write!(f, "m")?;
        }

        write!(f, "{}", self.string)?;

        if formatted {
            write!(f, "\x1b[0m")?;
        }

        return Ok(());
    }
}

#[derive(Debug, Default)]
pub struct FormattedText {
    s: Vec<FormattedString>,
}

impl FormattedText {
    pub fn new() -> Self {
        Self { s: Vec::new() }
    }

    pub fn push_fmtstr(&mut self, s: FormattedString) {
        self.s.push(s);
    }

    pub fn push_str<I: ToString>(&mut self, s: I) {
        self.s.push(FormattedString::new(s));
    }

    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        let mut len = 0;
        for segment in &self.s {
            len += segment.string.len();
        }

        return len;
    }
}

impl Display for FormattedText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for segment in &self.s {
            write!(f, "{segment}")?;
        }
        return Ok(());
    }
}
