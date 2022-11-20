use wgpu::Color;


pub fn color_to_f32_array(color: Color) -> [f32; 4] {
    let Color { r, g, b, a } = color;
    [
        r as f32,
        g as f32,
        b as f32,
        a as f32
    ]
}