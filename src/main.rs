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
fn find_closest_color(target_ycbcr: ColorYCbCr, named_colors: &[NamedColor]) -> (&str, f64) {
    if named_colors.is_empty() {
        return ("", f64::NAN);
    }
    let mut closest_name = "";
    let mut min_distance_sq = f64::MAX;
    for color in named_colors {
        let distance_sq = color_distance_sq(target_ycbcr, color.ycbcr);

        if distance_sq < min_distance_sq {
            min_distance_sq = distance_sq;
            closest_name = &color.name;
        }
    }
    let min_distance = min_distance_sq.sqrt();
    (closest_name, min_distance)
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
            let (closest_name, distance) = find_closest_color(target_ycbcr, &named_colors);
            let end_time = Instant::now();
            println!("Processing time: {:.2?}", end_time - start_time);
            println!("Closest Named Color: **{}**", closest_name);
            println!("Color Difference (Euclidean Distance in YCbCr space): {:.4}", distance);
        }
        Err(e) => {
            println!("\nError processing input: {}", e);
        }
    }
    Ok(())
}