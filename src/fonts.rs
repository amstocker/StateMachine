use iced::Font;

pub const JETBRAINS_MONO: Font = Font::External {
    name: "Jetbrains Mono",
    bytes: include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
};

pub const JETBRAINS_MONO_BOLD: Font = Font::External {
    name: "Jetbrains Mono Bold",
    bytes: include_bytes!("../assets/fonts/JetBrainsMono-Bold.ttf"),
};

pub const JETBRAINS_MONO_NL_EXTRA_BOLD_ITALIC: Font = Font::External {
    name: "Jetbrains Mono NL Extra Bold Italic",
    bytes: include_bytes!("../assets/fonts/JetBrainsMonoNL-ExtraBoldItalic.ttf"),
};

pub const JETBRAINS_MONO_LIGHT_ITALIC: Font = Font::External {
    name: "Jetbrains Mono Light Italic",
    bytes: include_bytes!("../assets/fonts/JetBrainsMono-LightItalic.ttf"),
};