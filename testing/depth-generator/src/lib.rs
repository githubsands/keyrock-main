use market_objects::DepthUpdate;
use rand::{thread_rng, Rng};
use rand_distr::{Distribution, Normal};
use tracing::info;

extern crate rand;

#[derive(Clone)]
pub struct DepthMessageGenerator {
    pub volume: f64,
    pub price: f64,
    pub vol_std: f64,
    pub price_std: f64,
    pub level_diff: f64,
}

impl DepthMessageGenerator {
    pub fn new(
        initial_volume: f64,
        initial_price: f64,
        vol_std: f64,
        price_std: f64,
        level_diff: f64,
    ) -> DepthMessageGenerator {
        DepthMessageGenerator {
            volume: initial_volume,
            price: initial_price,
            vol_std,
            price_std,
            level_diff,
        }
    }
    pub fn depth_message_random(&mut self) -> DepthUpdate {
        let kind: u8;
        let location: u8;
        let _ = match rand::random() {
            true => {
                kind = 0;
            }
            false => {
                kind = 1;
            }
        };
        let _ = match rand::random() {
            true => {
                location = 1;
            }
            false => {
                location = 2;
            }
        };

        let normal = Normal::new(0.0, 1.0).unwrap();
        let volume_diff = normal.sample(&mut thread_rng()) * self.vol_std;
        let price_diff = normal.sample(&mut thread_rng()) * self.price_std;
        let volume = (self.volume + volume_diff).max(0.0);
        let price: f64;
        match kind {
            0 => {
                price = self.price - price_diff;
            }
            1 => {
                price = self.price + price_diff;
            }
            _ => {
                panic!()
            }
        };
        DepthUpdate {
            k: kind,
            p: DepthMessageGenerator::round_to_hundreth(price as f64),
            q: volume as f64,
            l: location,
        }
    }
    pub fn depth_message(&mut self, location: u8, is_ask: bool) -> DepthUpdate {
        let kind: u8 = if is_ask { 0 } else { 1 };
        let normal = Normal::new(0.0, 1.0).unwrap();
        let volume_diff = normal.sample(&mut rand::thread_rng()) * self.vol_std;
        let price_diff = normal.sample(&mut rand::thread_rng()) * self.price_std;
        let volume = (self.volume + volume_diff).max(0.0);
        let price = if is_ask {
            self.price - price_diff
        } else {
            self.price + price_diff
        };
        DepthUpdate {
            k: kind,
            p: DepthMessageGenerator::round_to_hundreth(price as f64),
            q: volume as f64,
            l: location,
        }
    }
    pub fn depth_bulk(
        &mut self,
        exchange_locations: u8,
        depth_size_one_sided: u8,
    ) -> (Vec<DepthUpdate>, Vec<DepthUpdate>) {
        let mut rng = rand::thread_rng();

        let asks = vec![0; depth_size_one_sided as usize];
        let bids = vec![0; depth_size_one_sided as usize];
        let asks: Vec<DepthUpdate> = asks
            .iter()
            .map(|_| self.depth_message(rng.gen_range(1..=exchange_locations), true))
            .collect();
        let bids: Vec<DepthUpdate> = bids
            .iter()
            .map(|_| self.depth_message(rng.gen_range(1..=exchange_locations), false))
            .collect();
        (asks, bids)
    }
    fn round_to_hundreth(n: f64) -> f64 {
        (n * 100.0).round() / 100.0
    }
    fn round(x: f64, decimals: u32) -> f64 {
        let y = 10i64.pow(decimals) as f64;
        (x * y).round() / y
    }
    pub fn depth_balanced_orderbook(
        &mut self,
        depth: usize,
        exchange_locations: usize,
        mid_price: usize,
    ) -> (Vec<DepthUpdate>, Vec<DepthUpdate>) {
        let mut asks: Vec<DepthUpdate> = vec![];
        let mut bids: Vec<DepthUpdate> = vec![];
        for i in 1..=exchange_locations {
            let mut current_level: f64 = mid_price as f64;
            for _ in mid_price..=mid_price + depth {
                current_level =
                    DepthMessageGenerator::round_to_hundreth(current_level - self.level_diff);
                let mut depth_update = DepthUpdate::default();
                depth_update.k = 0;
                depth_update.p = current_level;
                depth_update.q = 1 as f64;
                depth_update.l = i as u8;
                asks.push(depth_update);
            }
        }
        for i2 in 1..=exchange_locations {
            let mut current_level: f64 = mid_price as f64;
            for _ in (mid_price - depth)..=mid_price {
                current_level = DepthMessageGenerator::round_to_hundreth(
                    current_level - self.level_diff as f64,
                );
                let mut depth_update = DepthUpdate::default();
                depth_update.k = 1;
                depth_update.p = current_level;
                depth_update.q = 1 as f64;
                depth_update.l = i2 as u8;
                bids.push(depth_update);
            }
        }
        (asks, bids)
    }
    pub fn depth_imbalanced_orderbook_ask(
        depth: usize,
        exchange_num: usize,
        mid_price: usize,
    ) -> (Vec<DepthUpdate>, Vec<DepthUpdate>) {
        let mut asks: Vec<DepthUpdate> = vec![];
        let mut bids: Vec<DepthUpdate> = vec![];
        for i2 in 1..exchange_num {
            for j2 in mid_price..mid_price - 2 * depth {
                let mut depth_update = DepthUpdate::default();
                depth_update.k = 1;
                depth_update.p = j2 as f64;
                depth_update.q = 1 as f64;
                depth_update.l = i2 as u8;
                bids.push(depth_update);
            }
        }
        for i in 1..exchange_num {
            for j in mid_price..mid_price + depth {
                let mut depth_update = DepthUpdate::default();
                depth_update.k = 1;
                depth_update.p = j as f64;
                depth_update.q = 1 as f64;
                depth_update.l = i as u8;
                asks.push(depth_update);
            }
        }
        (asks, bids)
    }
    pub fn depth_imbalanced_orderbook_bid(
        depth: usize,
        exchange_num: usize,
        mid_price: usize,
    ) -> (Vec<DepthUpdate>, Vec<DepthUpdate>) {
        let mut asks: Vec<DepthUpdate> = vec![];
        let mut bids: Vec<DepthUpdate> = vec![];
        let mut current_level: f64;
        for i in 0..exchange_num {
            for j in mid_price..mid_price + depth * 2 {
                let mut depth_update = DepthUpdate::default();
                depth_update.k = 0;
                depth_update.p = j as f64; // update this
                depth_update.q = 1 as f64;
                depth_update.l = i as u8;
                asks.push(depth_update);
            }
        }
        for i in 0..exchange_num {
            for j in mid_price..mid_price - depth {
                let mut depth_update = DepthUpdate::default();
                depth_update.k = 1;
                depth_update.p = j as f64;
                depth_update.q = 1 as f64;
                depth_update.l = i as u8;
                bids.push(depth_update);
            }
        }
        (asks, bids)
    }
}

impl Default for DepthMessageGenerator {
    fn default() -> Self {
        Self {
            volume: 400.0,
            price: 2700.0,
            vol_std: 200.0,
            price_std: 0.1,
            level_diff: 0.01,
        }
    }
}
