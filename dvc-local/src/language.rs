#[derive(PartialEq, Clone, Copy)]
#[repr(i32)]
pub enum Localization {
	Jap = 0,
	USEng = 1,
	USFrench = 2,
	USSpanish = 3,
	EUEnglish = 4,
	EUFrench = 5,
	EUSpanish = 6,
	EUGerman = 7,
	EUItalian = 8,
	CNTraditional = 9,
	CNSimplified = 10,
	Korean = 11,
}

impl From<i32> for Localization {
	fn from(value: i32) -> Self {
		match value {
			0 => Localization::Jap,
			2 => Localization::USFrench,
			3 => Localization::USSpanish,
			4 => Localization::EUEnglish,
			5 => Localization::EUFrench,
			6 => Localization::EUSpanish,
			7 => Localization::EUGerman,
			8 => Localization::EUItalian,
			9 => Localization::CNTraditional,
			10 => Localization::CNSimplified,
			11 => Localization::Korean,
			_ => Localization::USEng
		}
	}
}
impl Localization {
	pub fn get() -> Localization {
		let v = unsafe { get_lang(None) };
		Localization::from(v)
	}
	pub fn get_lang_code(&self) -> &'static str {
		match self {
			Localization::Jap => "ja",
			Localization::USEng|Self::EUEnglish => "en.txt",
			Localization::USFrench|Localization::EUFrench => "fr",
			Localization::USSpanish|Localization::EUSpanish => "es",
			Localization::EUGerman => "de",
			Localization::EUItalian => "it",
			Localization::CNTraditional => "tw",
			Localization::CNSimplified => "cn",
			Localization::Korean => "kr",
		}
	}
}

#[skyline::from_offset(0x01bdbc80)]
fn get_lang(method_info: unity::prelude::OptionalMethod) -> i32;