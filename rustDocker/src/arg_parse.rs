use getopts::Options;


fn print_usage(opts: Options) {
    let brief = format!("Usage: mini_docker [options]");
    print!("{}", opts.usage(&brief));
}


pub fn process_options(args: &Vec<String>) -> Option<String>{
    //set-up options
    let mut opts = Options::new();
    opts.optopt("r", "run", "run a busybox cmd", "");
    opts.optflag("h", "help", "print this help menu");

    // parse options from args
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!("{}",f) }
    };

    // print help message
    if matches.opt_present("h") {
        print_usage(opts);
        return None;
    }

    // check for run option
    if matches.opt_str("r").is_none() {
        print_usage(opts);
        return None;
    }

    return matches.opt_str("r");
}
