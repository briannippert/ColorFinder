use std::error::Error;
use csv;
use serde::Deserialize;
use std::time::Instant;

struct ColorRGB {
    r: u8,
    g: u8,
    b: u8,
}
#[derive(Debug, Deserialize)]
struct CsvColorRecord {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Hex (24 bit)")]
    hex: String,
    #[serde(default)]
    #[serde(skip_deserializing)]
    _ignore: (),
}
#[derive(Debug, Clone, Copy)]
struct ColorYCbCr {
    y: f64,
    cb: f64,
    cr: f64,
}
struct NamedColor {
    name: String,
    ycbcr: ColorYCbCr,
}

mod user_input {
    use std::io;
    pub fn get_input(prompt: &str) -> String{
        println!("{}",prompt);
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(_goes_into_input_above) => {},
            Err(_no_updates_is_fine) => {},
        }
        input.trim().to_string()
    }
}

fn convert_ycbcr(rgb: ColorRGB) -> ColorYCbCr {
    let r_norm = rgb.r as f64 / 255.0;
    let g_norm = rgb.g as f64 / 255.0;
    let b_norm = rgb.b as f64 / 255.0;
    let y = 0.299 * r_norm + 0.587 * g_norm + 0.114 * b_norm;
    let cb = 0.564 * (b_norm - y);
    let cr = 0.713 * (r_norm - y);
    ColorYCbCr { y, cb, cr }
}

fn convert_hex(ycbcr: ColorYCbCr) -> String {
    let r_norm = ycbcr.y + 1.402 * ycbcr.cr;
    let g_norm = ycbcr.y - 0.344136 * ycbcr.cb - 0.714136 * ycbcr.cr;
    let b_norm = ycbcr.y + 1.772 * ycbcr.cb;
    let r = (r_norm.clamp(0.0, 1.0) * 255.0).round() as u8;
    let g = (g_norm.clamp(0.0, 1.0) * 255.0).round() as u8;
    let b = (b_norm.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

fn hex_to_rgb(hex: &str) -> Result<ColorRGB, &'static str> {
    // Ensure the input is exactly 7 characters long and starts with '#'
    if hex.len() != 7 || !hex.starts_with('#') {
        return Err("Invalid hex format. Must be '#RRGGBB'.");
    }

    let r_hex = &hex[1..3];
    let g_hex = &hex[3..5];
    let b_hex = &hex[5..7];

    let r = u8::from_str_radix(r_hex, 16).map_err(|_| "Invalid hex R component")?;
    let g = u8::from_str_radix(g_hex, 16).map_err(|_| "Invalid hex G component")?;
    let b = u8::from_str_radix(b_hex, 16).map_err(|_| "Invalid hex B component")?;

    Ok(ColorRGB { r, g, b })
}
fn find_closest_color(target_ycbcr: ColorYCbCr, named_colors: &[NamedColor]) -> (&str, f64, String) {
    if named_colors.is_empty() {
        return ("", f64::NAN, String::from("NULL"));
    }
    let mut closest_name = "";
    let mut min_distance_sq = f64::MAX;
    let mut closest_color_YCbCr = ColorYCbCr { y: 0.0, cb: 0.0, cr: 0.0 };
    for color in named_colors {
        let distance_sq = color_distance_sq(target_ycbcr, color.ycbcr);

        if distance_sq < min_distance_sq {
            min_distance_sq = distance_sq;
            closest_name = &color.name;
            closest_color_YCbCr = color.ycbcr;
        }
    }
    let min_distance = min_distance_sq.sqrt();
    let closest_color_hex = convert_hex(closest_color_YCbCr);
    (closest_name, min_distance, closest_color_hex)
}

fn color_distance_sq(c1: ColorYCbCr, c2: ColorYCbCr) -> f64 {
    let dy = c1.y - c2.y;
    let dcb = c1.cb - c2.cb;
    let dcr = c1.cr - c2.cr;
    dy * dy + dcb * dcb + dcr * dcr
}

fn load_and_process_colors(file_path: &str) -> Result<Vec<NamedColor>, Box<dyn Error>> {
    let mut named_colors = Vec::new();
    let mut rdr = csv::ReaderBuilder::new()
        .has_headers(true)
        .from_path(file_path)?;

    for result in rdr.deserialize() {
        let record: CsvColorRecord = result?;

        match hex_to_rgb(&record.hex) {
            Ok(rgb) => {
                let ycbcr = convert_ycbcr(rgb);
                named_colors.push(NamedColor {
                    name: record.name,
                    ycbcr,
                });
            },
            Err(e) => {
                eprintln!("Skipping color '{}' due to hex parse error: {}", record.name, e);
            }
        }
    }

    Ok(named_colors)
}

fn main() -> Result<(), Box<dyn Error>> {
    const FILE_PATH: &str = "input/color_names.csv";
    println!("Loading colors from: {}", FILE_PATH);
    let named_colors = match load_and_process_colors(FILE_PATH) {
        Ok(colors) => {
            println!("Successfully loaded {} named colors.", colors.len());
            colors
        },
        Err(e) => {
            eprintln!("Fatal error loading CSV data: {}", e);
            return Err(e);
        }
    };

    if named_colors.is_empty() {
        eprintln!("No valid color data loaded. Exiting.");
        return Ok(());
    }


    let input = user_input::get_input("Enter a Hex Color (e.g., #123456): ");
    let start_time = Instant::now();
    match hex_to_rgb(&input) {
        Ok(input_rgb) => {
            let target_ycbcr = convert_ycbcr(input_rgb);
            let (closest_name, distance, closest_hex) = find_closest_color(target_ycbcr, &named_colors);
            let end_time = Instant::now();
            println!("Processing time: {:.2?}", end_time - start_time);
            println!("Closest Named Color: {} - {}", closest_name, closest_hex);
            println!("Color Difference (Euclidean Distance in YCbCr space): {:.4}", distance);
        }
        Err(e) => {
            println!("\nError processing input: {}", e);
        }
    }
    Ok(())
}