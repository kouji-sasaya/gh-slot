// === 外部ライブラリのインポート ===
use rand::Rng;                        // ランダム数生成（リールの初期位置決定用）
use std::sync::{Arc, Mutex};          // スレッドセーフな共有データ用（複数スレッドで安全にデータを共有）
use std::time::Duration;              // 時間間隔の指定用
use tokio::time::sleep;               // 非同期での待機処理用

// === スロットマシンの基本設定 ===
pub const REEL_SIZE: usize = 21;      // 各リールのシンボル総数（21個の絵文字）
pub const DISPLAY_SIZE: usize = 3;    // 画面に表示される縦のシンボル数（3個）

// === 各リールのシンボル配列定義 ===
// 注意：各リールは異なるシンボル配列を持つため、当選確率が調整されています

// リール1のシンボル（左リール）
// 配列のインデックス0〜20に対応する21個の絵文字
pub const REEL1_SYMBOLS: [&str; REEL_SIZE] = [
    "⭐", "💯", "🏀", "🍀", "🏀", "🍀", "🎩",     // インデックス 0〜6
    "🍒", "🍀", "🏀", "🍀", "💯", "⚪", "🍀",     // インデックス 7〜13
    "🏀", "🍀", "🍒", "🎩", "🍀", "🏀", "🍀"      // インデックス 14〜20
];

// リール2のシンボル（中リール）
// 左リールとは異なる配列で、ゲームバランスを調整
pub const REEL2_SYMBOLS: [&str; REEL_SIZE] = [
    "🏀", "💯", "🍀", "🍒", "🏀", "⭐", "🍀",     // インデックス 0〜6
    "🍒", "🏀", "🎩", "🍀", "🍒", "🏀", "⭐",     // インデックス 7〜13
    "🍀", "🍒", "🏀", "🎩", "🍀", "🍒", "⚪"      // インデックス 14〜20
];

// リール3のシンボル（右リール）
// 右リール専用の配列で、当選ラインの成立確率を制御
pub const REEL3_SYMBOLS: [&str; REEL_SIZE] = [
    "🍀", "💯", "🎩", "⭐", "🏀", "🍀", "⚪",     // インデックス 0〜6
    "⭐", "🏀", "🍀", "⚪", "⭐", "🏀", "🍀",     // インデックス 7〜13
    "⚪", "⭐", "🏀", "🍀", "⚪", "⭐", "🏀"      // インデックス 14〜20
];

// === Reel構造体の定義 ===
// #[derive(Clone)] により、この構造体はコピー（クローン）が可能になる
// 複数のスレッドで同じリールデータを安全に共有するために必要
#[derive(Clone)]
pub struct Reel {
    // Arc<Mutex<T>>について：
    // Arc = Atomically Reference Counted：複数スレッドで安全に共有可能
    // Mutex = Mutual Exclusion：同時アクセスを防ぎ、データ競合を回避
    
    pub position: Arc<Mutex<usize>>,        // 現在のリール位置（0〜20の範囲）
    pub is_spinning: Arc<Mutex<bool>>,      // 回転中かどうかのフラグ
    pub stop_requested: Arc<Mutex<bool>>,   // 停止要求が出されたかのフラグ
    pub reel_id: usize,                     // リールのID（0=左, 1=中, 2=右）
}

impl Reel {
    /// 新しいリールインスタンスを作成
    /// 
    /// # 引数
    /// * `reel_id` - リールのID（0=左, 1=中, 2=右）
    /// 
    /// # 戻り値
    /// 初期化されたReelインスタンス（ランダムな開始位置を持つ）
    pub fn new(reel_id: usize) -> Self {
        let mut rng = rand::thread_rng();  // ランダム数生成器を取得
        Self {
            // Arc::new(Mutex::new(値)) でスレッドセーフな共有データを作成
            position: Arc::new(Mutex::new(rng.gen_range(0..REEL_SIZE))), // 0〜20のランダム位置
            is_spinning: Arc::new(Mutex::new(false)),                    // 初期状態は停止
            stop_requested: Arc::new(Mutex::new(false)),                 // 停止要求なし
            reel_id,                                                     // リールIDを保存
        }
    }

