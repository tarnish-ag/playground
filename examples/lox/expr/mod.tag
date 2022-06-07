grammar lox::expr; // extends/includes ...
with core; // always in scope unless overwritten
with term; // module scope

enum Expr {
    Term(term::Term),
    Expr(Expr), // should we allow recursive w/o ref, decide during implementation?
    Dot(Expr, str),
    Call(Expr, [Expr])
}

trait ExprT { e : Expr }

// template, not parsed unless instantiated, could be an extention instead of core, would still be nice in core
concrete nonterminal Comma<T> ::=
    (e : T) {
        self.lst : [T] = [e] ;
    } |  (hd : T) "," (tl : Comma<T>) {
        self.lst : [T] = [hd, ...tl.lst] ;
    }

// `=` for aliasing
concrete nonterminal Arguments = Comma<Expr> ;

concrete nonterminal Primary : ExprT ::=
    (term : Boolean)
      | (term : Nil)
      | (term : String)
      | (term : Number)
      | (term : Name)
    { // all grouped variants should have the same bindings
        self.e = Expr::Term(term.term) ;
    } | "(" (expr : Expr) ")" { // ambiguities / match order?
        self.e = Expr::Expr(expr.e) ;
    } | (p : Primary) "." (nm : Name) {
        self.e = Expr::Dot(p.e, @nm) ; // `@...` = literal token of `nm`
    } | (f : Primary) "(" (args_opt : Arguments?) ")" { // `?` gives `Option`, could be an extention instead of core, would still be nice in core
        self.e = Expr::Call(f.e, match(args_opt) {
            Some(args) => args.lst,
            None => []
        })
    }