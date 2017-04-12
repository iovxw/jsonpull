error_chain! {
    errors {
        Syntax(excped: ::ExpectType, found: char) {
            description("invalid syntax")
            display("expected '{:?}', found '{}'", excped, found)
        }
    }

    foreign_links {
        Utf8(::std::string::FromUtf8Error);
        Io(::std::io::Error);
        Unicode(::std::num::ParseIntError);
    }
}
