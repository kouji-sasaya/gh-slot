// リールモジュールをインポート（同じディレクトリのreel.rsファイル）
mod reel;

// クロスターミナルライブラリから必要な機能をインポート
// これらはターミナル操作（画面クリア、カーソル移動、色設定など）に使用
use crossterm::{
    cursor,                                                    // カーソル移動機能
    event::{self, Event, KeyCode, KeyEvent},                  // キーボード入力イベント処理
    execute,                                                   // ターミナルコマンド実行マクロ
    style::{Color, Print, SetForegroundColor},                // 色設定と文字出力
    terminal::{self, ClearType},                              // ターミナル制御（画面クリアなど）
};
// リールモジュールから必要な関数と構造体をインポート
use reel::{check_winnings, Reel, DISPLAY_SIZE, PAYLINES};
// 標準ライブラリから入出力と時間機能をインポート
use std::io::{self, stdout};                                  // 入出力エラー処理と標準出力
use std::time::Duration;                                      // 時間間隔指定
// 非同期処理のためのtokioライブラリから時間待機機能をインポート
use tokio::time::sleep;

/// スロットマシン全体を管理する構造体
/// 3つのリールと前回の回転状態を保持
struct SlotMachine {
    reels: [Reel; 3],                // 3つのリールを配列で管理
    last_spinning_state: [bool; 3],  // 前回の各リールの回転状態（状態変化検出用）
}

impl SlotMachine {
    /// 新しいスロットマシンインスタンスを作成
    /// 各リールには0, 1, 2のIDを割り当て
    fn new() -> Self {
        Self {
            reels: [Reel::new(0), Reel::new(1), Reel::new(2)],
            last_spinning_state: [false, false, false],
        }
    }

    /// 全てのリールの回転を開始する非同期関数
    /// 各リールを並行して回転させるために非同期タスクを作成
    async fn start_all_reels(&mut self) {
        // 全リールの回転開始フラグを設定
        for reel in &self.reels {
            reel.start_spinning();
        }

        // 各リールのスピンループを並行実行
        // tokio::spawnで各リールを独立したタスクとして実行
        for reel in &self.reels {
            let reel_clone = reel.clone();
            
            // リール識別付きでタスクを作成
            tokio::spawn(async move { 
                reel_clone.spin_loop().await;
            });
        }
    }

    /// 指定されたインデックスのリールを停止する
    /// index: 停止するリールの番号（0:左, 1:中央, 2:右）
    fn stop_reel(&self, index: usize) {
        if index < 3 {
            self.reels[index].request_stop();
        }
    }

    /// リールの回転状態が前回と変化したかを確認
    /// 表示の更新が必要かどうかを判断するために使用
    fn has_state_changed(&mut self) -> bool {
        // 現在の回転状態を取得
        let current_state = [
            self.reels[0].is_spinning(),
            self.reels[1].is_spinning(),
            self.reels[2].is_spinning(),
        ];
        
        // 前回の状態と比較
        let changed = current_state != self.last_spinning_state;
        // 今回の状態を保存
        self.last_spinning_state = current_state;
        changed
    }