    /// リールの回転を開始
    /// この関数は複数スレッドから安全に呼び出し可能
    pub fn start_spinning(&self) {
        // .lock().unwrap() でMutexロックを取得（他スレッドのアクセスをブロック）
        let mut is_spinning = self.is_spinning.lock().unwrap();
        let mut stop_requested = self.stop_requested.lock().unwrap();
        
        // デリファレンス演算子 * で実際の値を変更
        *is_spinning = true;        // 回転開始フラグをON
        *stop_requested = false;    // 停止要求をリセット
    } // スコープを抜ける時にMutexロックが自動的に解放される

    /// リールの停止要求を発行
    /// 実際の停止はspin_loop内で処理される
    pub fn request_stop(&self) {
        let mut stop_requested = self.stop_requested.lock().unwrap();
        *stop_requested = true;  // 停止要求フラグをON
    }

    /// リールが現在回転中かどうかを確認
    /// 
    /// # 戻り値
    /// true: 回転中, false: 停止中
    pub fn is_spinning(&self) -> bool {
        *self.is_spinning.lock().unwrap()  // 現在の回転状態を返す
    }

    /// 画面に表示される3つのシンボルを取得
    /// リールの現在位置から連続する3つのシンボルを返す
    /// 
    /// # 戻り値
    /// [上段, 中段, 下段] の順でシンボルが格納された配列
    pub fn get_visible_symbols(&self) -> [&'static str; DISPLAY_SIZE] {
        let position = *self.position.lock().unwrap();  // 現在のリール位置を取得
        
        // リールIDに応じて対応するシンボル配列を選択
        let symbols = match self.reel_id {
            0 => &REEL1_SYMBOLS,    // 左リール
            1 => &REEL2_SYMBOLS,    // 中リール  
            2 => &REEL3_SYMBOLS,    // 右リール
            _ => &REEL1_SYMBOLS,    // 想定外のIDの場合はデフォルトで左リール
        };
        
        // 現在位置から連続する3つのシンボルを配列で返す
        // % REEL_SIZE で配列の境界を超えた場合に先頭に戻る（循環）
        [
            symbols[position],                           // 上段：現在位置
            symbols[(position + 1) % REEL_SIZE],        // 中段：次の位置
            symbols[(position + 2) % REEL_SIZE],        // 下段：その次の位置
        ]
    }

    /// リールの回転処理メインループ（非同期関数）
    /// tokio::spawn()によって別タスクで実行される
    /// 回転中は一定間隔でリール位置を更新し続ける
    pub async fn spin_loop(&self) {
        let step_duration = Duration::from_millis(35); // 回転速度：35ミリ秒間隔（1周0.74秒）
        
        loop {  // 無限ループ（停止条件で抜ける）
            // === 回転状態の確認 ===
            // スコープブロック内でMutexロックを取得・即座に解放
            let is_spinning = *self.is_spinning.lock().unwrap();
            if !is_spinning {
                break;  // 回転していなければループ終了
            }

            // === 停止要求の確認 ===
            if *self.stop_requested.lock().unwrap() {
                // 停止要求があった場合、回転状態をfalseに変更
                let mut is_spinning = self.is_spinning.lock().unwrap();
                *is_spinning = false;
                break;  // ループ終了
            }

            // === リール位置の更新 ===
            {
                // スコープブロックでMutexロックの取得期間を制限
                let mut position = self.position.lock().unwrap();
                // 位置を1つ進める（21に達したら0に戻る循環処理）
                *position = (*position + 1) % REEL_SIZE;
            } // ここでMutexロックが解放される
            
            // === 待機処理 ===
            // 非同期待機：他のタスクに実行権を譲りながら100ms待機
            sleep(step_duration).await;
        } // ループ終了時にリール停止完了
    }
}

// 有効ライン（5ライン）の定義
pub const PAYLINES: [[usize; 3]; 5] = [
    [0, 0, 0], // 上段横一列
    [1, 1, 1], // 中段横一列
    [2, 2, 2], // 下段横一列
    [0, 1, 2], // 斜め下がり
    [2, 1, 0], // 斜め上がり
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