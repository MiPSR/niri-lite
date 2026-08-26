use std::iter::zip;

use niri_config::{Gradient, GradientRelativeTo};
use smithay::backend::renderer::element::{Element as _, Kind};
use smithay::utils::{Logical, Point, Rectangle, Size};

use crate::niri_render_elements;
use crate::render_helpers::border::BorderRenderElement;
use crate::render_helpers::renderer::NiriRenderer;
use crate::render_helpers::solid_color::{SolidColorBuffer, SolidColorRenderElement};

#[derive(Debug)]
pub struct FocusRing {
    buffers: [SolidColorBuffer; 8],
    locations: [Point<f64, Logical>; 8],
    sizes: [Size<f64, Logical>; 8],
    borders: [BorderRenderElement; 8],
    full_size: Size<f64, Logical>,
    is_border: bool,
    use_border_shader: bool,
    config: niri_config::FocusRing,
}

niri_render_elements! {
    FocusRingRenderElement => {
        SolidColor = SolidColorRenderElement,
        Gradient = BorderRenderElement,
    }
}

impl FocusRing {
    pub fn new(config: niri_config::FocusRing) -> Self {
        Self {
            buffers: Default::default(),
            locations: Default::default(),
            sizes: Default::default(),
            borders: Default::default(),
            full_size: Default::default(),
            is_border: false,
            use_border_shader: false,
            config,
        }
    }

    pub fn update_config(&mut self, config: niri_config::FocusRing) {
        self.config = config;
    }

    pub fn update_shaders(&mut self) {
        for elem in &mut self.borders {
            elem.damage_all();
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn update_render_elements(
        &mut self,
        win_size: Size<f64, Logical>,
        is_active: bool,
        is_border: bool,
        is_urgent: bool,
        view_rect: Rectangle<f64, Logical>,
        scale: f64,
        alpha: f32,
    ) {
        let width = self.config.width;
        self.full_size = win_size + Size::from((width, width)).upscale(2.);
        self.is_border = is_border;

        let color = if is_urgent {
            self.config.urgent_color
        } else if is_active {
            self.config.active_color
        } else {
            self.config.inactive_color
        };

        for buf in &mut self.buffers {
            buf.set_color(color);
        }

        let gradient = if is_urgent {
            self.config.urgent_gradient
        } else if is_active {
            self.config.active_gradient
        } else {
            self.config.inactive_gradient
        };

        self.use_border_shader = gradient.is_some();

        // Set the defaults for solid color + rounded corners.
        let gradient = gradient.unwrap_or_else(|| Gradient::from(color));

        let full_rect = Rectangle::new(Point::from((-width, -width)), self.full_size);
        let gradient_area = match gradient.relative_to {
            GradientRelativeTo::Window => full_rect,
            GradientRelativeTo::WorkspaceView => view_rect,
        };

        let border_width = if is_border { width as f32 } else { 0. };

        let ceil = |logical: f64| (logical * scale).ceil() / scale;

        // All of this stuff should end up aligned to physical pixels because:
        // * Window size and border width are rounded to physical pixels before being passed to this
        //   function.
        // * We do not divide anything, only add, subtract and multiply by integers.
        // * At rendering time, tile positions are rounded to physical pixels.

        if is_border {
            let corner = ceil(width);

            // Top edge.
            self.sizes[0] = Size::from((win_size.w + width * 2. - corner - corner, width));
            self.locations[0] = Point::from((-width + corner, -width));

            // Bottom edge.
            self.sizes[1] = Size::from((win_size.w + width * 2. - corner - corner, width));
            self.locations[1] = Point::from((-width + corner, win_size.h));

            // Left edge.
            self.sizes[2] = Size::from((width, win_size.h + width * 2. - corner - corner));
            self.locations[2] = Point::from((-width, -width + corner));

            // Right edge.
            self.sizes[3] = Size::from((width, win_size.h + width * 2. - corner - corner));
            self.locations[3] = Point::from((win_size.w, -width + corner));

            // Top-left corner.
            self.sizes[4] = Size::from((corner, corner));
            self.locations[4] = Point::from((-width, -width));

            // Top-right corner.
            self.sizes[5] = Size::from((corner, corner));
            self.locations[5] = Point::from((win_size.w + width - corner, -width));

            // Bottom-right corner.
            self.sizes[6] = Size::from((corner, corner));
            self.locations[6] = Point::from((
                win_size.w + width - corner,
                win_size.h + width - corner,
            ));

            // Bottom-left corner.
            self.sizes[7] = Size::from((corner, corner));
            self.locations[7] = Point::from((-width, win_size.h + width - corner));

            for (buf, size) in zip(&mut self.buffers, self.sizes) {
                buf.resize(size);
            }

            for (border, (loc, size)) in zip(&mut self.borders, zip(self.locations, self.sizes)) {
                border.update(
                    size,
                    Rectangle::new(gradient_area.loc - loc, gradient_area.size),
                    gradient.in_,
                    gradient.from,
                    gradient.to,
                    ((gradient.angle as f32) - 90.).to_radians(),
                    Rectangle::new(full_rect.loc - loc, full_rect.size),
                    border_width,
                    scale as f32,
                    alpha,
                );
            }
        } else {
            self.sizes[0] = self.full_size;
            self.buffers[0].resize(self.sizes[0]);
            self.locations[0] = Point::from((-width, -width));

            self.borders[0].update(
                self.sizes[0],
                Rectangle::new(gradient_area.loc - self.locations[0], gradient_area.size),
                gradient.in_,
                gradient.from,
                gradient.to,
                ((gradient.angle as f32) - 90.).to_radians(),
                Rectangle::new(full_rect.loc - self.locations[0], full_rect.size),
                border_width,
                scale as f32,
                alpha,
            );
        }
    }

    pub fn render(
        &self,
        renderer: &mut impl NiriRenderer,
        location: Point<f64, Logical>,
        push: &mut dyn FnMut(FocusRingRenderElement),
    ) {
        if self.config.off {
            return;
        }

        let border_width = -self.locations[0].y;

        // If drawing as a border with width = 0, then there's nothing to draw.
        if self.is_border && border_width == 0. {
            return;
        }

        let has_border_shader = BorderRenderElement::has_shader(renderer);

        let mut push = |buffer, border: &BorderRenderElement, location: Point<f64, Logical>| {
            let elem = if self.use_border_shader && has_border_shader {
                border.clone().with_location(location).into()
            } else {
                let alpha = border.alpha();
                SolidColorRenderElement::from_buffer(buffer, location, alpha, Kind::Unspecified)
                    .into()
            };
            push(elem);
        };

        if self.is_border {
            for ((buf, border), loc) in zip(zip(&self.buffers, &self.borders), self.locations) {
                push(buf, border, location + loc);
            }
        } else {
            push(
                &self.buffers[0],
                &self.borders[0],
                location + self.locations[0],
            );
        }
    }

    pub fn width(&self) -> f64 {
        self.config.width
    }

    pub fn is_off(&self) -> bool {
        self.config.off
    }

    pub fn config(&self) -> &niri_config::FocusRing {
        &self.config
    }
}
