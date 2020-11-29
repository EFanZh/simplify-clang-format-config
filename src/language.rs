use std::str::FromStr;

#[derive(Clone, Copy)]
pub enum Language {
    CSharp,
    Cpp,
    Java,
    JavaScript,
    ObjC,
    Proto,
    TableGen,
    TextProto,
}

impl Language {
    pub fn get_name(self) -> &'static str {
        match self {
            Language::CSharp => "CSharp",
            Language::Cpp => "Cpp",
            Language::Java => "Java",
            Language::JavaScript => "JavaScript",
            Language::ObjC => "ObjC",
            Language::Proto => "Proto",
            Language::TableGen => "TableGen",
            Language::TextProto => "TextProto",
        }
    }

    pub fn get_file_extension(self) -> &'static str {
        match self {
            Language::CSharp => ".cs",
            Language::Cpp => ".cpp",
            Language::Java => ".java",
            Language::JavaScript => ".js",
            Language::ObjC => ".m",
            Language::Proto => ".proto",
            Language::TableGen => ".td",
            Language::TextProto => ".textpb",
        }
    }
}

pub struct ParseLanguageError(());

impl FromStr for Language {
    type Err = ParseLanguageError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CSharp" => Ok(Language::CSharp),
            "Cpp" => Ok(Language::Cpp),
            "Java" => Ok(Language::Java),
            "JavaScript" => Ok(Language::JavaScript),
            "ObjC" => Ok(Language::ObjC),
            "Proto" => Ok(Language::Proto),
            "TableGen" => Ok(Language::TableGen),
            "TextProto" => Ok(Language::TextProto),
            _ => Err(ParseLanguageError(())),
        }
    }
}
