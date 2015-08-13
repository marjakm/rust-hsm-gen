#![feature(quote, rustc_private)]
#![feature(box_syntax)]

extern crate syntax;
extern crate rustc;
extern crate rustc_driver;


#[cfg(test)]
mod test {
    use std::io::Write;
    use syntax::ext::base::ExtCtxt;

    use syntax::ast;
    use syntax::parse;
    use syntax::ext;
    use syntax::print;

    use rustc::session::{build_session,Session};
    use rustc::session::config::{build_session_options,build_configuration, Input};
    use rustc_driver::{handle_options,diagnostics_registry};
    use rustc_driver::driver::{phase_1_parse_input, source_name};

    fn pprint(sess: Session, krate: &ast::Crate, src_name: String) {
        let src = sess.codemap().get_filemap(&src_name[..])
                            .src
                            .as_ref()
                            .unwrap()
                            .as_bytes()
                            .to_vec();
        let mut rdr = &src[..];

        let mut out = Vec::new();
        let ann = print::pprust::NoAnn;
        {
            let out_w: &mut Write = &mut out;
            print::pprust::print_crate(sess.codemap(),
                                            sess.diagnostic(),
                                            krate,
                                            src_name,
                                            &mut rdr,
                                            box out_w,
                                            &ann,
                                            false);
        }
        print!("{}", String::from_utf8(out).unwrap());
    }


    #[test]
    fn it_works() {
        let crate_src = "fn main(){}".to_string();

        let matches = match handle_options(vec!["".to_string(), "-".to_string()]) {
            Some(matches) => matches,
            None => unreachable!()
        };

        let descriptions = diagnostics_registry();
        let sopts = build_session_options(&matches);

        let sess = build_session(sopts, None, descriptions);
        let cfg = build_configuration(&sess);
        let input = Input::Str(crate_src);
        let krate = phase_1_parse_input(&sess, cfg, &input);
        println!("{:#?}", krate);

        let src_name = source_name(&input);
        pprint(sess, &krate, src_name);


        panic!("Lõpp");
        // let exc = ext::expand::ExpansionConfig::default(cratenam);
        // let cx = ExtCtxt::new(
        //     &sess.parse_sess,
        //     cfg,
        //     exc,
        // );
        // //
        // let x = quote_expr!(&cx, 1 + 2);
        // stringify!(x);
        // // println!("{:?}", );
    }

}
