use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("input_codes.rs");

    std::fs::write(dest, get_keys()).unwrap();
    println!("cargo::rerun-if-changed=build.rs");
}

fn get_keys() -> String {
    let event_codes = get_event_codes().expect("Can get event codes");

    let mut output = String::new();

    for (key, val) in event_codes {
        let new = format!("pub const {key}: u32 = {val};\n");
        output.push_str(&new);
    }

    output
}

fn event_codes_file_path() -> PathBuf {
    PathBuf::from("/usr/include/linux/input-event-codes.h")
}

fn trim_comments(input: &str) -> String {
    let mut out = String::new();

    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '/' && chars.peek() == Some(&'*') {
            chars.next(); // Consume the *
            loop {
                if chars.next() == Some('*') && chars.next() == Some('/') {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }

    out
}

fn trim_empty_lines(input: &str) -> Vec<String> {
    let mut out = Vec::new();

    for line in input.lines() {
        let new = line.trim();
        if line.trim() == "" {
            continue;
        }

        out.push(new.to_string());
    }

    out
}

fn to_key_val(input: &[String]) -> Option<Vec<(String, u32)>> {
    let mut out = Vec::new();

    let mut lines = input.iter();

    while let Some(line) = lines.next() {
        if line.starts_with("#ifndef") || line.starts_with("#endif") {
            lines.next(); // Skip next line
            continue;
        }

        let mut s = line.trim_start_matches("#define").split_whitespace();
        // let key = s.next()?.to_string();
        // let val_str = s.next()?;
        let key = s.next().unwrap().to_string();
        let val_str = s.next().unwrap();

        // Dont include weird things that reference other keys. Not needed
        if val_str.starts_with('(') {
            continue;
        }

        let radix = if val_str.starts_with("0x") { 16 } else { 10 };

        let mut trimed = val_str.trim_start_matches("0x").trim_start_matches("0");
        if trimed.is_empty() {
            trimed = "0";
        }

        let Some(val) = u32::from_str_radix(trimed, radix).ok() else {
            continue;
        };

        out.push((key, val));
    }

    Some(out)
}

fn get_event_codes() -> Option<Vec<(String, u32)>> {
    to_key_val(&trim_empty_lines(&trim_comments(
        &std::fs::read_to_string(event_codes_file_path()).ok()?,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_auto() {
        u32::from_str_radix("7", 16).unwrap();

        let data = &std::fs::read_to_string(event_codes_file_path()).unwrap();
        let trimed = &trim_empty_lines(&trim_comments(data));

        to_key_val(trimed).unwrap();
    }

    #[test]
    fn test_all_manual() {
        // God this is awful to look at

        let comments = trim_comments(
            r#"#define INPUT_PROP_CNT			(INPUT_PROP_MAX + 1)

/*
 * Event types
 */

#define EV_SYN			0x00
#define INPUT_PROP_PRESSUREPAD		0x07	/* pressure triggers clicks */
"#,
        );

        // -----------------------------------------------------

        assert_eq!(
            comments,
            r#"#define INPUT_PROP_CNT			(INPUT_PROP_MAX + 1)



#define EV_SYN			0x00
#define INPUT_PROP_PRESSUREPAD		0x07	
"#
            .to_string()
        );

        // -----------------------------------------------------

        let lined = trim_empty_lines(&comments);
        assert_eq!(
            lined,
            vec!(
                "#define INPUT_PROP_CNT			(INPUT_PROP_MAX + 1)",
                "#define EV_SYN			0x00",
                "#define INPUT_PROP_PRESSUREPAD		0x07",
            )
        );

        // -----------------------------------------------------

        let keyd = to_key_val(&lined);
        assert_eq!(
            keyd,
            Some(
                [("EV_SYN", 0x00), ("INPUT_PROP_PRESSUREPAD", 0x07),]
                    .iter()
                    .map(|(k, v)| (k.to_string(), *v))
                    .collect()
            )
        );
    }
}
