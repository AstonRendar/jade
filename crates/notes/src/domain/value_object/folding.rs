use unicode_normalization::UnicodeNormalization;

pub(crate) fn fold(text: &str) -> String {
    text.nfc().collect::<String>().to_lowercase()
}
