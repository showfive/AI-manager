# AI Manager プロジェクト構造設計

## 1. 全体構造

```text
ai-manager/
├── Cargo.toml              # Workspace設定
├── README.md               # プロジェクト概要
├── CLAUDE.md              # Claude Code用ガイド
├── .gitignore
├── .github/                # GitHub Actions設定
│   └── workflows/
│       ├── ci.yml
│       └── release.yml
│
├── crates/                 # Rustクレート群
│   ├── core/               # コアサービス
│   ├── llm-service/        # LLM連携サービス
│   ├── data-service/       # データ永続化サービス
│   ├── external-service/   # 外部サービス連携
│   └── shared/             # 共通型・ユーティリティ
│
├── ui/                     # Tauriデスクトップアプリ
│   ├── src-tauri/          # Tauriバックエンド
│   ├── src/                # Reactフロントエンド
│   ├── package.json
│   └── tauri.conf.json
│
├── config/                 # 設定ファイル
│   ├── default.toml
│   ├── development.toml
│   └── production.toml
│
├── docs/                   # ドキュメント
│   ├── requirements.md
│   ├── tech-stack.md
│   ├── architecture.md
│   ├── development-plan.md
│   ├── project-structure.md
│   └── api.md
│
├── scripts/                # 開発・デプロイスクリプト
│   ├── setup.sh
│   ├── build.sh
│   └── test.sh
│
└── tests/                  # 統合テスト
    ├── integration/
    └── e2e/
```

## 2. ルートレベル設定

### 2.1 Cargo.toml (Workspace設定)

```toml
[workspace]
members = [
    "crates/core",
    "crates/llm-service", 
    "crates/data-service",
    "crates/external-service",
    "crates/shared",
    "ui/src-tauri"
]

[workspace.dependencies]
# 非同期ランタイム
tokio = { version = "1.0", features = ["full"] }
futures = "0.3"

# シリアライゼーション
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"

# HTTP クライアント
reqwest = { version = "0.11", features = ["json"] }

# データベース（複数DB対応）
sqlx = { version = "0.7", features = [
    "runtime-tokio-rustls", "sqlite", "postgres"
] }
sea-orm = { version = "0.12", optional = true }

# エラーハンドリング
anyhow = "1.0"
thiserror = "1.0"

# ログ・監視
tracing = "0.1"
tracing-subscriber = "0.3"

# 設定管理
config = "0.14"

# 時間処理
chrono = { version = "0.4", features = ["serde"] }

# UUID生成
uuid = { version = "1.0", features = ["v4"] }
```

## 3. Cratesの詳細構造

### 3.1 Core Service

```text
crates/core/
├── Cargo.toml
├── src/
│   ├── lib.rs              # クレートエントリポイント
│   ├── main.rs             # バイナリエントリポイント
│   ├── event_bus.rs        # イベントバス実装
│   ├── service_manager.rs  # サービス管理
│   ├── config.rs           # 設定管理
│   ├── health.rs           # ヘルスチェック
│   └── handlers/           # メッセージハンドラー
│       ├── mod.rs
│       ├── user_input.rs
│       ├── llm_response.rs
│       └── system_events.rs
└── tests/
    ├── integration.rs
    └── event_bus_test.rs
```

**Cargo.toml**:

```toml
[package]
name = "ai-manager-core"
version = "0.1.0"
edition = "2021"

[dependencies]
ai-manager-shared = { path = "../shared" }
ai-manager-llm-service = { path = "../llm-service" }
ai-manager-data-service = { path = "../data-service" }
ai-manager-external-service = { path = "../external-service" }

tokio = { workspace = true }
serde = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
config = { workspace = true }
```

### 3.2 LLM Service

```text
crates/llm-service/
├── Cargo.toml
├── src/
│   ├── lib.rs              # クレートエントリポイント
│   ├── provider.rs         # LLMプロバイダートレイト
│   ├── openai.rs           # OpenAI実装
│   ├── claude.rs           # Claude実装
│   ├── local.rs            # ローカルLLM実装
│   ├── prompt_manager.rs   # プロンプト管理
│   └── usage_tracker.rs    # 使用量追跡
└── tests/
    ├── openai_test.rs
    └── prompt_test.rs
```

### 3.3 Data Service

