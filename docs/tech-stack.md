# AI Manager 技術スタック仕様書

## 1. 技術スタック概要

### 1.1 基本方針

- **非同期処理**: tokioランタイムによる完全非同期実装
- **イベント駆動**: ユーザ入力とシステム応答の効率的処理
- **マイクロサービス**: 各サービスの完全分離による保守性向上
- **軽量化**: バックグラウンド常駐に適した最適化

## 2. コアシステム（Rust）

### 2.1 非同期ランタイム

```toml
tokio = { version = "1.0", features = ["full"] }
```

- **tokio::spawn**: タスク並行実行
- **tokio::sync::mpsc**: サービス間チャネル通信
- **tokio::time**: タイマー、スケジューリング
- **futures**: Stream処理

### 2.2 Webフレームワーク

```toml
axum = "0.7"
tower = "0.4"
```

- RESTful API提供
- ミドルウェアサポート
- 高パフォーマンス

### 2.3 設定管理

```toml
config = "0.14"
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
```

- TOML形式の設定ファイル
- 環境別設定管理
- 型安全な設定読み込み

## 3. データベース層

### 3.1 データベース抽象化

```toml
sqlx = { version = "0.7", features = [
    "runtime-tokio-rustls", "sqlite", "postgres"
] }
sea-orm = "0.12"  # ORM選択肢
```

- **ローカル**: SQLite（軽量、個人PC向け）
- **クラウド**: PostgreSQL、MySQL対応
- **外部サービス**: クラウドDB（AWS RDS、Google Cloud SQL等）
- 非同期クエリ実行
- コンパイル時クエリ検証
- データベース選択可能な抽象化レイヤー

### 3.2 マイグレーション・ORM

```toml
sqlx-cli = "0.7"
sea-orm-cli = "0.12"  # ORM使用時
```

- スキーマバージョン管理
- 自動マイグレーション実行
- 複数DB対応マイグレーション

## 4. LLMサービス層

### 4.1 HTTP通信

```toml
reqwest = { version = "0.11", features = ["json"] }
```

- 非同期HTTP通信対応
- JSON自動シリアライゼーション
- タイムアウト・リトライ機能

### 4.2 対応LLMサービス

- **OpenAI API**: GPT-4, GPT-3.5-turbo
- **Claude API**: Claude-3
- **ローカルLLM**: Ollama等のローカル実行

## 5. 外部サービス連携

### 5.1 Google Calendar

```toml
google-calendar3 = "5.0"
yup-oauth2 = "8.0"
```

- OAuth2認証
- Calendar API v3対応
- イベントCRUD操作

### 5.2 メール処理

```toml
async-imap = "0.9"
lettre = "0.11"
```

- IMAP（受信）: 非同期メール取得
- SMTP（送信）: 非同期メール送信
- マルチアカウント対応

### 5.3 通知システム

```toml
notify-rust = "4.0"
```

- OS標準通知機能
- クロスプラットフォーム対応

## 6. UI層

### 6.1 デスクトップアプリ

```toml
tauri = "1.0"
```

- 軽量デスクトップアプリフレームワーク
- Rust-JavaScript間通信
- セキュアなAPI公開

### 6.2 フロントエンド

```json
{
  "react": "^18.0.0",
  "typescript": "^5.0.0",
  "@tauri-apps/api": "^1.0.0"
}
```

- React + TypeScript
- モダンなチャットUI
- リアルタイム更新

## 7. イベント駆動アーキテクチャ

### 7.1 メッセージパッシング

```rust
// サービス間通信
tokio::sync::mpsc::channel()
tokio::sync::broadcast::channel()
```

### 7.2 イベントバス設計

```rust
enum SystemEvent {
    UserInput(String),
    CalendarSync,
    EmailReceived(Email),
    LLMResponse(String),
}
```

## 8. 開発・運用ツール

### 8.1 開発ツール

```toml
# 開発用
tokio-test = "0.4"
mockall = "0.11"
criterion = "0.5"
```

### 8.2 ログ・監視

```toml
tracing = "0.1"
tracing-subscriber = "0.3"
sentry = "0.31"
```

### 8.3 エラーハンドリング

```toml
anyhow = "1.0"
thiserror = "1.0"
```

## 9. 将来拡張対応

### 9.1 音声処理（Phase 3）

- **音声認識**: whisper-rs
- **音声合成**: TTS API連携
- **音声UI**: tauri音声プラグイン

### 9.2 PC操作自動化（Phase 3）

- **画面操作**: enigo（キーボード・マウス制御）
- **画面認識**: image, opencv-rust
- **プロセス制御**: sysinfo
