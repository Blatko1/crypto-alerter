mod cli;

use std::{
    fs::File,
    io::BufReader,
    ops::Sub,
    path::Path,
    time::{Duration, Instant}, sync::{mpsc::{channel, Receiver}, Arc},
};

use anyhow::Result;
use binance::{api::Binance, market::Market, model::SymbolPrice, errors::Result as BinanceResult};
use rodio::{source::Buffered, Decoder, OutputStream, OutputStreamHandle, Sink, Source};

const UPDATE_INTERVAL: Duration = Duration::from_millis(1500);

fn main() {
    // ====================== ARGUMENT PROCESSING ======================
    let matches = cli::parse_args();

    let symbol = matches.get_one::<String>("symbol").unwrap();
    let mut triggers: Vec<f64> = matches
        .get_many::<f64>("price_trigger")
        .unwrap()
        .copied()
        .collect();
    triggers.sort_by(|a, b| a.partial_cmp(b).unwrap());

    // ========================= PREPARE ALERTS =========================
    let alerter = match matches.get_one::<String>("sfx") {
        Some(path) => Alerter::with_sound(path).expect("Error at reading sound file"),
        None => Alerter::new(),
    };

    // ====================== PREPARE THE TRIGGERS ======================
    let market = Arc::new(Market::new(None, None));
    let price = market.get_price(symbol).unwrap();
    let mut live_price = LivePrice::new(market.clone(), price.symbol);

    // Split trigger prices into two arrays for easier comparing.
    // If the trigger is the same as the current price it will get filtered
    let mut higher: Vec<&f64> = triggers.iter().filter(|&&t| t > price.price).collect();
    let mut lower: Vec<&f64> = triggers.iter().filter(|&&t| t < price.price).collect();
    println!("lower: {:?}, higher{higher:?}", lower);
    // ========================== MAIN LOOP ==========================
    let mut time = Instant::now();
    loop {
        let elapsed = time.elapsed();

        if elapsed >= UPDATE_INTERVAL {
            // Add to different thread Price parsing
            let last_price = live_price.get_live_price();

            if !higher.is_empty() {
                if last_price >= **higher.first().unwrap() {
                    alerter
                        .output_alert(
                            symbol.clone(),
                            **higher.first().unwrap(),
                            TriggerCause::BreakAboveEq,
                        )
                        .unwrap();
                    higher.remove(0);
                }
            }
            if !lower.is_empty() {
                if last_price <= **lower.last().unwrap() {
                    alerter
                        .output_alert(
                            symbol.clone(),
                            **lower.last().unwrap(),
                            TriggerCause::BreakBelowEq,
                        )
                        .unwrap();
                    lower.pop();
                }
            }

            time = Instant::now();
        } else {
            println!("slep {} {}", UPDATE_INTERVAL.sub(elapsed).as_secs(), elapsed.as_millis());
            std::thread::sleep(UPDATE_INTERVAL.sub(elapsed));
        }
        println!("now");
    }
}

struct LivePrice {
    last_price: f64,
    receiver: Receiver<BinanceResult<SymbolPrice>>
}

impl LivePrice {
    fn new(market: Arc<Market>, symbol: String) -> Self {
        Self {
            last_price: f64::NAN,
            receiver: Self::spawn_price_tracker(market, symbol),
        }
    }

    fn get_live_price(&mut self) -> f64 {
        if let Ok(price) = self.receiver.try_recv() {
            self.last_price = price.unwrap().price;
        }

        self.last_price
    }

    fn spawn_price_tracker(market: Arc<Market>, symbol: String) -> Receiver<BinanceResult<SymbolPrice>> {
        let (tx, rx) = channel();

        std::thread::spawn(move || loop {
            let price = market.get_price(&symbol);

            match tx.send(price) {
                Ok(_) => std::thread::sleep(UPDATE_INTERVAL),
                Err(err) => panic!("Live Price Thread Error {err}"),
            }
        });

        rx
    }
}

struct Alerter {
    player: Option<SoundPlayer>,
}

impl Alerter {
    fn new() -> Self {
        Self { player: None }
    }

    fn with_sound<P: AsRef<Path>>(path: P) -> Result<Self> {
        Ok(Self {
            player: Some(SoundPlayer::new(path)?),
        })
    }

    fn output_alert(&self, symbol: String, trigger_price: f64, cause: TriggerCause) -> Result<()> {
        if let Some(player) = &self.player {
            player.repeat_sound()?;
        }

        println!("❗❗❗ NEW ALERT for {symbol} ❗❗❗");

        // BOLD And Italic the messages
        let message = match cause {
            TriggerCause::BreakAboveEq => format!(
                "> The price of {symbol} broke over the \
                    trigger price {trigger_price}!!!"
            ),

            TriggerCause::BreakBelowEq => format!(
                "> The price of {symbol} broke below the \
                trigger price {trigger_price}!!!"
            ),
        };

        println!("{}", message);

        Ok(())
    }

    fn stop_sound(&self) {
        self.player.as_ref().expect("No Sound provided!").stop();
    }
}

struct SoundPlayer {
    source: SoundSource,
    _stream: OutputStream,
    _stream_handle: OutputStreamHandle,
    sink: Sink,
}

impl SoundPlayer {
    fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = File::open(
            std::env::current_exe()?
                .parent()
                .unwrap()
                .join(path)
                .to_str()
                .unwrap()
                .trim_start_matches("\\\\?\\"),
        )?;
        let source = Decoder::new(BufReader::new(file))?.buffered();
        let (_stream, _stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&_stream_handle)?;
        Ok(Self {
            source,
            _stream,
            _stream_handle,
            sink,
        })
    }

    fn play_sound(&self) -> Result<()> {
        self.sink.append(self.source.clone());
        Ok(())
    }

    fn repeat_sound(&self) -> Result<()> {
        self.sink.append(self.source.clone().repeat_infinite());
        Ok(())
    }

    fn stop(&self) {
        self.sink.stop()
    }
}

type SoundSource = Buffered<Decoder<BufReader<File>>>;

enum TriggerCause {
    BreakAboveEq,
    BreakBelowEq,
}
