//! Localization of texts in widgets and containers.
use std::default::Default;

/// Handles the localization of default texts in [`crate::widgets`] and [`crate::containers`].
///
/// You can set the current language with [`crate::Context::set_localization`]. For example: `ctx.set_localization(Language::BahasaMalaysia)`.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "persistence", serde(default))]
pub struct Localization {
    /// The current language used for texts.
    pub lang: Language,

    // Texts for sliders
    pub slider_tooltip: &'static str,

    // Texts for colour pickers
    pub click_copy: &'static str,
    pub cp_edit: &'static str,
    pub cp_blending: &'static str,
    pub cp_additive: &'static str,
    pub cp_normal: &'static str,
    pub cp_selected_color: &'static str,
    pub cp_hue: &'static str,
    pub cp_saturation: &'static str,
    pub cp_value: &'static str,
    pub lang_text: &'static str,
}

impl Default for Localization {
    /// Sets English as the default language for texts.
    ///
    /// It can also be used to switch from another language to English.
    fn default() -> Self {
        Self {
            lang: Language::English,
            slider_tooltip: "Drag to edit or click to enter a value.\nPress 'Shift' while dragging for better control",
            click_copy: "Click to copy",
            cp_edit: "Click to edit color",
            cp_blending: "Blending",
            cp_additive: "Additive",
            cp_normal: "Normal",
            cp_selected_color: "Selected color",
            cp_hue: "Hue",
            cp_saturation: "Saturation",
            cp_value: "Value",
            lang_text: "Language",
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "persistence", derive(serde::Deserialize, serde::Serialize))]
/// Specifies the languages currently available for localization and is required by [`crate::Context::set_localization`] as the parameter type.
pub enum Language {
    English,
    BahasaMalaysia,
}

impl Default for Language {
    fn default() -> Self {
        Language::English
    }
}

impl Localization {
    /// Pattern matches on ['Language'] to call the function that'll set the fields within Localization accordingly.
    pub fn set_localization(&mut self, lang: Language) {
        *self = match lang {
            Language::English => Localization::default(),
            Language::BahasaMalaysia => Localization::malay(),
        };
    }

    /// Returns the current language used for texts.
    pub fn lang(&self) -> Language {
        match self.lang {
            Language::BahasaMalaysia => Language::BahasaMalaysia,
            _ => Language::English,
        }
    }

    /// Sets Bahasa Malaysia as the language for texts.
    fn malay() -> Self {
        Self {
            lang: Language::BahasaMalaysia,
            slider_tooltip: "Tarik untuk ubah atau klik untuk masukkan jumlah.\nTekan 'Shift' sambil tarik untuk pergerakan lebih terkawal.",
            click_copy: "Klik untuk salin",
            cp_edit: "Klik untuk ubah warna",
            cp_blending: "Campuran",
            cp_additive: "Tambahan",
            cp_normal: "Biasa",
            cp_selected_color: "Warna pilihan",
            cp_hue: "Rona",
            cp_saturation: "Ketepuan",
            cp_value: "Nilai",
            lang_text: "Bahasa",
        }
    }
}
