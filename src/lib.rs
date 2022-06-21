use plotters_backend::text_anchor::{HPos, VPos};
use plotters_backend::{
    BackendColor, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
};

#[derive(Copy, Clone)]
enum PixelState {
    Empty,
    HLine,
    VLine,
    Cross,
    Pixel,
    Filled,
    Text(char),
    Circle(bool),
}

impl PixelState {
    fn to_char(self) -> char {
        match self {
            Self::Empty => ' ',
            Self::HLine => '-',
            Self::VLine => '|',
            Self::Cross => '+',
            Self::Pixel => '.',
            Self::Text(c) => c,
            Self::Circle(filled) => {
                if filled {
                    '@'
                } else {
                    'O'
                }
            },
            Self::Filled => '█',
        }
    }

    fn update(&mut self, new_state: PixelState) {
        let next_state = match (*self, new_state) {
            (Self::HLine, Self::VLine) => Self::Cross,
            (Self::VLine, Self::HLine) => Self::Cross,
            (Self::Filled, _) => Self::Filled,
            (_, Self::Filled) => Self::Filled,
            (_, Self::Circle(what)) => Self::Circle(what),
            (Self::Circle(what), _) => Self::Circle(what),
            (_, Self::Pixel) => Self::Pixel,
            (Self::Pixel, _) => Self::Pixel,
            (_, new) => new,
        };

        *self = next_state;
    }
}

pub struct TextDrawingBackend {
    size: (u32, u32),
    data: Vec<PixelState>
}

impl TextDrawingBackend {
    pub fn new(width: u32, height: u32) -> TextDrawingBackend {
        TextDrawingBackend {
            size: (width, height),
            data: vec![PixelState::Empty; (width * height) as usize],
        }
    }
}

impl DrawingBackend for TextDrawingBackend {
    type ErrorType = std::io::Error;

    fn get_size(&self) -> (u32, u32) {
        self.size
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<std::io::Error>> {
        Ok(())
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<std::io::Error>> {
        for r in 0..self.size.1 {
            let mut buf = String::new();
            for c in 0..self.size.0 {
                buf.push(self.data[(r * self.size.0 + c) as usize].to_char());
            }
            println!("{}", buf);
        }

        Ok(())
    }

    fn draw_pixel(
        &mut self,
        mut pos: (i32, i32),
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<std::io::Error>> {
        pos.0 = pos.0.max(0).min(self.size.0 as i32);
        pos.1 = pos.1.max(0).min(self.size.1 as i32);
        if color.alpha > 0.3 {
            self.data[(pos.1 * self.size.0 as i32 + pos.0) as usize].update(PixelState::Pixel);
        }
        Ok(())
    }
    
    fn draw_rect<S: BackendStyle>(
        &mut self,
        mut upper_left: (i32, i32),
        mut bottom_right: (i32, i32),
        _style: &S,
        _fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        upper_left.0 = upper_left.0.max(0).min(self.size.0 as i32);
        upper_left.1 = upper_left.1.max(0).min(self.size.1 as i32);
        bottom_right.0 = bottom_right.0.max(0).min(self.size.0 as i32 - 1);
        bottom_right.1 = bottom_right.1.max(0).min(self.size.1 as i32 - 1);

        for x in upper_left.0..=bottom_right.0 {
            for y in upper_left.1..=bottom_right.1 {
                self.data[(y * self.size.0 as i32 + x) as usize].update(PixelState::Filled);
            }
        }

        Ok(())
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: (i32, i32),
        to: (i32, i32),
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if from.0 == to.0 {
            let x = from.0;
            let y0 = from.1.min(to.1).max(0).min(self.size.1 as i32);
            let y1 = from.1.max(to.1).max(0).min(self.size.1 as i32);
            for y in y0..y1 {
                self.data[(y * self.size.0 as i32 + x) as usize].update(PixelState::VLine);
            }
            return Ok(());
        }

        if from.1 == to.1 {
            let y = from.1;
            let x0 = from.0.min(to.0).max(0).min(self.size.0 as i32);
            let x1 = from.0.max(to.0).max(0).min(self.size.0 as i32);
            for x in x0..x1 {
                self.data[(y * self.size.0 as i32 + x) as usize].update(PixelState::HLine);
            }
            return Ok(());
        }

        plotters_backend::rasterizer::draw_line(self, from, to, style)
    }

    fn estimate_text_size<S: BackendTextStyle>(
        &self,
        text: &str,
        _: &S,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        Ok((text.len() as u32, 1))
    }

    fn draw_text<S: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &S,
        pos: (i32, i32),
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        let (width, height) = self.estimate_text_size(text, style)?;
        let (width, height) = (width as i32, height as i32);
        let dx = match style.anchor().h_pos {
            HPos::Left => 0,
            HPos::Right => -width,
            HPos::Center => -width / 2,
        };
        let dy = match style.anchor().v_pos {
            VPos::Top => 0,
            VPos::Center => -height / 2,
            VPos::Bottom => -height,
        };
        let offset = (pos.1 + dy).max(0) * self.size.0 as i32 + (pos.0 + dx).max(0);
        for (idx, chr) in (offset..).zip(text.chars()) {
            if (idx as usize) < self.data.len() {
                self.data[idx as usize].update(PixelState::Text(chr));
            }
        }
        Ok(())
    }
}

