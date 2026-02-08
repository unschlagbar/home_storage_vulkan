#![allow(unused)]

use std::{
    char,
    fs::File,
    io::{BufReader, Write},
};

#[test]
fn generate() {
    use std::{
        fs::File,
        io::{BufReader, Write},
    };

    let file = File::open("font/default8.png").unwrap();

    let decoder = png::Decoder::new(BufReader::new(file));
    let mut reader = decoder.read_info().unwrap();

    let mut buf = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut buf).unwrap();

    let img_width = info.width as usize;
    let img_height = info.height as usize;
    let bytes_per_pixel = info.color_type.samples();

    let char_size = 8;
    let cols = img_width / char_size;
    let rows = img_height / char_size;

    // (x, y, width)
    let mut chars = vec![vec![(0u16, 0u16, 0u16); cols]; rows];

    let mut i = -1;

    for row in 0..rows {
        for col in 0..cols {
            i += 1;

            let start_x = col * char_size;
            let start_y = row * char_size;

            let mut max_x = 0;

            for y in 0..char_size {
                let py = start_y + y;
                for x in 0..char_size {
                    let px = start_x + x;

                    let idx = (py * img_width + px) * bytes_per_pixel;

                    // Alpha oder Luminanz prüfen
                    let visible = match info.color_type {
                        png::ColorType::Rgba => buf[idx + 3] != 0,
                        png::ColorType::GrayscaleAlpha => buf[idx + 1] != 0,
                        png::ColorType::Grayscale => buf[idx] != 0,
                        _ => buf[idx] != 0,
                    };

                    if visible {
                        max_x = max_x.max(x + 2);
                    }
                }
            }

            if i == 32 {
                max_x = 2;
            }

            chars[row][col] = (
                start_x as u16,
                start_y as u16,
                max_x.clamp(1, char_size) as u16,
            );
        }
    }

    let mut out_file = File::create("font/std2.fef").unwrap();

    for row in &chars {
        for (x, y, w) in row {
            out_file.write_all(&x.to_le_bytes()).unwrap();
            out_file.write_all(&y.to_le_bytes()).unwrap();
            out_file.write_all(&w.to_le_bytes()).unwrap();
        }
    }
}