```text
crates/data-service/
├── Cargo.toml
├── src/
│   ├── lib.rs              # クレートエントリポイント
│   ├── connection/         # データベース抽象化
│   │   ├── mod.rs
│   │   ├── trait.rs        # DatabaseConnection trait
│   │   ├── sqlite.rs       # SQLite実装
│   │   ├── postgres.rs     # PostgreSQL実装
│   │   └── factory.rs      # 接続ファクトリ
│   ├── models/             # データモデル
│   │   ├── mod.rs
│   │   ├── conversation.rs
│   │   ├── user_profile.rs
│   │   └── settings.rs
│   ├── repositories/       # リポジトリパターン
│   │   ├── mod.rs
│   │   ├── conversation_repo.rs
│   │   └── profile_repo.rs
│   └── migrations/         # マイグレーション管理
│       ├── mod.rs
│       ├── sqlite/
│       └── postgres/
├── migrations/             # SQLマイグレーション
│   ├── sqlite/
│   │   ├── 001_initial.sql
│   │   └── 002_user_profiles.sql
│   └── postgres/
│       ├── 001_initial.sql
│       └── 002_user_profiles.sql
└── tests/
    ├── repository_test.rs
    ├── sqlite_test.rs
    └── postgres_test.rs
```

### 3.4 External Service

```text
crates/external-service/
├── Cargo.toml
├── src/
│   ├── lib.rs              # クレートエントリポイント
│   ├── calendar/           # カレンダー連携
│   │   ├── mod.rs
│   │   ├── google.rs
│   │   └── events.rs
│   ├── email/              # メール処理
│   │   ├── mod.rs
│   │   ├── imap_client.rs
│   │   ├── smtp_client.rs
│   │   └── processor.rs
│   ├── notifications/      # 通知機能
│   │   ├── mod.rs
│   │   └── os_notifications.rs
│   └── auth/               # 認証管理
│       ├── mod.rs
│       ├── oauth2.rs
│       └── token_manager.rs
└── tests/
    ├── calendar_test.rs
    └── email_test.rs
```

### 3.5 Shared

```text
crates/shared/
├── Cargo.toml
├── src/
│   ├── lib.rs              # クレートエントリポイント
│   ├── messages.rs         # メッセージ型定義
│   ├── types.rs            # 共通型定義
│   ├── errors.rs           # エラー型定義
│   ├── utils.rs            # ユーティリティ関数
│   └── constants.rs        # 定数定義
└── tests/
    └── messages_test.rs
```

## 4. UI層構造

### 4.1 Tauri Backend

```text
ui/src-tauri/
├── Cargo.toml
├── src/
│   ├── main.rs             # Tauriメイン
│   ├── commands.rs         # Tauriコマンド
│   ├── state.rs            # アプリケーション状態
│   └── events.rs           # イベントハンドリング
├── tauri.conf.json         # Tauri設定
├── build.rs                # ビルドスクリプト
└── icons/                  # アプリアイコン
```

### 4.2 React Frontend

```text
ui/src/
├── App.tsx                 # メインアプリ
├── components/             # Reactコンポーネント
│   ├── Chat/
│   │   ├── ChatWindow.tsx
│   │   ├── MessageInput.tsx
│   │   └── MessageList.tsx
│   ├── Settings/
│   │   ├── SettingsPanel.tsx
│   │   └── ServiceConfig.tsx
│   └── Common/
│       ├── Loading.tsx
│       └── ErrorBoundary.tsx
├── hooks/                  # カスタムフック
│   ├── useChat.ts
│   ├── useSettings.ts
│   └── useTauri.ts
├── types/                  # TypeScript型定義
│   ├── api.ts
│   └── ui.ts
├── styles/                 # CSS/SCSS
│   ├── global.css
│   └── components/
└── utils/                  # ユーティリティ
    ├── api.ts
    └── format.ts
```

## 5. 設定・ドキュメント構造

### 5.1 設定ファイル

```text
config/
├── default.toml            # デフォルト設定
├── development.toml        # 開発環境設定
└── production.toml         # 本番環境設定
```

### 5.2 スクリプト

```text
scripts/
├── setup.sh                # 初期セットアップ
├── build.sh                # ビルドスクリプト
├── test.sh                 # テスト実行
├── dev.sh                  # 開発サーバー起動
└── release.sh              # リリースビルド
```

## 6. 開発ワークフロー

### 6.1 開発コマンド

```bash
# 開発環境セットアップ
./scripts/setup.sh

# 開発サーバー起動
./scripts/dev.sh

# テスト実行
./scripts/test.sh

# ビルド
./scripts/build.sh
```

### 6.2 依存関係管理

- Workspace依存関係で統一バージョン管理
- 各crateは最小限の依存関係を持つ
- 循環依存の回避（shared crateで共通型を管理）
