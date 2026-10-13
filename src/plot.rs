//! A tiny plotting library that writes SVG files. Given code.
//!
//! Open the `.svg` files it writes in any web browser. You don't need to
//! read this file to do the course, but it uses nothing beyond what the
//! lessons teach, so it's good reading once you're past lesson 07. It even
//! writes PNG images by hand (see `png_bytes`) so spectrograms stay sharp.
//!
//! ```no_run
//! use lab::plot::Plot;
//! let xs = vec![0.0, 1.0, 2.0, 3.0];
//! let ys = vec![0.0, 1.0, 4.0, 9.0];
//! Plot::new("Squares")
//!     .x_label("x")
//!     .y_label("x squared")
//!     .line("x²", &xs, &ys)
//!     .save("out/squares.svg")
//!     .unwrap();
//! ```

use std::fmt::Write as _;
use std::io;
use std::path::Path;

// Colours: one fixed order for series, quiet greys for everything else.
const SURFACE: &str = "#fcfcfb";
const INK: &str = "#0b0b0b";
const INK_2: &str = "#52514e";
const MUTED: &str = "#898781";
const GRID: &str = "#e1e0d9";
const AXIS: &str = "#c3c2b7";
const SERIES: [&str; 8] = [
    "#2a78d6", "#eb6834", "#1baf7a", "#eda100", "#e87ba4", "#008300", "#4a3aa7", "#e34948",
];
/// Sequential ramp for heatmaps: quiet values fade into the background,
/// loud values are dark blue.
const RAMP: [(u8, u8, u8); 8] = [
    (0xfc, 0xfc, 0xfb),
    (0xcd, 0xe2, 0xfb),
    (0x9e, 0xc5, 0xf4),
    (0x6d, 0xa7, 0xec),
    (0x39, 0x87, 0xe5),
    (0x25, 0x6a, 0xbf),
    (0x18, 0x4f, 0x95),
    (0x0d, 0x36, 0x6b),
];
const FONT: &str = "system-ui, -apple-system, 'Segoe UI', sans-serif";

const WIDTH: f64 = 820.0;
const HEIGHT: f64 = 440.0;
const LEFT: f64 = 70.0;
const TOP: f64 = 62.0;
const BOTTOM: f64 = 56.0;

#[derive(Clone, Copy, PartialEq)]
enum Mark {
    Line,
    Points,
}

struct Series {
    label: Option<String>,
    xs: Vec<f64>,
    ys: Vec<f64>,
    mark: Mark,
    color: usize,
}

/// A line/scatter plot, built step by step and then saved.
pub struct Plot {
    title: String,
    x_label: String,
    y_label: String,
    x_range: Option<(f64, f64)>,
    y_range: Option<(f64, f64)>,
    series: Vec<Series>,
    next_color: usize,
}

impl Plot {
    pub fn new(title: &str) -> Self {
        Plot {
            title: title.to_string(),
            x_label: String::new(),
            y_label: String::new(),
            x_range: None,
            y_range: None,
            series: Vec::new(),
            next_color: 0,
        }
    }

    pub fn x_label(mut self, label: &str) -> Self {
        self.x_label = label.to_string();
        self
    }

    pub fn y_label(mut self, label: &str) -> Self {
        self.y_label = label.to_string();
        self
    }

    /// Fix the x axis instead of fitting it to the data.
    pub fn x_range(mut self, lo: f64, hi: f64) -> Self {
        self.x_range = Some((lo, hi));
        self
    }

    /// Fix the y axis instead of fitting it to the data.
    pub fn y_range(mut self, lo: f64, hi: f64) -> Self {
        self.y_range = Some((lo, hi));
        self
    }

    /// Add a line. Each labelled series gets the next colour. NaN values
    /// leave a gap in the line (handy for "no pitch here").
    pub fn line(self, label: &str, xs: &[f64], ys: &[f64]) -> Self {
        self.add(Some(label), xs, ys, Mark::Line)
    }

    /// Add dots, one per value (good for showing individual samples).
    pub fn points(self, label: &str, xs: &[f64], ys: &[f64]) -> Self {
        self.add(Some(label), xs, ys, Mark::Points)
    }

