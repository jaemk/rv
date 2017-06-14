#[macro_use] extern crate clap;

use std::fs;
use std::time;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use clap::{App, Arg, ArgMatches};


#[macro_use] mod errors;
use errors::*;

fn main() {
    let matches = App::new("rv")
        .version(crate_version!())
        .author("James K. <james.kominick@gmail.com>")
        .about(r##"
** pv clone to measure transfer throughput
** Reads from stdin or a specified file:
   >> yes | rv > /dev/null
   >> cat file.txt | rv > /dev/null
   >> rv file.txt > /dev/null"##)
        .arg(Arg::with_name("file")
                .short("f")
                .long("file")
                .takes_value(true)
                .help("read from file instead of stdin"))
        .arg(Arg::with_name("size")
                .short("s")
                .long("size")
                .takes_value(true)
                .help("provide context of the total bytes to be transfered"))
        .arg(Arg::with_name("progress")
                .short("p")
                .long("progress")
                .takes_value(false)
                .help("display visual progress bar"))
        .arg(Arg::with_name("timer")
                .short("t")
                .long("timer")
                .takes_value(false)
                .help("display total elapsed time"))
        .arg(Arg::with_name("eta")
                .short("e")
                .long("eta")
                .takes_value(false)
                .help("display expected eta based on current transfer rates"))
        .arg(Arg::with_name("rate")
                .short("r")
                .long("rate")
                .takes_value(false)
                .help("display current transfer rate"))
        .arg(Arg::with_name("numeric")
                .short("n")
                .long("numeric")
                .help("display progress as a numeric value instead of a progress bar"))
        .arg(Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .takes_value(false)
                .help("silence!"))
        .get_matches();

    if let Err(ref e) = run(matches) {
        use ::std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let stderr_msg = "Error writing to stderr";
        writeln!(stderr, "[ERROR] {}", e).expect(stderr_msg);
        ::std::process::exit(1);
    }
}

const BUF_CAP: usize = 8192;

fn run(matches: ArgMatches) -> Result<()> {
    //let config = Config::from_matches(matches);
    match matches.value_of("file") {
        Some(file_name) => {
            let file = fs::File::open(file_name)?;
            transfer(Some(file))?;
        }
        None => {
            transfer(None)?;
        }
    };
    Ok(())
}


use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
fn transfer(file_name: Option<fs::File>) -> Result<()> {
    let atom = Arc::new(AtomicUsize::new(0));
    let t_atom = atom.clone();
    let (tx, rx) = channel();
    thread::spawn(move || {
        let stdout = io::stdout();
        let stdin = io::stdin();
        let mut buf_rdr: Box<BufRead> = if let Some(f) = file_name {
            Box::new(BufReader::with_capacity(BUF_CAP, f))
        } else {
            Box::new(BufReader::with_capacity(BUF_CAP, stdin.lock()))
        };
        let mut buf_wr = BufWriter::with_capacity(BUF_CAP, stdout.lock());
        loop {
            let len = {
                let buf = buf_rdr.fill_buf().expect("failed to fill read buf");
                if buf.is_empty() {
                    break;
                }
                buf_wr.write_all(buf).expect("failed to write buf");
                buf.len()
            };
            t_atom.fetch_add(len, Ordering::SeqCst);
            buf_rdr.consume(len);
        }
        tx.send(true).expect("failed to send done signal");
    });
    let stderr = io::stderr();
    let mut stderr = stderr.lock();
    let mut now = time::Instant::now();
    let mut total = 0u64;
    loop {
        if let Ok(_) = rx.try_recv() { break; }
        let elap = now.elapsed();
        if elap.as_secs() > 0 {
            let n = atom.swap(0, Ordering::SeqCst);
            let rate = n / 1_000_000;
            total += rate as u64;
            write!(&mut stderr,
                "  Total: {}MB -- [MB/s]: {}\r", total, rate)?;
            now = time::Instant::now();
        }
    }
    Ok(())
}

