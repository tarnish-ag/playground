enum Term {
    Str(String),
    Re(String),
}

struct Terminal {
    term: Term,
    ignore: bool,
}

// term_re!(ignore Whitespace = r"\s"); // =>

// term_re!(Num = r"[0-9]+");

// prod!(Name ::=
//     rule!(id *),
//     attributes!{
//         // attrs
//     }
// );