    /// Add another line in the same colour as the previous series, with no
    /// legend entry. Use it to draw a family of curves, like a filterbank.
    pub fn line_same(self, xs: &[f64], ys: &[f64]) -> Self {
        self.add(None, xs, ys, Mark::Line)
    }

    fn add(mut self, label: Option<&str>, xs: &[f64], ys: &[f64], mark: Mark) -> Self {
        assert_eq!(xs.len(), ys.len(), "plot: xs and ys must have the same length");
        let color = match (label, self.series.last()) {
            (None, Some(prev)) => prev.color,
            _ => {
                self.next_color += 1;
                self.next_color - 1
            }
        };
        assert!(
            color < SERIES.len(),
            "plot: more than {} coloured series; split into several plots or use line_same",
            SERIES.len()
        );
        self.series.push(Series {
            label: label.map(|s| s.to_string()),
            xs: xs.to_vec(),
            ys: ys.to_vec(),
            mark,
            color,
        });
        self
    }

    /// Write the plot to an SVG file, creating parent folders if needed.
    pub fn save(&self, path: impl AsRef<Path>) -> io::Result<()> {
        write_file(path.as_ref(), &self.to_svg())
    }

    pub fn to_svg(&self) -> String {
        let right = 24.0;
        let (pw, ph) = (WIDTH - LEFT - right, HEIGHT - TOP - BOTTOM);
        let (x0, x1) = self
            .x_range
            .unwrap_or_else(|| data_range(self.series.iter().flat_map(|s| s.xs.iter().copied())));
        let (y0, y1) = self.y_range.unwrap_or_else(|| {
            let (lo, hi) = data_range(self.series.iter().flat_map(|s| s.ys.iter().copied()));
            let pad = (hi - lo) * 0.06;
            (lo - pad, hi + pad)
        });
        let px = |x: f64| LEFT + (x - x0) / (x1 - x0) * pw;
        let py = |y: f64| TOP + ph - (y - y0) / (y1 - y0) * ph;

        let mut svg = svg_open();
        self.write_header(&mut svg);
        write_axes(&mut svg, (x0, x1), (y0, y1), pw, ph, &self.x_label, &self.y_label);

        let _ = write!(
            svg,
            r#"<clipPath id="area"><rect x="{LEFT}" y="{TOP}" width="{pw}" height="{ph}"/></clipPath><g clip-path="url(#area)">"#
        );
        for s in &self.series {
            let color = SERIES[s.color];
            match s.mark {
                Mark::Line => {
                    for segment in finite_segments(&s.xs, &s.ys) {
                        let pts: Vec<(f64, f64)> =
                            segment.iter().map(|&(x, y)| (px(x), py(y))).collect();
                        let pts = decimate(&pts);
                        let mut attr = String::new();
                        for (x, y) in pts {
                            let _ = write!(attr, "{x:.1},{y:.1} ");
                        }
                        let _ = write!(
                            svg,
                            r#"<polyline points="{attr}" fill="none" stroke="{color}" stroke-width="2" stroke-linejoin="round" stroke-linecap="round"/>"#
                        );
                    }
                }
                Mark::Points => {
                    for (&x, &y) in s.xs.iter().zip(&s.ys) {
                        if x.is_finite() && y.is_finite() {
                            let _ = write!(
                                svg,
                                r#"<circle cx="{:.1}" cy="{:.1}" r="4" fill="{color}" stroke="{SURFACE}" stroke-width="2"/>"#,
                                px(x),
                                py(y)
                            );
                        }
                    }
                }
            }
        }
        svg.push_str("</g></svg>\n");
        svg
    }

