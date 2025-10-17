# リール回転詳細 シーケンス図

## 概要
リールの回転開始から停止までの詳細なタイムラインを表現したシーケンス図です。

## リール回転ライフサイクル

```mermaid
sequenceDiagram
    participant User as ユーザー
    participant SlotMachine as SlotMachine
    participant Reel as Reel
    participant SpinState as AtomicBool
    participant Tokio as Tokioタスク

    Note over User,Tokio: スペースキー押下からリール停止まで

    User->>SlotMachine: スペースキー押下
    
    loop 各リール (i = 0,1,2)
        SlotMachine->>Reel: start_spinning()
        Reel->>SpinState: store(true, Ordering::Relaxed)
        Note right of SpinState: 回転状態をONに設定
        
        SlotMachine->>Tokio: spawn(reel.spin_loop())
        
        loop spin_loop() - 無限ループ
            Tokio->>SpinState: load(Ordering::Relaxed)
            
            alt 回転中 (spinning == true)
                Tokio->>Reel: update_position()
                Note right of Reel: position を更新<br/>ランダムシンボル選択
                Tokio->>Tokio: sleep(100ms)
                
            else 停止要求 (spinning == false)
                Note over Tokio: ループ終了
                Tokio-->>SlotMachine: タスク完了
            end
        end
    end
    
    Note over User,Tokio: --- リール回転中 ---
    
    User->>SlotMachine: 矢印キー押下 (例: ←)
    SlotMachine->>Reel: stop_reel(0) → request_stop()
    Reel->>SpinState: store(false, Ordering::Relaxed)
    Note right of SpinState: 回転状態をOFFに設定
    
    Note over Tokio: 次のループでspinning==false検出
    Tokio->>Tokio: spin_loop()終了
    
    SlotMachine->>Reel: get_visible_symbols()
    Reel-->>SlotMachine: 最終停止シンボル
    SlotMachine-->>User: 画面更新・結果表示
```

## 状態管理の詳細

### AtomicBoolによるスレッドセーフ制御

```mermaid
graph TD
    A[start_spinning()] --> B[spinning.store(true)]
    B --> C[tokio::spawn(spin_loop)]
    C --> D{spinning.load()}
    D -->|true| E[update_position()]
    E --> F[sleep(100ms)]
    F --> D
    D -->|false| G[ループ終了]
    
    H[request_stop()] --> I[spinning.store(false)]
    I --> D
    
    style A fill:#e1f5fe
    style G fill:#ffebee
    style I fill:#fff3e0
```

## タイミング仕様

| 処理 | 間隔 | 説明 |
|------|------|------|
| **spin_loop()** | 100ms | リールシンボル更新周期 |
| **画面更新** | 50ms | メインループの更新間隔 |
| **キー入力** | 100ms | `event::poll()`のタイムアウト |

## 並行実行の可視化

```
時間軸: 0ms -----> 100ms -----> 200ms -----> 300ms
         |           |            |            |
Reel[0]: 🍎 -----> 🍌 -----> 🍇 -----> [停止]
Reel[1]: 🎲 -----> 🎯 -----> 🎪 -----> 🎭
Reel[2]: ⭐ -----> 🌟 -----> ✨ -----> 💫

ユーザー: [Space] -----> [←] -----> -----> [↓]
         全開始    左停止         中停止
```

---

*このシーケンス図は、Rustの非同期処理とAtomicBoolによるスレッドセーフな状態管理を詳細に表現しています。*