    /// リールの表示を行う関数
    /// スロットマシンの見た目をターミナルに描画
    fn display_reels(&self) -> io::Result<()> {
        // カーソルを指定位置に移動してから表示
        execute!(stdout(), cursor::MoveTo(0, 2))?;

        // 各リールから現在表示すべきシンボルを取得
        let reel_symbols: Vec<[&str; DISPLAY_SIZE]> = self
            .reels
            .iter()
            .map(|reel| reel.get_visible_symbols())
            .collect();

        // 各行を個別に出力して正確な表示を確保
        execute!(stdout(), cursor::MoveTo(0, 2))?;
        execute!(stdout(), Print("┌────┬────┬────┐"))?;
        
        execute!(stdout(), cursor::MoveTo(0, 3))?;
        execute!(
            stdout(),
            Print(format!(
                "│ {} │ {} │ {} │",
                reel_symbols[0][0], reel_symbols[1][0], reel_symbols[2][0]
            ))
        )?;
        
        execute!(stdout(), cursor::MoveTo(0, 4))?;
        execute!(stdout(), Print("├────┼────┼────┤"))?;
        
        execute!(stdout(), cursor::MoveTo(0, 5))?;
        execute!(
            stdout(),
            Print(format!(
                "│ {} │ {} │ {} │",
                reel_symbols[0][1], reel_symbols[1][1], reel_symbols[2][1]
            ))
        )?;
        
        execute!(stdout(), cursor::MoveTo(0, 6))?;
        execute!(stdout(), Print("├────┼────┼────┤"))?;
        
        execute!(stdout(), cursor::MoveTo(0, 7))?;
        execute!(
            stdout(),
            Print(format!(
                "│ {} │ {} │ {} │",
                reel_symbols[0][2], reel_symbols[1][2], reel_symbols[2][2]
            ))
        )?;
        
        execute!(stdout(), cursor::MoveTo(0, 8))?;
        execute!(stdout(), Print("└────┴────┴────┘"))?;

        // リールの状態表示（各リールが回転中か停止中かを表示）
        let status: Vec<String> = self
            .reels
            .iter()
            .enumerate()
            .map(|(i, reel)| {
                if reel.is_spinning() {
                    format!("リール{}: 回転中", i + 1)
                } else {
                    format!("リール{}: 停止", i + 1)
                }
            })
            .collect();

        execute!(stdout(), cursor::MoveTo(0, 10))?;
        execute!(stdout(), Print(&status[0]))?;
        execute!(stdout(), cursor::MoveTo(0, 11))?;
        execute!(stdout(), Print(&status[1]))?;
        execute!(stdout(), cursor::MoveTo(0, 12))?;
        execute!(stdout(), Print(&status[2]))?;

        // 結果表示エリアをクリア
        execute!(stdout(), cursor::MoveTo(0, 14))?;
        execute!(stdout(), terminal::Clear(ClearType::FromCursorDown))?;

        // 当選チェック（全リール停止時のみ）
        if !self.reels.iter().any(|reel| reel.is_spinning()) {
            let winning_lines = check_winnings(&self.reels);
            if !winning_lines.is_empty() {
                // 当選時の表示
                execute!(stdout(), cursor::MoveTo(0, 14))?;
                execute!(stdout(), SetForegroundColor(Color::Yellow))?;
                execute!(stdout(), Print("🎉 当選! 🎉"))?;
                execute!(stdout(), cursor::MoveTo(0, 15))?;
                execute!(stdout(), Print("当選ライン: "))?;
                for line in &winning_lines {
                    execute!(stdout(), Print(format!("{} ", line + 1)))?;
                }
                execute!(stdout(), SetForegroundColor(Color::White))?;
                
                // 当選ラインの詳細表示
                execute!(stdout(), cursor::MoveTo(0, 17))?;
                self.display_paylines(&winning_lines)?;
            } else {
                // ハズレ時の表示
                execute!(stdout(), cursor::MoveTo(0, 14))?;
                execute!(stdout(), Print("残念、ハズレです"))?;
            }
        }

        Ok(())
    }

