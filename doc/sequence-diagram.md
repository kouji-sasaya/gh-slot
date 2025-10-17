# スロットマシン UMLシーケンス図

## 概要
このドキュメントは、gh-slotスロットマシンゲームのメインゲームフローを表現したUMLシーケンス図です。

## ゲームフロー シーケンス図

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant Main as main()
    participant SlotMachine as SlotMachine
    participant Reel1 as Reel[0]
    participant Reel2 as Reel[1] 
    participant Reel3 as Reel[2]
    participant Tokio as Tokioランタイム

    User->>Main: プログラム実行
    Main->>Main: terminal::enable_raw_mode()
    Main->>SlotMachine: new()
    SlotMachine-->>Main: インスタンス作成
    Main->>SlotMachine: display_initial_screen()
    SlotMachine-->>User: 初期画面表示

    loop ゲームループ
        Main->>Main: event::poll() - キー入力チェック
        
        alt スペースキー押下
            User->>Main: スペースキー入力
            Main->>SlotMachine: start_all_reels()
            
            SlotMachine->>Reel1: start_spinning()
            SlotMachine->>Reel2: start_spinning()
            SlotMachine->>Reel3: start_spinning()
            
            SlotMachine->>Tokio: spawn(reel1.spin_loop())
            SlotMachine->>Tokio: spawn(reel2.spin_loop())
            SlotMachine->>Tokio: spawn(reel3.spin_loop())
            
            par 並行実行
                Tokio->>Reel1: spin_loop() 開始
                Tokio->>Reel2: spin_loop() 開始
                Tokio->>Reel3: spin_loop() 開始
            end
            
        else 矢印キー押下（リール停止）
            User->>Main: ←/↓/→キー入力
            Main->>SlotMachine: stop_reel(index)
            SlotMachine->>Reel1: request_stop() (該当リール)
            
        else ESCキー押下
            User->>Main: ESCキー入力
            Main->>Main: break - ループ終了
        end
        
        Main->>SlotMachine: has_state_changed()
        SlotMachine-->>Main: 状態変化チェック結果
        
        alt リール回転中 or 状態変化
            Main->>SlotMachine: display_reels()
            SlotMachine->>Reel1: get_visible_symbols()
            SlotMachine->>Reel2: get_visible_symbols()
            SlotMachine->>Reel3: get_visible_symbols()
            SlotMachine-->>User: リール表示更新
        end
        
        alt 全リール停止
            SlotMachine->>SlotMachine: check_winnings()
            SlotMachine-->>User: 当選結果表示
        end
        
        Main->>Main: sleep(50ms) - 待機
    end
    
    Main->>Main: terminal::disable_raw_mode()
    Main-->>User: ゲーム終了メッセージ
```

## 設計のポイント

### 1. 非同期並行実行
- **Tokioランタイム**により3つのリールが同時に回転
- `tokio::spawn()`でタスクを並行実行
- リアルタイムなユーザー体験を実現

### 2. イベントドリブン設計
- **ノンブロッキング**なキー入力処理
- `event::poll()`によるリアルタイム応答
- ユーザー操作に即座に反応

### 3. 効率的な画面更新
- `has_state_changed()`による状態監視
- 必要な時のみ画面を再描画
- パフォーマンスの最適化

### 4. 適切なリソース管理
- ゲーム開始：`terminal::enable_raw_mode()`
- ゲーム終了：`terminal::disable_raw_mode()`
- メモリリークを防ぐ適切なクリーンアップ

## キー操作フロー

| キー | 動作 | シーケンス上の処理 |
|------|------|-----------------|
| **スペース** | 全リール回転開始 | `start_all_reels()` → 3つの`tokio::spawn()` |
| **←** | 左リール停止 | `stop_reel(0)` → `request_stop()` |
| **↓** | 中リール停止 | `stop_reel(1)` → `request_stop()` |
| **→** | 右リール停止 | `stop_reel(2)` → `request_stop()` |
| **ESC** | ゲーム終了 | ループ終了 → クリーンアップ |

## 技術仕様

- **言語**: Rust (Edition 2021)
- **非同期ランタイム**: Tokio (current_thread flavor)
- **ターミナルUI**: crossterm
- **アーキテクチャ**: 非同期イベントドリブン

---

*このシーケンス図は、gh-slot v1.7.0のアーキテクチャを表現しています。*