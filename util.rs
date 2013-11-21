use extra::json;

use std::io::{Reader, Writer};
use std::str;

use crypto;
use json::ExtraJSON;

impl crypto::SHA1 {
    pub fn special_digest(self) -> ~str {
        let mut digest = self.final();

        let neg = (digest[0] & 0x80) == 0x80;
        if neg {
            let mut carry = true;
            for x in digest.mut_iter().invert() {
                *x = !*x;
                if carry {
                    carry = *x == 0xFF;
                    *x = *x + 1;
                }
            }
        }

        let digest = crypto::SHA1::hex(digest);
        let digest = digest.trim_left_chars(&'0').to_owned();

        if neg { "-" + digest } else { digest }
    }
}

pub trait WriterExtensions: Writer {
    fn write_varint(&mut self, mut x: i32) {
        let mut buf = [0u8, ..10];
        let mut i = 0;
        if x < 0 {
            x = x + (1 << 32);
        }
        while x >= 0x80 {
            buf[i] = (x & 0x7F) as u8 | 0x80;
            x = x >> 7;
            i = i + 1;
        }
        buf[i] = x as u8;

        self.write(buf.slice_to(i + 1));
    }

    fn write_string(&mut self, s: &str) {
        self.write_varint(s.len() as i32);
        self.write(s.as_bytes());
    }
}

impl<T: Writer> WriterExtensions for T {}

pub trait ReaderExtensions: Reader {
    fn read_varint(&mut self) -> i32 {
        let (mut total, mut shift, mut val) = (0, 0, 0x80);

        while (val & 0x80) != 0 {
            val = self.read_u8() as i32;
            total = total | ((val & 0x7F) << shift);
            shift = shift + 7;
        }

        if (total & (1 << 31)) != 0 {
            total = total - (1 << 32);
        }

        total
    }

    fn read_string(&mut self) -> ~str {
        let len = self.read_varint();
        let buf = self.read_bytes(len as uint);

        str::from_utf8_owned(buf)
    }
}

impl<T: Reader> ReaderExtensions for T {}

pub fn maybe_print_message(j: json::Json) {
    // Let's wrap up the Json so that we can
    // deal with it more easily
    let j = ExtraJSON::new(j);

    let ty = j["translate"].string();

    // Player Chat
    if "chat.type.text" == ty {

        let user = j["with"][0]["text"].string();
        let msg = j["with"][1].string();
        println!("<{}> {}", user, msg);

    // Server Message
    } else if "chat.type.announcement" == ty {

        let msg = j["with"][1]["extra"].list_map(|x| x.string()).concat();
        println!("[Server] {}", msg);

    }
}