    fn write_header(&self, svg: &mut String) {
        let _ = write!(
            svg,
            r#"<text x="{LEFT}" y="26" font-size="16" font-weight="600" fill="{INK}">{}</text>"#,
            escape(&self.title)
        );
        let labelled: Vec<&Series> = self.series.iter().filter(|s| s.label.is_some()).collect();
        if labelled.len() < 2 {
            return; // a single series is named by the title
        }
        let mut x = LEFT;
        for s in labelled {
            let color = SERIES[s.color];
            let label = escape(s.label.as_deref().unwrap_or(""));
            match s.mark {
                Mark::Line => {
                    let _ = write!(
                        svg,
                        r#"<line x1="{x}" y1="44" x2="{}" y2="44" stroke="{color}" stroke-width="2" stroke-linecap="round"/>"#,
                        x + 16.0
                    );
                }
                Mark::Points => {
                    let _ = write!(
                        svg,
                        r#"<circle cx="{}" cy="44" r="4" fill="{color}"/>"#,
                        x + 8.0
                    );
                }
            }
            let _ = write!(
                svg,
                r#"<text x="{}" y="48" font-size="12" fill="{INK_2}">{label}</text>"#,
                x + 22.0
            );
            x += 22.0 + 7.0 * label.chars().count() as f64 + 18.0;
        }
    }
}

/// A heatmap (for spectrograms). `data[i][j]`: `i` runs along the x axis
/// (time frames), `j` up the y axis (frequency bins), with `j = 0` at the
/// bottom.
pub struct Heatmap {
    title: String,
    x_label: String,
    y_label: String,
    value_label: String,
    x_extent: Option<(f64, f64)>,
    y_extent: Option<(f64, f64)>,
    value_range: Option<(f64, f64)>,
}

impl Heatmap {
    pub fn new(title: &str) -> Self {
        Heatmap {
            title: title.to_string(),
            x_label: String::new(),
            y_label: String::new(),
            value_label: String::new(),
            x_extent: None,
            y_extent: None,
            value_range: None,
        }
    }

    pub fn x_label(mut self, label: &str) -> Self {
        self.x_label = label.to_string();
        self
    }

    pub fn y_label(mut self, label: &str) -> Self {
        self.y_label = label.to_string();
        self
    }

    /// Label for the colour bar, e.g. "dB".
    pub fn value_label(mut self, label: &str) -> Self {
        self.value_label = label.to_string();
        self
    }

    /// The x values at the left and right edges (e.g. 0.0 and the duration).
    pub fn x_extent(mut self, lo: f64, hi: f64) -> Self {
        self.x_extent = Some((lo, hi));
        self
    }

    /// The y values at the bottom and top edges (e.g. 0.0 and Nyquist).
    pub fn y_extent(mut self, lo: f64, hi: f64) -> Self {
        self.y_extent = Some((lo, hi));
        self
    }

    /// Clamp colours to this range instead of the data's min and max.
    pub fn value_range(mut self, lo: f64, hi: f64) -> Self {
        self.value_range = Some((lo, hi));
        self
    }

    pub fn save(&self, path: impl AsRef<Path>, data: &[Vec<f64>]) -> io::Result<()> {
        write_file(path.as_ref(), &self.to_svg(data))
    }

