use wgpu_glyph::ab_glyph::FontArc;


pub struct Font {
    name: &'static str,
    bytes: &'static [u8]
}

impl Into<FontArc> for Font {
    fn into(self) -> FontArc {
        FontArc::try_from_slice(self.bytes).unwrap()
    } 
}

pub const JETBRAINS_MONO: Font = Font {
    name: "Jetbrains Mono",
    bytes: include_bytes!("../../assets/fonts/JetBrainsMono-Regular.ttf"),
};

pub const JETBRAINS_MONO_BOLD: Font = Font {
    name: "Jetbrains Mono Bold",
    bytes: include_bytes!("../../assets/fonts/JetBrainsMono-Bold.ttf"),
};

pub const JETBRAINS_MONO_NL_EXTRA_BOLD_ITALIC: Font = Font {
    name: "Jetbrains Mono NL Extra Bold Italic",
    bytes: include_bytes!("../../assets/fonts/JetBrainsMonoNL-ExtraBoldItalic.ttf"),
};

pub const JETBRAINS_MONO_LIGHT_ITALIC: Font = Font {
    name: "Jetbrains Mono Light Italic",
    bytes: include_bytes!("../../assets/fonts/JetBrainsMono-LightItalic.ttf"),
};