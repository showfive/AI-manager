# AI Manager 段階的開発計画

## 1. 開発フェーズ概要

### Phase 1: コア基盤 (2-3週間)

**目標**: 基本的な非同期メッセージングとLLM連携

- 基本アーキテクチャの構築
- LLMサービス連携
- 基本的なチャットUI

### Phase 2: 外部サービス連携 (3-4週間)  

**目標**: Google Calendar・メール処理

- 外部サービスAPI連携
- データ永続化
- 実用的な機能実装

### Phase 3: 高度機能 (4-6週間)

**目標**: 自動化・音声対話

- 高度な自動化機能
- 音声インターフェース
- PC操作マクロ

## 2. Phase 1 詳細スケジュール

### Week 1: 基盤構築

#### Day 1-2: プロジェクトセットアップ

**成果物**:

- [ ] Cargo Workspace構成
- [ ] 基本的なディレクトリ構造
- [ ] CI/CD設定

**実装内容**:

```bash
# プロジェクト構造作成
cargo new ai-manager --lib
cd ai-manager
mkdir -p crates/{core,llm-service,data-service,external-service,shared}
mkdir -p ui/{src,src-tauri} config docs

# 各crateの初期化
cargo new crates/core --lib
cargo new crates/llm-service --lib
cargo new crates/data-service --lib
cargo new crates/external-service --lib
cargo new crates/shared --lib
```

#### Day 3-4: 共通型・イベントバス実装

**成果物**:

- [ ] メッセージ型定義 (`shared/src/messages.rs`)
- [ ] イベントバス基盤 (`core/src/event_bus.rs`)
- [ ] 基本的なエラーハンドリング

**実装内容**:

```rust
// shared/src/messages.rs
pub enum ServiceMessage { ... }

// core/src/event_bus.rs
pub struct EventBus { ... }
impl EventBus {
    pub async fn route_message(&self, msg: ServiceMessage) -> Result<()> { ... }
}
```

#### Day 5-7: LLMサービス実装

**成果物**:

- [ ] OpenAI API クライアント
- [ ] プロンプト管理システム
- [ ] レスポンス処理ロジック
- [ ] 基本的なテスト

**実装内容**:

```rust
// llm-service/src/openai.rs
pub struct OpenAIClient { ... }
impl LLMProvider for OpenAIClient { ... }
```

### Week 2: コアロジック

#### Day 1-3: データサービス実装

**成果物**:

- [ ] データベース抽象化レイヤー実装
- [ ] SQLite実装（初期デフォルト）
- [ ] 対話履歴管理
- [ ] ユーザープロフィール機能
- [ ] マイグレーション機能

**実装内容**:

```rust
// データベース抽象化トレイト
pub trait DatabaseConnection {
    async fn execute(&self, query: &str) -> Result<()>;
    async fn fetch_one<T>(&self, query: &str) -> Result<T>;
}

// SQLite実装例
pub struct SqliteConnection { /* ... */ }
impl DatabaseConnection for SqliteConnection { /* ... */ }

// PostgreSQL実装例（将来）
pub struct PostgresConnection { /* ... */ }
impl DatabaseConnection for PostgresConnection { /* ... */ }
```

#### Day 4-5: コアサービス実装

**成果物**:

- [ ] サービス管理機能
- [ ] ヘルスチェック機能
- [ ] 自動再起動機能
- [ ] ログ記録システム

#### Day 6-7: UI基盤実装

**成果物**:

- [ ] Tauri アプリケーションセットアップ
- [ ] React チャットUI
- [ ] Tauri-Core間通信
- [ ] 基本的なチャット機能

## 3. Phase 2 詳細スケジュール

### Week 3: Google Calendar連携

#### Day 1-3: Calendar API実装

**成果物**:

- [ ] OAuth2認証フロー
- [ ] Calendar API クライアント
- [ ] イベント読み書き機能

#### Day 4-5: スケジュール管理機能

**成果物**:

- [ ] 自動スケジューリング
- [ ] 競合検出機能
- [ ] リマインダー機能

### Week 4: メール処理

#### Day 1-3: メールクライアント実装

**成果物**:

- [ ] IMAP/SMTP設定
- [ ] メール取得・送信機能
- [ ] マルチアカウント対応

#### Day 4-5: メール処理ロジック

**成果物**:

- [ ] 自動分類機能
- [ ] 優先度判定
- [ ] 返信文案生成

### Week 5: 統合・最適化

#### Day 1-3: システム統合

**成果物**:

- [ ] 全サービス連携テスト
- [ ] エンドツーエンドテスト
- [ ] パフォーマンス測定

#### Day 4-5: 最適化・仕上げ

**成果物**:

- [ ] パフォーマンス調整
- [ ] エラー処理改善
- [ ] ドキュメント整備

## 4. MVP成功基準

### Week 1完了時点

- [ ] LLMとの基本対話が動作
- [ ] メッセージの送受信が正常動作
- [ ] 基本的なエラーハンドリング

### Week 2完了時点

- [ ] 対話履歴の保存・読み込み
- [ ] チャットUIが完全動作
- [ ] サービス管理機能が動作

### Week 3-5完了時点（MVP完成）

- [ ] Google Calendar連携が動作
- [ ] メール処理が動作
- [ ] 全体統合が正常動作
- [ ] 基本的な自動化機能

## 5. Phase 3 計画概要

### 音声対話機能

- 音声認識（Whisper）
- 音声合成（TTS API）
- 音声UI統合

### PC操作自動化

- キーボード・マウス制御
- 画面認識・OCR
- アプリケーション制御

### 高度な外部連携

- Slack/Discord統合
- より多くのメールサービス
- カスタム外部サービス対応

## 6. 品質保証

### テスト戦略

- **単体テスト**: 各サービスの独立テスト
- **統合テスト**: サービス間通信テスト
- **E2Eテスト**: 実際のユースケーステスト

### パフォーマンス指標

- **応答時間**: チャット応答 < 2秒
- **メモリ使用量**: < 100MB（常駐時）
- **CPU使用率**: < 5%（アイドル時）

### セキュリティ対策

- API認証情報の暗号化保存
- 通信の暗号化
- ログの機密情報除去