    pub fn to_svg(&self, data: &[Vec<f64>]) -> String {
        let right = 96.0;
        let (pw, ph) = (WIDTH - LEFT - right, HEIGHT - TOP - BOTTOM);
        let nx = data.len().max(1);
        let ny = data.first().map_or(1, |c| c.len().max(1));
        let (x0, x1) = self.x_extent.unwrap_or((0.0, nx as f64));
        let (y0, y1) = self.y_extent.unwrap_or((0.0, ny as f64));
        let (v0, v1) = self
            .value_range
            .unwrap_or_else(|| data_range(data.iter().flatten().copied()));

        // Shrink big matrices by averaging blocks, so the file stays small.
        let cols = nx.min(900);
        let rows = ny.min(420);
        let mut rgb = Vec::with_capacity(cols * rows * 3);
        for r in (0..rows).rev() {
            let (j0, j1) = (r * ny / rows, ((r + 1) * ny / rows).max(r * ny / rows + 1));
            for c in 0..cols {
                let (i0, i1) = (c * nx / cols, ((c + 1) * nx / cols).max(c * nx / cols + 1));
                let (mut sum, mut count) = (0.0, 0);
                for column in &data[i0..i1.min(data.len())] {
                    for &v in &column[j0..j1.min(column.len())] {
                        if v.is_finite() {
                            sum += v;
                            count += 1;
                        }
                    }
                }
                let v = if count > 0 { sum / count as f64 } else { v0 };
                let (cr, cg, cb) = ramp_color((v - v0) / (v1 - v0));
                rgb.extend_from_slice(&[cr, cg, cb]);
            }
        }
        let png = base64(&png_bytes(cols as u32, rows as u32, &rgb));

        let mut svg = svg_open();
        let _ = write!(
            svg,
            r#"<text x="{LEFT}" y="26" font-size="16" font-weight="600" fill="{INK}">{}</text>"#,
            escape(&self.title)
        );
        let _ = write!(
            svg,
            r#"<image x="{LEFT}" y="{TOP}" width="{pw}" height="{ph}" preserveAspectRatio="none" style="image-rendering:pixelated" href="data:image/png;base64,{png}"/>"#
        );
        write_axes_with(&mut svg, (x0, x1), (y0, y1), pw, ph, &self.x_label, &self.y_label, false);

        // Colour bar
        let bx = LEFT + pw + 24.0;
        let _ = write!(svg, r#"<defs><linearGradient id="ramp" x1="0" y1="1" x2="0" y2="0">"#);
        for (k, (r, g, b)) in RAMP.iter().enumerate() {
            let _ = write!(
                svg,
                r##"<stop offset="{:.3}" stop-color="#{r:02x}{g:02x}{b:02x}"/>"##,
                k as f64 / (RAMP.len() - 1) as f64
            );
        }
        let _ = write!(
            svg,
            r#"</linearGradient></defs><rect x="{bx}" y="{TOP}" width="14" height="{ph}" fill="url(#ramp)" stroke="{AXIS}" stroke-width="1"/>"#
        );
        for (v, y) in [(v1, TOP + 4.0), (v0, TOP + ph)] {
            let _ = write!(
                svg,
                r#"<text x="{}" y="{y}" font-size="11" fill="{MUTED}" style="font-variant-numeric:tabular-nums">{}</text>"#,
                bx + 20.0,
                fmt_num(v, (v1 - v0).abs() / 10.0)
            );
        }
        let _ = write!(
            svg,
            r#"<text x="{}" y="{}" font-size="12" fill="{INK_2}">{}</text>"#,
            bx + 20.0,
            TOP + ph / 2.0,
            escape(&self.value_label)
        );
        svg.push_str("</svg>\n");
        svg
    }
}

// ---------------------------------------------------------------------------
// Shared drawing helpers
// ---------------------------------------------------------------------------

fn svg_open() -> String {
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" font-family="{FONT}"><rect width="100%" height="100%" fill="{SURFACE}"/>"#
    )
}

fn write_axes(
    svg: &mut String,
    (x0, x1): (f64, f64),
    (y0, y1): (f64, f64),
    pw: f64,
    ph: f64,
    x_label: &str,
    y_label: &str,
) {
    write_axes_with(svg, (x0, x1), (y0, y1), pw, ph, x_label, y_label, true);
}

