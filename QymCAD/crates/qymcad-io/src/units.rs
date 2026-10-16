//! The units a file's numbers can be in, as files name them and as a person picks them.

/// A unit of length a file can be drawn in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileUnit {
    Micron,
    Millimetre,
    Centimetre,
    Metre,
    Inch,
    Foot,
}

impl FileUnit {
    /// Every unit, the metric ones smallest first, then the imperial ones.
    pub const ALL: [FileUnit; 6] = [FileUnit::Micron, FileUnit::Millimetre, FileUnit::Centimetre, FileUnit::Metre, FileUnit::Inch, FileUnit::Foot];

    /// The length of the unit in millimetres.
    pub fn mm(self) -> f64 {
        match self {
            FileUnit::Micron => 0.001,
            FileUnit::Millimetre => 1.0,
            FileUnit::Centimetre => 10.0,
            FileUnit::Metre => 1000.0,
            FileUnit::Inch => 25.4,
            FileUnit::Foot => 304.8,
        }
    }

    /// The unit a 3MF or an AMF names in its `unit` attribute. AMF writes `feet` where 3MF writes `foot`, and only
    /// 3MF knows the centimetre; both words are taken from either file.
    pub fn of_word(word: &str) -> Option<FileUnit> {
        Some(match word {
            "micron" => FileUnit::Micron,
            "millimeter" => FileUnit::Millimetre,
            "centimeter" => FileUnit::Centimetre,
            "meter" => FileUnit::Metre,
            "inch" => FileUnit::Inch,
            "foot" | "feet" => FileUnit::Foot,
            _ => return None,
        })
    }

    /// A stable name for the settings file, never translated.
    pub fn code(self) -> &'static str {
        match self {
            FileUnit::Micron => "micron",
            FileUnit::Millimetre => "mm",
            FileUnit::Centimetre => "cm",
            FileUnit::Metre => "m",
            FileUnit::Inch => "inch",
            FileUnit::Foot => "foot",
        }
    }

    /// The unit whose `code` this is.
    pub fn of_code(code: &str) -> Option<FileUnit> {
        FileUnit::ALL.into_iter().find(|u| u.code() == code)
    }
}
