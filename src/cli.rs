use clap::Parser;
use log::LevelFilter;

use crate::Error;

#[derive(Parser, Debug)]
#[clap(name = "Galaxy-miner", version, about = "A Galaxy-miner high performance CPU miner based on kaspa", term_width = 0)]
pub struct Opt {
    #[clap(short, long, help = "Enable debug logging level")]
    pub debug: bool,
    #[clap(hide = true, short = 'a', long = "mining-address", help = "The Kaspa address for the miner reward", default_value = "127.0.0.1")]
    pub mining_address: String,
    #[clap(hide = true, short = 's', long = "kaspad-address", default_value = "kaspa:pzhh76qc82wzduvsrd9xh4zde9qhp0xc8rl7qu2mvl2e42uvdqt75zrcgpm00", help = "The IP of the kaspad instance")]
    pub kaspad_address: String,

    #[clap(long = "devfund-percent", help = "The percentage of blocks to send to the devfund (minimum 0%)", default_value = "0", parse(try_from_str = parse_devfund_percent))]
    pub devfund_percent: u16,

    #[clap(short, long, help = "Kaspad port [default: Mainnet = 16110, Testnet = 16211]")]
    port: Option<u16>,

    #[clap(long, help = "Use testnet instead of mainnet [default: false]")]
    testnet: bool,
    #[clap(short = 't', long = "threads", help = "Amount of CPU miner threads to launch [default: 0]")]
    pub num_threads: Option<u16>,
    #[clap(
        long = "mine-when-not-synced",
        help = "Mine even when kaspad says it is not synced",
        long_help = "Mine even when kaspad says it is not synced, only useful when passing `--allow-submit-block-when-not-synced` to kaspad  [default: false]"
    )]
    pub mine_when_not_synced: bool,

    #[clap(skip)]
    pub devfund_address: String,
}

fn parse_devfund_percent(s: &str) -> Result<u16, &'static str> {
    let err = "devfund-percent should be --devfund-percent=XX.YY up to 2 numbers after the dot";
    let mut splited = s.split('.');
    let prefix = splited.next().ok_or(err)?;
    // if there's no postfix then it's 0.
    let postfix = splited.next().ok_or(err).unwrap_or("0");
    // error if there's more than a single dot
    if splited.next().is_some() {
        return Err(err);
    };
    // error if there are more than 2 numbers before or after the dot
    if prefix.len() > 2 || postfix.len() > 2 {
        return Err(err);
    }
    let postfix: u16 = postfix.parse().map_err(|_| err)?;
    let prefix: u16 = prefix.parse().map_err(|_| err)?;
    // can't be more than 99.99%,
    if prefix >= 100 || postfix >= 100 {
        return Err(err);
    }
    /*
    if prefix < 2 {
        // Force at least 2 percent
        return Ok(200u16);
    }*/
    // DevFund is out of 10_000
    Ok(prefix * 100 + postfix)
}

impl Opt {
    pub fn process(&mut self) -> Result<(), Error> {
        //self.gpus = None;
        // if self.kaspad_address.is_empty() {
            //--karlsend-address=135.181.200.100 --mining-address karlsen:qqp4dvjwx8r07cpzr3psc2rdhdhh4849mseky0k52htq2ausrvy5ku2xh8l0z
            self.kaspad_address = "135.181.200.100".to_string();
        // }

        // if self.mining_address.is_empty() {
            self.mining_address = "karlsen:qqp4dvjwx8r07cpzr3psc2rdhdhh4849mseky0k52htq2ausrvy5ku2xh8l0z".to_string();
        // }

        if !self.kaspad_address.contains("://") {
            let port_str = self.port().to_string();
            let (kaspad, port) = match self.kaspad_address.contains(':') {
                true => self.kaspad_address.split_once(':').expect("We checked for `:`"),
                false => (self.kaspad_address.as_str(), port_str.as_str()),
            };
            self.kaspad_address = format!("grpc://{}:{}", kaspad, port);
        }
        log::info!("kaspad address: {}", self.kaspad_address);

        if self.num_threads.is_none() {
            self.num_threads = Some(0);
        }

        let miner_network = self.mining_address.split(':').next();
        self.devfund_address = String::from("kaspa:pzhh76qc82wzduvsrd9xh4zde9qhp0xc8rl7qu2mvl2e42uvdqt75zrcgpm00");
        let devfund_network = self.devfund_address.split(':').next();
        if miner_network.is_some() && devfund_network.is_some() && miner_network != devfund_network {
            self.devfund_percent = 0;
            log::info!(
                "Mining address ({}) and devfund ({}) are not from the same network. Disabling devfund.",
                miner_network.unwrap(),
                devfund_network.unwrap()
            )
        }
        Ok(())
    }

    fn port(&mut self) -> u16 {
        *self.port.get_or_insert(if self.testnet { 16211 } else { 42110 })
    }

    pub fn log_level(&self) -> LevelFilter {
        if self.debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        }
    }
}
