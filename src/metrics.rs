use bencher::BencherConfig;
use bootstrap;
use collections::HashMap;
use serialize::{Decodable,json};
use std::hash::sip::SipHasher;
use std::io::{File,Truncate,Write};

pub struct Metrics {
    samples: HashMap<String, Vec<f64>, SipHasher>,
}

impl Metrics {
    pub fn new() -> Metrics {
        let p = Path::new("metrics.json");
        let map = match File::open(&p).and_then(|mut f| f.read_to_str()) {
            Err(_) => HashMap::new(),
            Ok(s) => match json::from_str(s.as_slice()) {
                Err(_) => HashMap::new(),
                Ok(j) => match Decodable::decode(&mut json::Decoder::new(j)) {
                    Err(_) => HashMap::new(),
                    Ok(m) => m,
                }
            }
        };

        Metrics {
            samples: map,
        }
    }

    fn save(&self) {
        let p = Path::new("metrics.json");
        match File::open_mode(&p, Truncate, Write) {
            Err(_) => fail!("couldn't open metrics.json"),
            Ok(mut f) => {
                let s = json::Encoder::str_encode(&self.samples);
                match f.write_str(s.as_slice()) {
                    Err(_) => fail!("couldn't write metrics.json"),
                    Ok(_) => {},
                }
            }
        }
    }

    pub fn update(&mut self,
                  name: &String,
                  sample: Vec<f64>,
                  config: &BencherConfig) {
        let old = match self.samples.swap(name.clone(), sample) {
            None => {
                println!("> storing new sample in metrics.json");
                self.save();
                return;
            },
            Some(old) => old,
        };

        let new = self.samples.find(name).unwrap();
        let old = old.as_slice();
        let new = new.as_slice();

        println!("> testing hypotheses against previous sample");
        if !bootstrap::same_population(old, new, config) {
            if bootstrap::mean_regressed(old, new, config) &
                bootstrap::median_regressed(old, new, config) {
                fail!("regression")
            }
        }

        self.save();
    }
}