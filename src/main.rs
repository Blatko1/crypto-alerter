mod args;

use std::{
    fs::File,
    io::BufReader,
    ops::Sub,
    time::{Duration, Instant},
};

use binance::{api::Binance, market::Market, model::SymbolPrice};
use rodio::{Decoder, OutputStream, OutputStreamHandle, Source};

const UPDATE_INTERVAL: Duration = Duration::from_millis(1000);

fn main() {
    // ====================== ARGUMENT PROCESSING ======================
    let matches = args::parse_args();

    let symbol = matches.get_one::<String>("symbol").unwrap();

    let triggers: Vec<f64> = matches.get_many::<f64>("price_trigger").unwrap().copied().collect();

    // ========================= PREPARE ALERTS =========================
    let file = File::open(
        std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("alert_sound.mp3")
            .to_str()
            .to_owned()
            .unwrap()
            .trim_start_matches("\\\\?\\"),
    )
    .unwrap();
    let market = Market::new(None, None);
    let price = market.get_price(symbol).unwrap();
    /*let buffer = BufReader::new(file);
    let alerter = Alerter::new(buffer);

    // ====================== PREPARE THE TRIGGERS ======================
    let market = Market::new(None, None);
    let price = market.get_price(symbol).unwrap();

    // Split trigger prices into two arrays for easier comparing.
    // If the trigger is the same as the current price it will get filtered
    let higher: Vec<&f64> = triggers.iter().filter(|&&t| t > price.price).collect();
    let lower: Vec<&f64> = triggers.iter().filter(|&&t| t < price.price).collect();

    // ========================== MAIN LOOP ==========================
    let mut time = Instant::now();
    loop {
        let elapsed = time.elapsed();

        if elapsed >= UPDATE_INTERVAL {
            let last_price = market.get_price(symbol).unwrap();

            if !higher.is_empty() {
                if last_price.price > **higher.first().unwrap() {
                    alerter.output_alert(last_price);
                }
            } else if !lower.is_empty() {
                if last_price.price < **lower.first().unwrap() {}
            }

            time = Instant::now();
        } else {
            std::thread::sleep(UPDATE_INTERVAL.sub(elapsed));
        }
    }*/
}

struct Alerter {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    source: Decoder<BufReader<File>>,
}

impl Alerter {
    fn new(buffer: BufReader<File>) -> Self {
        let (stream, stream_handle) = OutputStream::try_default().unwrap();
        let source = Decoder::new(buffer).unwrap();
        Self {
            stream,
            stream_handle,
            source,
        }
    }

    fn output_alert(&self, price: SymbolPrice) {}
}