    /// 初期画面を表示する関数
    /// ゲーム開始時にタイトル、リール、操作説明を表示
    fn display_initial_screen(&self) -> io::Result<()> {
        // 画面をクリアして初期表示
        execute!(stdout(), terminal::Clear(ClearType::All))?;
        execute!(stdout(), cursor::MoveTo(0, 0))?;

        // タイトル
        execute!(stdout(), Print("🎰 スロットマシン 🎰"))?;

        // リール表示
        self.display_reels()?;

        // 操作説明を下部に表示（固定位置）
        execute!(stdout(), cursor::MoveTo(0, 20))?;
        execute!(stdout(), Print("操作方法:"))?;
        execute!(stdout(), cursor::MoveTo(0, 21))?;
        execute!(stdout(), Print("スペースキー: 全リール回転開始"))?;
        execute!(stdout(), cursor::MoveTo(0, 22))?;
        execute!(stdout(), Print("←キー: 左リール停止"))?;
        execute!(stdout(), cursor::MoveTo(0, 23))?;
        execute!(stdout(), Print("↓キー: 中リール停止"))?;
        execute!(stdout(), cursor::MoveTo(0, 24))?;
        execute!(stdout(), Print("→キー: 右リール停止"))?;
        execute!(stdout(), cursor::MoveTo(0, 25))?;
        execute!(stdout(), Print("ESCキー: ゲーム終了"))?;

        Ok(())
    }

    /// 有効ラインの表示
    /// 当選ライン情報を画面に表示する
    fn display_paylines(&self, winning_lines: &[usize]) -> io::Result<()> {
        execute!(stdout(), Print("有効ライン:"))?;
        // 全てのペイラインを表示し、当選したラインをマークする
        for (i, line) in PAYLINES.iter().enumerate() {
            let status = if winning_lines.contains(&i) { "🎯" } else { "  " };
            execute!(stdout(), cursor::MoveTo(0, (18 + i) as u16))?;
            execute!(
                stdout(),
                Print(format!(
                    "{} ライン{}: [{}, {}, {}]",
                    status,
                    i + 1,
                    line[0] + 1,
                    line[1] + 1,
                    line[2] + 1
                ))
            )?;
        }
        Ok(())
    }
}

/// メイン関数
/// スロットマシンゲームのエントリーポイント
/// 非同期実行とターミナル制御を行う
#[tokio::main]
async fn main() -> io::Result<()> {
    // ターミナルの初期化
    // raw_modeにすることで、キー入力を即座に検知できるように設定
    terminal::enable_raw_mode()?;
    execute!(stdout(), terminal::Clear(ClearType::All))?;

    // スロットマシンのインスタンスを作成
    let mut slot_machine = SlotMachine::new();
    
    // 初期画面表示
    slot_machine.display_initial_screen()?;

    // メインゲームループ
    loop {
        // 状態変化を先にチェック
        // リールの回転状態が変わったかを確認
        let state_changed = slot_machine.has_state_changed();
        
        // キー入力のチェック（ノンブロッキング）
        // 100ミリ秒間待機してキー入力があるかをチェック
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    // スペースキー、全角スペース、その他の空白文字で全リール開始
                    KeyCode::Char(c) if c == ' ' || c == '\u{3000}' || c.is_whitespace() => {
                        slot_machine.start_all_reels().await;
                    }
                    // 左矢印キーで左リール停止
                    KeyCode::Left => {
                        slot_machine.stop_reel(0);
                    }
                    // 下矢印キーで中央リール停止
                    KeyCode::Down => {
                        slot_machine.stop_reel(1);
                    }
                    // 右矢印キーで右リール停止
                    KeyCode::Right => {
                        slot_machine.stop_reel(2);
                    }
                    // ESCキーでゲーム終了
                    KeyCode::Esc => {
                        break;
                    }
                    // その他のキーは無視
                    _ => {}
                }
            }
        }

        // リールが回転中または状態が変化した場合に画面を更新
        if slot_machine.reels.iter().any(|reel| reel.is_spinning()) || state_changed {
            slot_machine.display_reels()?;
        }

        // 少し待機（CPUの負荷を下げるため）
        sleep(Duration::from_millis(50)).await;
    }

    // ターミナルの復元
    // ゲーム終了時にターミナルを元の状態に戻す
    terminal::disable_raw_mode()?;
    execute!(stdout(), terminal::Clear(ClearType::All))?;
    execute!(stdout(), cursor::MoveTo(0, 0))?;
    println!("ゲームを終了しました。ありがとうございました！");

    Ok(())
}