/// `grid`: full-width horizontal gridlines (line plots), or just short tick
/// marks beside the axis (heatmaps, where lines would cover the data).
#[allow(clippy::too_many_arguments)]
fn write_axes_with(
    svg: &mut String,
    (x0, x1): (f64, f64),
    (y0, y1): (f64, f64),
    pw: f64,
    ph: f64,
    x_label: &str,
    y_label: &str,
    grid: bool,
) {
    let px = |x: f64| LEFT + (x - x0) / (x1 - x0) * pw;
    let py = |y: f64| TOP + ph - (y - y0) / (y1 - y0) * ph;
    let tick_style = format!(r#"font-size="11" fill="{MUTED}" style="font-variant-numeric:tabular-nums""#);

    let (yt, ystep) = nice_ticks(y0, y1, 6);
    for &y in &yt {
        let yy = py(y);
        let (line_start, line_end, color) = if grid { (LEFT, LEFT + pw, GRID) } else { (LEFT - 4.0, LEFT, AXIS) };
        let _ = write!(
            svg,
            r#"<line x1="{line_start}" y1="{yy:.1}" x2="{line_end:.1}" y2="{yy:.1}" stroke="{color}" stroke-width="1"/><text x="{}" y="{:.1}" text-anchor="end" {tick_style}>{}</text>"#,
            LEFT - 8.0,
            yy + 4.0,
            fmt_num(y, ystep)
        );
    }
    let (xt, xstep) = nice_ticks(x0, x1, 8);
    for &x in &xt {
        let xx = px(x);
        let _ = write!(
            svg,
            r#"<line x1="{xx:.1}" y1="{:.1}" x2="{xx:.1}" y2="{:.1}" stroke="{AXIS}" stroke-width="1"/><text x="{xx:.1}" y="{:.1}" text-anchor="middle" {tick_style}>{}</text>"#,
            TOP + ph,
            TOP + ph + 4.0,
            TOP + ph + 18.0,
            fmt_num(x, xstep)
        );
    }
    let _ = write!(
        svg,
        r#"<line x1="{LEFT}" y1="{:.1}" x2="{:.1}" y2="{:.1}" stroke="{AXIS}" stroke-width="1"/>"#,
        TOP + ph,
        LEFT + pw,
        TOP + ph
    );
    let _ = write!(
        svg,
        r#"<text x="{:.1}" y="{:.1}" text-anchor="middle" font-size="12" fill="{INK_2}">{}</text>"#,
        LEFT + pw / 2.0,
        HEIGHT - 12.0,
        escape(x_label)
    );
    let _ = write!(
        svg,
        r#"<text transform="translate(18 {:.1}) rotate(-90)" text-anchor="middle" font-size="12" fill="{INK_2}">{}</text>"#,
        TOP + ph / 2.0,
        escape(y_label)
    );
}

/// Min and max of the finite values, widened if they're equal.
fn data_range(values: impl Iterator<Item = f64>) -> (f64, f64) {
    let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
    for v in values.filter(|v| v.is_finite()) {
        lo = lo.min(v);
        hi = hi.max(v);
    }
    if lo > hi {
        (0.0, 1.0)
    } else if lo == hi {
        (lo - 1.0, hi + 1.0)
    } else {
        (lo, hi)
    }
}

/// Tick positions at "nice" round numbers (1, 2 or 5 times a power of ten).
fn nice_ticks(lo: f64, hi: f64, target: usize) -> (Vec<f64>, f64) {
    let raw = (hi - lo).abs() / target as f64;
    if !raw.is_finite() || raw <= 0.0 {
        return (vec![lo], 1.0);
    }
    let mag = 10f64.powf(raw.log10().floor());
    let step = mag
        * match raw / mag {
            n if n < 1.5 => 1.0,
            n if n < 3.0 => 2.0,
            n if n < 7.0 => 5.0,
            _ => 10.0,
        };
    let mut ticks = Vec::new();
    let mut t = (lo / step).ceil() * step;
    while t <= hi + step * 1e-9 {
        ticks.push(t);
        t += step;
    }
    (ticks, step)
}

fn fmt_num(v: f64, step: f64) -> String {
    let decimals = (-step.log10().floor()).clamp(0.0, 6.0) as usize;
    let s = format!("{:.*}", decimals, v);
    if s.starts_with('-') && s.trim_start_matches(['-', '0', '.']).is_empty() {
        s[1..].to_string() // "-0.0" -> "0.0"
    } else {
        s
    }
}

/// Split a series at NaN/infinite values so gaps stay gaps.
fn finite_segments(xs: &[f64], ys: &[f64]) -> Vec<Vec<(f64, f64)>> {
    let mut segments = vec![Vec::new()];
    for (&x, &y) in xs.iter().zip(ys) {
        if x.is_finite() && y.is_finite() {
            segments.last_mut().unwrap().push((x, y));
        } else if !segments.last().unwrap().is_empty() {
            segments.push(Vec::new());
        }
    }
    segments.retain(|s| !s.is_empty());
    segments
}

/// Thousands of points in one pixel column look the same as their min and max,
/// so keep just those. Keeps waveforms of long recordings small and accurate.
fn decimate(pts: &[(f64, f64)]) -> Vec<(f64, f64)> {
    if pts.len() < 4000 {
        return pts.to_vec();
    }
    let mut out = Vec::new();
    let mut i = 0;
    while i < pts.len() {
        let column = pts[i].0.floor();
        let mut j = i;
        let (mut lo, mut hi) = (i, i);
        while j < pts.len() && pts[j].0.floor() == column {
            if pts[j].1 < pts[lo].1 {
                lo = j;
            }
            if pts[j].1 > pts[hi].1 {
                hi = j;
            }
            j += 1;
        }
        out.push(pts[lo.min(hi)]);
        if lo != hi {
            out.push(pts[lo.max(hi)]);
        }
        i = j;
    }
    out
}

fn ramp_color(t: f64) -> (u8, u8, u8) {
    let t = if t.is_finite() { t.clamp(0.0, 1.0) } else { 0.0 };
    let pos = t * (RAMP.len() - 1) as f64;
    let k = (pos.floor() as usize).min(RAMP.len() - 2);
    let f = pos - k as f64;
    let mix = |a: u8, b: u8| (a as f64 + (b as f64 - a as f64) * f).round() as u8;
    let (a, b) = (RAMP[k], RAMP[k + 1]);
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

fn write_file(path: &Path, contents: &str) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        if !dir.as_os_str().is_empty() {
            std::fs::create_dir_all(dir)?;
        }
    }
    std::fs::write(path, contents)
}

