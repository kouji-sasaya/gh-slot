use rand::Rng;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::sleep;

pub const REEL_SIZE: usize = 21;
pub const DISPLAY_SIZE: usize = 3;

// リール1のシンボル（フルーツ中心）
pub const REEL1_SYMBOLS: [&str; REEL_SIZE] = [
    "🍒", "🍋", "🍊", "🍇", "🍉", "🍓", "🥝",
    "🍌", "🍑", "🍎", "🥭", "🍍", "🥥", "🍈",
    "🔔", "⭐", "💎", "🍀", "🎰", "💰", "🌹"
];

// リール2のシンボル（バランス型）
pub const REEL2_SYMBOLS: [&str; REEL_SIZE] = [
    "🍎", "🍒", "🍊", "🔔", "🍋", "⭐", "🍇",
    "💎", "🍉", "🍓", "🎰", "🥝", "🍌", "💰",
    "🍑", "🌹", "🥭", "🍍", "🥥", "🍈", "🍀"
];

// リール3のシンボル（特殊シンボル多め）
pub const REEL3_SYMBOLS: [&str; REEL_SIZE] = [
    "💎", "🔔", "⭐", "🍒", "🌹", "🎰", "💰",
    "🍋", "🍊", "🍇", "🍀", "🍉", "🍓", "🥝",
    "🍌", "🍑", "🍎", "🥭", "🍍", "🥥", "🍈"
];

#[derive(Clone)]
pub struct Reel {
    pub position: Arc<Mutex<usize>>,
    pub is_spinning: Arc<Mutex<bool>>,
    pub stop_requested: Arc<Mutex<bool>>,
    pub reel_id: usize, // リールのID（0, 1, 2）
}

impl Reel {
    pub fn new(reel_id: usize) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            position: Arc::new(Mutex::new(rng.gen_range(0..REEL_SIZE))),
            is_spinning: Arc::new(Mutex::new(false)),
            stop_requested: Arc::new(Mutex::new(false)),
            reel_id,
        }
    }

    pub fn start_spinning(&self) {
        let mut is_spinning = self.is_spinning.lock().unwrap();
        let mut stop_requested = self.stop_requested.lock().unwrap();
        *is_spinning = true;
        *stop_requested = false;
    }

    pub fn request_stop(&self) {
        let mut stop_requested = self.stop_requested.lock().unwrap();
        *stop_requested = true;
    }

    pub fn is_spinning(&self) -> bool {
        *self.is_spinning.lock().unwrap()
    }

    pub fn get_visible_symbols(&self) -> [&'static str; DISPLAY_SIZE] {
        let position = *self.position.lock().unwrap();
        let symbols = match self.reel_id {
            0 => &REEL1_SYMBOLS,
            1 => &REEL2_SYMBOLS,
            2 => &REEL3_SYMBOLS,
            _ => &REEL1_SYMBOLS, // デフォルト
        };
        
        [
            symbols[position],
            symbols[(position + 1) % REEL_SIZE],
            symbols[(position + 2) % REEL_SIZE],
        ]
    }

    pub async fn spin_loop(&self) {
        let step_duration = Duration::from_millis(100); // スピード調整：より速く回転

        loop {
            // スピン状態をチェック
            let is_spinning = *self.is_spinning.lock().unwrap();
            if !is_spinning {
                break;
            }

            // 停止要求をチェック
            if *self.stop_requested.lock().unwrap() {
                let mut is_spinning = self.is_spinning.lock().unwrap();
                *is_spinning = false;
                break;
            }

            // 1つずつ位置を進める
            {
                let mut position = self.position.lock().unwrap();
                *position = (*position + 1) % REEL_SIZE;
            }
            
            sleep(step_duration).await;
        }
    }
}

// 有効ライン（7ライン）の定義
pub const PAYLINES: [[usize; 3]; 7] = [
    [0, 0, 0], // 上段横一列
    [1, 1, 1], // 中段横一列
    [2, 2, 2], // 下段横一列
    [0, 1, 2], // 斜め下がり
    [2, 1, 0], // 斜め上がり
    [0, 1, 0], // V字
    [2, 1, 2], // 山字
];

pub fn check_winnings(reels: &[Reel; 3]) -> Vec<usize> {
    let mut winning_lines = Vec::new();
    
    let reel_symbols: Vec<[&str; DISPLAY_SIZE]> = reels
        .iter()
        .map(|reel| reel.get_visible_symbols())
        .collect();

    for (line_index, line) in PAYLINES.iter().enumerate() {
        let symbols: Vec<&str> = line
            .iter()
            .enumerate()
            .map(|(reel_index, &row)| reel_symbols[reel_index][row])
            .collect();

        // 3つのシンボルが同じかチェック
        if symbols[0] == symbols[1] && symbols[1] == symbols[2] {
            winning_lines.push(line_index);
        }
    }

    winning_lines
}