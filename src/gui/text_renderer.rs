use sdl2::{
    rect::Rect,
    render::Canvas,
    ttf::{Font, Sdl2TtfContext},
    video::Window,
};

use crate::{configuration::Config, gui::renderer::Renderer};

pub struct TextRenderer<'ttf> {
    font: Font<'ttf, 'static>,
}

impl<'ttf> TextRenderer<'ttf> {
    pub fn new(ttf_context: &'ttf Sdl2TtfContext) -> Self {
        Self {
            font: ttf_context.load_font("assets/DejaVuSans.ttf", 30).unwrap(),
        }
    }

    pub fn draw_multi_line(
        &mut self,
        multi_line_text: &[String],
        x: i32,
        y: i32,
        canvas: &mut Canvas<Window>,
        config: &Config,
    ) {
        // Draw background
        let (width, height) = self.get_size(multi_line_text);
        canvas.set_draw_color(Renderer::to_color(
            &config.renderer.color.text_background_color,
        ));
        let _ = canvas.fill_rect(Rect::new(x, y, width, height));

        // Draw text
        let mut y_offset: u32 = 0;
        for line in multi_line_text.iter() {
            let (_, h) = self.draw(line, x, y + y_offset as i32, canvas, config);
            y_offset += h;
        }
    }

    pub fn get_size(&self, multi_line_text: &[String]) -> (u32, u32) {
        let mut max_width: u32 = 0;
        let mut total_height: u32 = 0;
        for line in multi_line_text.iter() {
            let (width, height) = self.font.size_of(line).unwrap();
            if width > max_width {
                max_width = width;
            }
            total_height += height;
        }
        (max_width, total_height)
    }

    pub fn draw(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        canvas: &mut Canvas<Window>,
        config: &Config,
    ) -> (u32, u32) {
        let surface = self
            .font
            .render(text)
            .blended(Renderer::to_color(&config.renderer.color.text_color))
            .unwrap();
        let texture_creator = canvas.texture_creator();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        let query = texture.query();
        let (width, height) = (query.width, query.height);
        let target = Rect::new(x, y, width, height);
        canvas.copy(&texture, None, Some(target)).unwrap();
        (width, height)
    }
}