// ---------------------------------------------------------------------------
// A minimal PNG writer: 8-bit RGB, no compression. A PNG file is a signature
// followed by chunks, each with a length, a 4-letter type, data and a CRC.
// The pixel data is wrapped in zlib "stored" blocks, which skip compression.
// ---------------------------------------------------------------------------

fn png_bytes(width: u32, height: u32, rgb: &[u8]) -> Vec<u8> {
    let mut raw = Vec::with_capacity(rgb.len() + height as usize);
    for row in rgb.chunks(width as usize * 3) {
        raw.push(0); // filter type 0: none
        raw.extend_from_slice(row);
    }

    let mut zlib = vec![0x78, 0x01];
    let blocks: Vec<&[u8]> = if raw.is_empty() { vec![&[]] } else { raw.chunks(65535).collect() };
    for (i, block) in blocks.iter().enumerate() {
        zlib.push(if i + 1 == blocks.len() { 1 } else { 0 });
        let len = block.len() as u16;
        zlib.extend_from_slice(&len.to_le_bytes());
        zlib.extend_from_slice(&(!len).to_le_bytes());
        zlib.extend_from_slice(block);
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut ihdr = Vec::new();
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]); // 8 bits, RGB, default methods

    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    for (kind, data) in [(b"IHDR", &ihdr), (b"IDAT", &zlib), (b"IEND", &Vec::new())] {
        png.extend_from_slice(&(data.len() as u32).to_be_bytes());
        let start = png.len();
        png.extend_from_slice(kind);
        png.extend_from_slice(data);
        let crc = crc32(&png[start..]);
        png.extend_from_slice(&crc.to_be_bytes());
    }
    png
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in bytes {
        crc ^= b as u32;
        for _ in 0..8 {
            crc = if crc & 1 != 0 { (crc >> 1) ^ 0xEDB8_8320 } else { crc >> 1 };
        }
    }
    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    for &x in bytes {
        a = (a + x as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn base64(bytes: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = (chunk[0] as u32) << 16
            | (*chunk.get(1).unwrap_or(&0) as u32) << 8
            | *chunk.get(2).unwrap_or(&0) as u32;
        for k in 0..4 {
            if k <= chunk.len() {
                out.push(ALPHABET[(n >> (18 - 6 * k) & 63) as usize] as char);
            } else {
                out.push('=');
            }
        }
    }
    out
}
