mod reel;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{self, ClearType},
};
use reel::{check_winnings, Reel, DISPLAY_SIZE, PAYLINES};
use std::io::{self, stdout};
use std::time::Duration;
use tokio::time::sleep;

struct SlotMachine {
    reels: [Reel; 3],
    last_spinning_state: [bool; 3],
}

impl SlotMachine {
    fn new() -> Self {
        Self {
            reels: [Reel::new(), Reel::new(), Reel::new()],
            last_spinning_state: [false, false, false],
        }
    }

    async fn start_all_reels(&self) {
        for reel in &self.reels {
            reel.start_spinning();
        }

        // 各リールのスピンループを並行実行
        let reel0 = self.reels[0].clone();
        let reel1 = self.reels[1].clone();
        let reel2 = self.reels[2].clone();

        tokio::spawn(async move { reel0.spin_loop().await });
        tokio::spawn(async move { reel1.spin_loop().await });
        tokio::spawn(async move { reel2.spin_loop().await });
    }

    fn stop_reel(&self, index: usize) {
        if index < 3 {
            self.reels[index].request_stop();
        }
    }

    fn has_state_changed(&mut self) -> bool {
        let current_state = [
            self.reels[0].is_spinning(),
            self.reels[1].is_spinning(),
            self.reels[2].is_spinning(),
        ];
        
        let changed = current_state != self.last_spinning_state;
        self.last_spinning_state = current_state;
        changed
    }

    fn display_reels(&self) -> io::Result<()> {
        // カーソルを指定位置に移動してから表示
        execute!(stdout(), cursor::MoveTo(0, 2))?;

        // リールの表示
        let reel_symbols: Vec<[&str; DISPLAY_SIZE]> = self
            .reels
            .iter()
            .map(|reel| reel.get_visible_symbols())
            .collect();

        // 各行を個別に出力して正確な表示を確保
        execute!(stdout(), cursor::MoveTo(0, 2))?;
        execute!(stdout(), Print("┌─────┬─────┬─────┐"))?;
        
        execute!(stdout(), cursor::MoveTo(0, 3))?;
        execute!(
            stdout(),
            Print(format!(
                "│  {}  │  {}  │  {}  │",
                reel_symbols[0][0], reel_symbols[1][0], reel_symbols[2][0]
            ))
        )?;
        
        execute!(stdout(), cursor::MoveTo(0, 4))?;
        execute!(stdout(), Print("├─────┼─────┼─────┤"))?;
        
        execute!(stdout(), cursor::MoveTo(0, 5))?;
        execute!(
            stdout(),
            Print(format!(
                "│  {}  │  {}  │  {}  │",
                reel_symbols[0][1], reel_symbols[1][1], reel_symbols[2][1]
            ))
        )?;
        
        execute!(stdout(), cursor::MoveTo(0, 6))?;
        execute!(stdout(), Print("├─────┼─────┼─────┤"))?;
        
        execute!(stdout(), cursor::MoveTo(0, 7))?;
        execute!(
            stdout(),
            Print(format!(
                "│  {}  │  {}  │  {}  │",
                reel_symbols[0][2], reel_symbols[1][2], reel_symbols[2][2]
            ))
        )?;
        
        execute!(stdout(), cursor::MoveTo(0, 8))?;
        execute!(stdout(), Print("└─────┴─────┴─────┘"))?;

        // リールの状態表示
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

        // 当選チェック
        if !self.reels.iter().any(|reel| reel.is_spinning()) {
            let winning_lines = check_winnings(&self.reels);
            if !winning_lines.is_empty() {
                execute!(stdout(), cursor::MoveTo(0, 14))?;
                execute!(stdout(), SetForegroundColor(Color::Yellow))?;
                execute!(stdout(), Print("🎉 当選! 🎉"))?;
                execute!(stdout(), cursor::MoveTo(0, 15))?;
                execute!(stdout(), Print("当選ライン: "))?;
                for line in &winning_lines {
                    execute!(stdout(), Print(format!("{} ", line + 1)))?;
                }
                execute!(stdout(), SetForegroundColor(Color::White))?;
                
                // 当選ラインの表示
                execute!(stdout(), cursor::MoveTo(0, 17))?;
                self.display_paylines(&winning_lines)?;
            } else {
                execute!(stdout(), cursor::MoveTo(0, 14))?;
                execute!(stdout(), Print("残念、ハズレです"))?;
            }
        }

        Ok(())
    }

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

    fn display_paylines(&self, winning_lines: &[usize]) -> io::Result<()> {
        execute!(stdout(), Print("有効ライン:"))?;
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

#[tokio::main]
async fn main() -> io::Result<()> {
    // ターミナルの初期化
    terminal::enable_raw_mode()?;
    execute!(stdout(), terminal::Clear(ClearType::All))?;

    let mut slot_machine = SlotMachine::new();
    
    // 初期画面表示
    slot_machine.display_initial_screen()?;

    loop {
        // 状態変化を先にチェック
        let state_changed = slot_machine.has_state_changed();
        
        // キー入力のチェック（ノンブロッキング）
        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char(' ') => {
                        slot_machine.start_all_reels().await;
                    }
                    KeyCode::Left => {
                        slot_machine.stop_reel(0);
                    }
                    KeyCode::Down => {
                        slot_machine.stop_reel(1);
                    }
                    KeyCode::Right => {
                        slot_machine.stop_reel(2);
                    }
                    KeyCode::Esc => {
                        break;
                    }
                    _ => {}
                }
            }
        }

        // リールが回転中または状態が変化した場合に画面を更新
        if slot_machine.reels.iter().any(|reel| reel.is_spinning()) || state_changed {
            slot_machine.display_reels()?;
        }

        // 少し待機
        sleep(Duration::from_millis(50)).await;
    }

    // ターミナルの復元
    terminal::disable_raw_mode()?;
    execute!(stdout(), terminal::Clear(ClearType::All))?;
    execute!(stdout(), cursor::MoveTo(0, 0))?;
    println!("ゲームを終了しました。ありがとうございました！");

    Ok(())
}
