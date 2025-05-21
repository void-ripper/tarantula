use std::{collections::HashMap, error::Error, sync::Arc, time::Duration};

use headless_chrome::{Browser, LaunchOptions, Tab};
use k256::elliptic_curve::rand_core::OsRng;
use serde::{Deserialize, Serialize};
use tracing::Level;
use ureq::http::Uri;

mod config;

#[derive(Deserialize)]
struct NextWork {
    url: String,
}

#[derive(Serialize)]
pub struct ScrapResult {
    url: String,
    keywords: HashMap<String, u32>,
    links: Vec<String>,
}

fn scrap(cfg: config::Config, tab: Arc<Tab>, public: Box<[u8]>) -> Result<(), Box<dyn Error>> {
    let next_uri: Uri = format!("{}/next-work/{}", &cfg.api, hex::encode(&public)).parse()?;
    let scrap_uri: Uri = format!("{}/scrap-result", &cfg.api).parse()?;

    loop {
        let res: NextWork = ureq::post(next_uri).send("")?.body_mut().read_json()?;

        tab.navigate_to(&res.url)?;
        tab.wait_until_navigated()?;
        let links = tab.find_elements("a")?;
        let links = links
            .iter()
            .filter_map(|n| n.get_attribute_value("href").ok().flatten())
            .collect();

        let scrap = ScrapResult {
            url: res.url.clone(),
            keywords: HashMap::new(),
            links,
        };

        ureq::post(scrap_uri).send_json(scrap)?;

        break;
    }

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let cfg = config::load("scrapper.toml")?;

    tracing_subscriber::fmt().with_env_filter(&cfg.log).init();

    let opts = LaunchOptions::default_builder()
        .headless(false)
        .path(Some(cfg.browser.clone()))
        .build()?;
    let b = Browser::new(opts)?;
    let private = k256::SecretKey::random(&mut OsRng);
    let public = private.public_key().to_sec1_bytes();

    for _ in 0..cfg.parallel {
        let tab = b.new_tab()?;
        let cfg = cfg.clone();
        let public = public.clone();

        std::thread::spawn(move || {
            if let Err(e) = scrap(cfg, tab, public) {
                println!("scrap: {}", e);
            }
        });
    }
    std::thread::sleep(Duration::from_secs(3));

    Ok(())
}
