use super::Font;

pub struct Glean;

impl Font for Glean {
    const CHAR_WIDTH: u32 = 5;
    const CHAR_HEIGHT: u32 = 10;

    #[inline]
    fn get_char(c: char) -> Option<&'static [u8]> {
        match c {
            ' ' => Some(&[
                5, 10, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 7,
            ]),
            '!' => Some(&[3, 10, 255, 182, 125, 63]),
            '"' => Some(&[4, 10, 95, 85, 255, 255, 255]),
            '#' => Some(&[5, 10, 181, 130, 90, 65, 173, 255, 3]),
            '$' => Some(&[5, 10, 255, 142, 58, 87, 220, 255, 3]),
            '%' => Some(&[5, 10, 255, 183, 122, 247, 106, 255, 3]),
            '&' => Some(&[5, 10, 255, 239, 186, 87, 93, 255, 3]),
            '\'' => Some(&[3, 10, 223, 246, 255, 63]),
            '(' => Some(&[5, 10, 255, 221, 189, 247, 190, 239, 3]),
            ')' => Some(&[4, 10, 223, 123, 119, 183, 253]),
            '*' => Some(&[4, 10, 255, 177, 245, 255, 255]),
            '+' => Some(&[5, 10, 255, 255, 189, 193, 222, 255, 3]),
            ',' => Some(&[3, 10, 255, 255, 111, 61]),
            '-' => Some(&[5, 10, 255, 255, 255, 195, 255, 255, 3]),
            '.' => Some(&[3, 10, 255, 255, 127, 63]),
            '/' => Some(&[5, 10, 255, 189, 123, 247, 238, 253, 3]),
            '0' => Some(&[5, 10, 255, 207, 86, 82, 155, 255, 3]),
            '1' => Some(&[4, 10, 255, 55, 119, 119, 255]),
            '2' => Some(&[5, 10, 255, 207, 118, 119, 15, 255, 3]),
            '3' => Some(&[5, 10, 255, 135, 119, 95, 155, 255, 3]),
            '4' => Some(&[5, 10, 255, 223, 89, 195, 189, 255, 3]),
            '5' => Some(&[5, 10, 255, 135, 30, 95, 155, 255, 3]),
            '6' => Some(&[5, 10, 255, 207, 30, 91, 155, 255, 3]),
            '7' => Some(&[5, 10, 255, 135, 119, 239, 222, 255, 3]),
            '8' => Some(&[5, 10, 255, 207, 54, 91, 155, 255, 3]),
            '9' => Some(&[5, 10, 255, 207, 214, 198, 155, 255, 3]),
            ':' => Some(&[3, 10, 255, 247, 239, 63]),
            ';' => Some(&[3, 10, 255, 247, 111, 61]),
            '<' => Some(&[5, 10, 255, 221, 221, 247, 125, 255, 3]),
            '=' => Some(&[5, 10, 255, 255, 31, 126, 248, 255, 3]),
            '>' => Some(&[5, 10, 191, 239, 251, 238, 238, 255, 3]),
            '?' => Some(&[5, 10, 255, 199, 119, 247, 223, 255, 3]),
            '@' => Some(&[5, 10, 255, 207, 86, 74, 159, 255, 3]),
            'A' => Some(&[5, 10, 255, 207, 214, 66, 107, 255, 3]),
            'B' => Some(&[5, 10, 255, 199, 22, 91, 139, 255, 3]),
            'C' => Some(&[5, 10, 255, 207, 214, 123, 155, 255, 3]),
            'D' => Some(&[5, 10, 255, 199, 214, 90, 139, 255, 3]),
            'E' => Some(&[5, 10, 255, 135, 30, 123, 15, 255, 3]),
            'F' => Some(&[5, 10, 255, 135, 30, 123, 239, 255, 3]),
            'G' => Some(&[5, 10, 255, 207, 214, 75, 155, 255, 3]),
            'H' => Some(&[5, 10, 255, 183, 22, 90, 107, 255, 3]),
            'I' => Some(&[5, 10, 255, 143, 123, 239, 29, 255, 3]),
            'J' => Some(&[5, 10, 255, 159, 247, 94, 155, 255, 3]),
            'K' => Some(&[5, 10, 255, 183, 154, 107, 107, 255, 3]),
            'L' => Some(&[5, 10, 255, 247, 222, 123, 15, 255, 3]),
            'M' => Some(&[5, 10, 255, 183, 16, 90, 107, 255, 3]),
            'N' => Some(&[5, 10, 255, 183, 148, 74, 105, 255, 3]),
            'O' => Some(&[5, 10, 255, 207, 214, 90, 155, 255, 3]),
            'P' => Some(&[5, 10, 255, 199, 22, 123, 239, 255, 3]),
            'Q' => Some(&[5, 10, 255, 207, 214, 90, 154, 231, 3]),
            'R' => Some(&[5, 10, 255, 199, 22, 115, 109, 255, 3]),
            'S' => Some(&[5, 10, 255, 207, 182, 111, 155, 255, 3]),
            'T' => Some(&[5, 10, 255, 143, 123, 239, 189, 255, 3]),
            'U' => Some(&[5, 10, 255, 183, 214, 90, 155, 255, 3]),
            'V' => Some(&[5, 10, 255, 183, 214, 230, 156, 255, 3]),
            'W' => Some(&[5, 10, 255, 183, 214, 66, 104, 255, 3]),
            'X' => Some(&[5, 10, 255, 183, 54, 103, 107, 255, 3]),
            'Y' => Some(&[5, 10, 255, 183, 214, 230, 189, 255, 3]),
            'Z' => Some(&[5, 10, 255, 135, 119, 119, 15, 255, 3]),
            '[' => Some(&[4, 10, 63, 187, 187, 187, 243]),
            '\\' => Some(&[5, 10, 191, 247, 189, 239, 125, 239, 3]),
            ']' => Some(&[4, 10, 63, 119, 119, 119, 243]),
            '^' => Some(&[5, 10, 255, 207, 217, 218, 255, 255, 3]),
            '_' => Some(&[5, 10, 255, 255, 255, 255, 255, 225, 3]),
            '`' => Some(&[4, 10, 255, 123, 255, 255, 255]),
            'a' => Some(&[5, 10, 255, 255, 63, 90, 89, 255, 3]),
            'b' => Some(&[5, 10, 255, 247, 30, 91, 139, 255, 3]),
            'c' => Some(&[5, 10, 255, 255, 63, 122, 31, 255, 3]),
            'd' => Some(&[5, 10, 255, 191, 55, 90, 27, 255, 3]),
            'e' => Some(&[5, 10, 255, 255, 63, 75, 30, 255, 3]),
            'f' => Some(&[5, 10, 255, 223, 181, 227, 222, 255, 3]),
            'g' => Some(&[5, 10, 255, 255, 63, 90, 27, 109, 2]),
            'h' => Some(&[5, 10, 255, 247, 30, 91, 107, 255, 3]),
            'i' => Some(&[5, 10, 255, 223, 63, 239, 29, 255, 3]),
            'j' => Some(&[4, 10, 255, 247, 115, 119, 181]),
            'k' => Some(&[5, 10, 255, 247, 222, 106, 108, 255, 3]),
            'l' => Some(&[5, 10, 255, 207, 123, 239, 61, 255, 3]),
            'm' => Some(&[5, 10, 255, 255, 159, 66, 107, 255, 3]),
            'n' => Some(&[5, 10, 255, 255, 31, 91, 107, 255, 3]),
            'o' => Some(&[5, 10, 255, 255, 63, 91, 155, 255, 3]),
            'p' => Some(&[5, 10, 255, 255, 31, 91, 139, 189, 3]),
            'q' => Some(&[5, 10, 255, 255, 63, 90, 27, 239, 1]),
            'r' => Some(&[5, 10, 255, 255, 31, 91, 239, 255, 3]),
            's' => Some(&[5, 10, 255, 255, 63, 242, 137, 255, 3]),
            't' => Some(&[5, 10, 255, 239, 29, 246, 186, 255, 3]),
            'u' => Some(&[5, 10, 255, 255, 223, 90, 27, 255, 3]),
            'v' => Some(&[5, 10, 255, 255, 223, 218, 156, 255, 3]),
            'w' => Some(&[5, 10, 255, 255, 223, 90, 72, 255, 3]),
            'x' => Some(&[5, 10, 255, 255, 223, 230, 108, 255, 3]),
            'y' => Some(&[5, 10, 255, 255, 223, 90, 27, 109, 2]),
            'z' => Some(&[5, 10, 255, 255, 31, 238, 14, 255, 3]),
            '{' => Some(&[5, 10, 103, 239, 221, 247, 222, 231, 3]),
            '|' => Some(&[4, 10, 127, 119, 119, 119, 247]),
            '}' => Some(&[5, 10, 249, 222, 123, 238, 189, 249, 3]),
            '~' => Some(&[5, 10, 255, 255, 166, 236, 255, 255, 3]),
            '¡' => Some(&[3, 10, 255, 191, 111, 27]),
            '¢' => Some(&[5, 10, 255, 255, 59, 106, 29, 247, 3]),
            '£' => Some(&[5, 10, 255, 223, 181, 227, 14, 255, 3]),
            '¤' => Some(&[5, 10, 255, 255, 54, 219, 108, 255, 3]),
            '¥' => Some(&[5, 10, 255, 183, 25, 110, 184, 255, 3]),
            '¦' => Some(&[3, 10, 255, 182, 111, 59]),
            '§' => Some(&[5, 10, 127, 244, 217, 230, 139, 255, 3]),
            '¨' => Some(&[5, 10, 191, 181, 255, 255, 255, 255, 3]),
            '©' => Some(&[5, 10, 63, 58, 197, 152, 114, 241, 3]),
            'ª' => Some(&[5, 10, 127, 180, 178, 126, 248, 255, 3]),
            '«' => Some(&[5, 10, 255, 255, 102, 109, 251, 255, 3]),
            '¬' => Some(&[5, 10, 255, 255, 31, 222, 251, 255, 3]),
            '®' => Some(&[5, 10, 63, 58, 161, 148, 114, 241, 3]),
            '¯' => Some(&[5, 10, 255, 135, 255, 255, 255, 255, 3]),
            '°' => Some(&[4, 10, 159, 150, 255, 255, 255]),
            '±' => Some(&[5, 10, 255, 239, 13, 246, 254, 224, 3]),
            '²' => Some(&[3, 10, 175, 86, 252, 63]),
            '³' => Some(&[3, 10, 71, 167, 254, 63]),
            '´' => Some(&[4, 10, 127, 183, 255, 255, 255]),
            'µ' => Some(&[5, 10, 255, 255, 223, 90, 73, 253, 3]),
            '¶' => Some(&[5, 10, 255, 143, 214, 198, 57, 255, 3]),
            '·' => Some(&[3, 10, 255, 191, 253, 63]),
            '¸' => Some(&[3, 10, 255, 255, 191, 43]),
            '¹' => Some(&[3, 10, 95, 182, 253, 63]),
            'º' => Some(&[5, 10, 127, 182, 54, 127, 248, 255, 3]),
            '»' => Some(&[5, 10, 255, 127, 219, 154, 253, 255, 3]),
            '¼' => Some(&[5, 10, 59, 239, 253, 206, 26, 239, 3]),
            '½' => Some(&[5, 10, 59, 239, 125, 215, 187, 227, 3]),
            '¾' => Some(&[5, 10, 113, 223, 186, 206, 26, 239, 3]),
            '¿' => Some(&[5, 10, 255, 255, 127, 255, 221, 125, 0]),
            'À' => Some(&[5, 10, 251, 254, 217, 66, 107, 255, 3]),
            'Á' => Some(&[5, 10, 119, 255, 217, 66, 107, 255, 3]),
            'Â' => Some(&[5, 10, 179, 253, 217, 66, 107, 255, 3]),
            'Ã' => Some(&[5, 10, 171, 254, 217, 66, 107, 255, 3]),
            'Ä' => Some(&[5, 10, 173, 253, 217, 66, 107, 255, 3]),
            'Å' => Some(&[5, 10, 179, 205, 217, 66, 107, 255, 3]),
            'Æ' => Some(&[5, 10, 255, 143, 90, 98, 45, 255, 3]),
            'Ç' => Some(&[5, 10, 255, 207, 214, 123, 155, 119, 3]),
            'È' => Some(&[5, 10, 251, 254, 208, 99, 15, 255, 3]),
            'É' => Some(&[5, 10, 119, 255, 208, 99, 15, 255, 3]),
            'Ê' => Some(&[5, 10, 179, 253, 208, 99, 15, 255, 3]),
            'Ë' => Some(&[5, 10, 173, 253, 208, 99, 15, 255, 3]),
            'Ì' => Some(&[4, 10, 189, 31, 187, 27, 255]),
            'Í' => Some(&[4, 10, 183, 31, 187, 27, 255]),
            'Î' => Some(&[5, 10, 179, 253, 184, 247, 142, 255, 3]),
            'Ï' => Some(&[5, 10, 173, 253, 184, 247, 142, 255, 3]),
            'Ð' => Some(&[5, 10, 255, 231, 218, 80, 205, 255, 3]),
            'Ñ' => Some(&[5, 10, 171, 254, 150, 82, 105, 255, 3]),
            'Ò' => Some(&[5, 10, 251, 254, 217, 90, 155, 255, 3]),
            'Ó' => Some(&[5, 10, 119, 255, 217, 90, 155, 255, 3]),
            'Ô' => Some(&[5, 10, 179, 253, 217, 90, 155, 255, 3]),
            'Õ' => Some(&[5, 10, 171, 254, 217, 90, 155, 255, 3]),
            'Ö' => Some(&[5, 10, 173, 253, 217, 90, 155, 255, 3]),
            '×' => Some(&[5, 10, 255, 127, 87, 119, 117, 255, 3]),
            'Ø' => Some(&[5, 10, 255, 141, 82, 82, 138, 253, 3]),
            'Ù' => Some(&[5, 10, 251, 254, 214, 90, 155, 255, 3]),
            'Ú' => Some(&[5, 10, 119, 255, 214, 90, 155, 255, 3]),
            'Û' => Some(&[5, 10, 179, 253, 214, 90, 155, 255, 3]),
            'Ü' => Some(&[5, 10, 173, 253, 214, 90, 155, 255, 3]),
            'Ý' => Some(&[5, 10, 119, 255, 214, 230, 189, 255, 3]),
            'Þ' => Some(&[5, 10, 255, 247, 216, 98, 239, 255, 3]),
            'ß' => Some(&[5, 10, 255, 207, 86, 91, 171, 253, 3]),
            'à' => Some(&[5, 10, 127, 223, 63, 90, 89, 255, 3]),
            'á' => Some(&[5, 10, 255, 238, 63, 90, 89, 255, 3]),
            'â' => Some(&[5, 10, 127, 182, 63, 90, 89, 255, 3]),
            'ã' => Some(&[5, 10, 127, 213, 63, 90, 89, 255, 3]),
            'ä' => Some(&[5, 10, 191, 181, 63, 90, 89, 255, 3]),
            'å' => Some(&[5, 10, 179, 205, 63, 90, 89, 255, 3]),
            'æ' => Some(&[5, 10, 255, 255, 63, 74, 13, 255, 3]),
            'ç' => Some(&[4, 10, 255, 255, 211, 61, 183]),
            'è' => Some(&[5, 10, 127, 223, 63, 75, 30, 255, 3]),
            'é' => Some(&[5, 10, 255, 238, 63, 75, 30, 255, 3]),
            'ê' => Some(&[5, 10, 127, 182, 63, 75, 30, 255, 3]),
            'ë' => Some(&[5, 10, 191, 181, 63, 75, 30, 255, 3]),
            'ì' => Some(&[4, 10, 191, 247, 115, 119, 255]),
            'í' => Some(&[5, 10, 255, 221, 63, 239, 189, 255, 3]),
            'î' => Some(&[5, 10, 127, 182, 63, 239, 189, 255, 3]),
            'ï' => Some(&[5, 10, 191, 181, 63, 239, 189, 255, 3]),
            'ð' => Some(&[5, 10, 191, 238, 122, 87, 155, 255, 3]),
            'ñ' => Some(&[5, 10, 127, 213, 31, 91, 107, 255, 3]),
            'ò' => Some(&[5, 10, 127, 223, 63, 91, 155, 255, 3]),
            'ó' => Some(&[5, 10, 255, 238, 63, 91, 155, 255, 3]),
            'ô' => Some(&[5, 10, 127, 182, 63, 91, 155, 255, 3]),
            'õ' => Some(&[5, 10, 127, 213, 63, 91, 155, 255, 3]),
            'ö' => Some(&[5, 10, 191, 181, 63, 91, 155, 255, 3]),
            '÷' => Some(&[5, 10, 127, 239, 15, 254, 222, 255, 3]),
            'ø' => Some(&[5, 10, 255, 255, 63, 74, 138, 255, 3]),
            'ù' => Some(&[5, 10, 127, 223, 223, 90, 27, 255, 3]),
            'ú' => Some(&[5, 10, 255, 238, 223, 90, 27, 255, 3]),
            'û' => Some(&[5, 10, 127, 182, 223, 90, 27, 255, 3]),
            'ü' => Some(&[5, 10, 191, 181, 223, 90, 27, 255, 3]),
            'ý' => Some(&[5, 10, 255, 238, 223, 90, 27, 109, 2]),
            'þ' => Some(&[5, 10, 255, 247, 30, 91, 139, 189, 3]),
            'ÿ' => Some(&[5, 10, 191, 181, 223, 90, 27, 109, 2]),
            '♡' => Some(&[5, 10, 255, 255, 170, 92, 221, 255, 255]),
            _ => None,
        }
    }
}