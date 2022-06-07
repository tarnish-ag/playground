grammar lox::expr::term; // extends/includes ...
// what operator to use for cons ?
with core; // with == use namespace

ignore terminal Comment ::= /\/\/[^\n]*\n/ | /\/\*.*\*\// ;
ignore terminal Whitespace ::= /\s/ ;

enum Term { // enum == abstract terminal ?
    Bool(bool),
    Str(str),
    Num(float),
    Nil,
    Name(str)
}

trait TermT { term : Term }

// longest first or declaration order ?
// concrete/abstract & terminal/nonterminal inferred/unified in main language ?
concrete terminal Boolean : TermT ::=
    // all variants should have the same attributes
    | "true" { // optional leading "|"
        self.term : Term = Term::Bool(true) ;
    } | "false" {
        self.term : Term = Term::Bool(false) ;
    } // types inferred in the main language

concrete terminal String : TermT ::=
    "\"" (st : /[^"]/) "\"" {
        self.term : Term = Term::Str(st) ;
    }

concrete terminal Number : TermT ::=
    /\d+(.\d+)?/ {
        self.term : Term = Term::Num(float::from(@self)) ; // `@self` = literal token
    }

concrete terminal Nil : TermT ::=
    "nil" {
        self.term : Term = Term::Nil ;
    }

// terminal & its production unified ?
concrete terminal Name : TermT ::=
    /[:alpha:][:alnum:]*/ {
        self.term : Term = Term::Name(@self) ;
    }