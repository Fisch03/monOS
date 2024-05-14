use rayon::prelude::*;
use std::io::Write;

struct CollectedChar {
    character: char,
    character_hex: String,
    byte_arr: String,
}

fn main() {
    let characters = include_str!("characters.txt");
    let fonts = std::fs::read_dir("fonts").unwrap();

    for font in fonts {
        let font_path = font.unwrap().path();
        let font_name = font_path.file_name().unwrap().to_str().unwrap();
        dbg!(&font_path);
        let mut characters = characters
            .chars()
            .par_bridge()
            .filter_map(|c| {
                let c_hex = format!("{:05x}", c as i32);

                let mut cmd = std::process::Command::new("pbmtext");
                cmd.arg("-nomargins");
                cmd.arg("-wchar");
                cmd.arg("-font").arg(&font_path);

                let mut child = cmd
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .spawn()
                    .unwrap();

                let mut stdin = child.stdin.take().unwrap();
                stdin.write_all(c.to_string().as_bytes()).unwrap();
                drop(stdin);

                let output = child.wait_with_output().unwrap();
                if let Ok(image) = image::load_from_memory(&output.stdout) {
                    let image = image.to_luma8();
                    // image
                    //     .save(format!("out/{}_{}.png", font_name, c_hex))
                    //     .unwrap();

                    let mut out = image
                        .as_raw()
                        .chunks(8)
                        .map(|byte| {
                            byte.iter().enumerate().fold(0, |acc, (i, &b)| {
                                if b > 0 {
                                    return acc | (1 << i);
                                } else {
                                    return acc;
                                }
                            })
                        })
                        .collect::<Vec<u8>>();

                    out.insert(0, image.width() as u8);
                    out.insert(1, image.height() as u8);

                    let out = out.iter().map(|b| format!("{}, ", b)).collect::<String>();
                    let byte_arr = format!("&[{}]\n", out);

                    Some(CollectedChar {
                        character: c,
                        character_hex: c_hex,
                        byte_arr,
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        characters.sort_by(|a, b| a.character.cmp(&b.character));

        let mut file = std::fs::File::create(format!("out/{}.rs", font_name)).unwrap();
        writeln!(file, "#[inline]").unwrap();
        writeln!(file, "pub fn get_char(c: char) -> Option<&'static [u8]> {{").unwrap();
        writeln!(file, "match c {{").unwrap();
        writeln!(
            file,
            "' ' => Some(&[6, 13, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 7]),"
        )
        .unwrap();
        for c in characters {
            match c.character {
                ' ' => {}
                '\\' => writeln!(file, "'\\\\' => Some({}),", c.byte_arr).unwrap(),
                '\'' => writeln!(file, "'\\'' => Some({}),", c.byte_arr).unwrap(),
                _ => writeln!(file, "'{}' => Some({}),", c.character, c.byte_arr).unwrap(),
            }
        }
        writeln!(file, "_ => None,").unwrap();
        writeln!(file, "}}").unwrap();
        writeln!(file, "}}").unwrap();

        drop(file);

        std::process::Command::new("rustfmt")
            .arg(format!("out/{}.rs", font_name))
            .output()
            .unwrap();
    }
}
