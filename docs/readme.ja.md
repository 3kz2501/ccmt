# ccmt

Claude を使った AI コミットメッセージジェネレーター。

ステージ済みの git diff から、Claude CLI または Anthropic API を使って意味のあるコミットメッセージを自動生成します。

[English](../README.md)

## 特徴

- **2つのプロバイダー** - Claude CLI（サブスクリプション）または Anthropic HTTP API
- **Conventional Commits** - `type(scope): description` 形式を自動適用
- **Gitmoji 対応** - オプションで絵文字プレフィックス
- **多言語対応** - 英語・日本語など任意の言語でメッセージ生成
- **対話式確認** - コミット前に Yes / Edit / Cancel で選択
- **Edit ループ** - 修正指示を書くと Claude が再生成
- **階層化コンフィグ** - グローバル + プロジェクト単位 + CLI フラグ
- **Git hook** - `prepare-commit-msg` で自動生成
- **Dry-run** - コミットせずにメッセージをプレビュー

## インストール

### crates.io から

```bash
cargo install ccmt
```

### ソースから

```bash
git clone https://github.com/3kz2501/ccmt.git
cd ccmt
cargo install --path .
```

### GitHub Releases から

[Releases](https://github.com/3kz2501/ccmt/releases) ページからプラットフォーム別のバイナリをダウンロードできます。

## 前提条件

- [Git](https://git-scm.com/)
- [Rust](https://www.rust-lang.org/tools/install)（ソースビルド / `cargo install` の場合）

加えて、使用するプロバイダーに応じて以下が必要です：

| プロバイダー | 必要なもの |
|-------------|-----------|
| `cli`（デフォルト） | [Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) がインストール・認証済み |
| `api` | [Anthropic API キー](https://console.anthropic.com/) |

## クイックスタート

```bash
# 変更をステージ
git add .

# メッセージ生成してコミット
ccmt

# コミットせずにプレビュー
ccmt --dry-run
```

## 使い方

```
USAGE: ccmt [OPTIONS] [COMMAND]

COMMANDS:
  config init     デフォルトのグローバル設定ファイルを生成
  config show     マージ済みの設定を表示
  hook install    prepare-commit-msg hook をインストール
  hook remove     hook をアンインストール

OPTIONS:
  -m, --message <HINT>     AI へのコンテキストヒント
  -d, --dry-run            メッセージ生成のみ（コミットしない）
      --push               コミット後に push
      --no-confirm         確認プロンプトをスキップ
      --provider <cli|api> 認証プロバイダーを一時的に上書き
      --model <MODEL>      モデルを一時的に上書き (sonnet, haiku, opus, or フルID)
      --language <LANG>    言語を一時的に上書き (en, ja, etc.)
  -h, --help               ヘルプを表示
```

### 使用例

```bash
# 基本的な使い方
ccmt

# コンテキストヒント付き
ccmt -m "認証モジュールをテスタビリティ向上のためリファクタリング"

# 日本語でコミットメッセージ
ccmt --language ja

# CLI の代わりに API を使用
ccmt --provider api

# 特定モデルで dry-run
ccmt --dry-run --model haiku

# コミット後に自動 push
ccmt --push

# 非対話モード（CI / スクリプト向け）
ccmt --no-confirm
```

## 確認フロー

`confirm = true`（デフォルト）の場合、対話式プロンプトが表示されます：

```
Generated commit message:
────────────────────────────────────────
feat(auth): JWT トークン検証を追加

ログイン・ログアウトエンドポイントと
トークン検証ミドルウェアを実装。
────────────────────────────────────────

? Commit with this message?
> Yes  - このメッセージでコミット
  Edit - 修正指示を書いて再生成
  No   - キャンセル
```

**Edit** を選ぶと修正指示を入力できます。Claude が指示に基づいてメッセージを再生成し、再度確認ループに入ります。

## 設定

ccmt は設定ファイルなしでもデフォルト値で動作します。カスタマイズしたい場合のみ設定ファイルを作成してください。

### 設定ファイルの生成

```bash
ccmt config init    # ~/.config/ccmt/config.toml を作成
ccmt config show    # マージ済みの設定を表示
```

### グローバル設定 (`~/.config/ccmt/config.toml`)

```toml
[auth]
provider = "cli"        # "cli" (Claude CLI) or "api" (HTTP API)
api_key = ""            # "api" プロバイダー使用時に必要
model = "sonnet"        # "sonnet", "haiku", "opus", またはフルモデルID

[commit]
conventional = true     # Conventional Commits 形式
emoji = false           # Gitmoji プレフィックス
language = "en"         # メッセージの言語 ("en", "ja" など)
auto_stage = false      # 生成前に `git add .` を自動実行
auto_push = false       # コミット後に `git push` を自動実行
confirm = true          # 対話式の確認プロンプト

[prompt]
system = ""             # カスタムシステムプロンプト（デフォルトに追加）
max_diff_length = 8000  # AI に送る diff の最大文字数
```

### プロジェクト設定 (`.ccmt.toml`)

プロジェクトルート（または親ディレクトリ）に `.ccmt.toml` を配置すると、プロジェクト単位で設定を上書きできます。上書きしたいフィールドのみ記載してください：

```toml
[commit]
language = "ja"
conventional = true
```

### 設定の優先順位

設定は以下の順にマージされます（後のものが優先）：

1. ハードコードされたデフォルト値
2. `~/.config/ccmt/config.toml`（グローバル）
3. `.ccmt.toml`（プロジェクト、cwd から親方向に探索）
4. CLI フラグ（最優先）

### 環境変数

| 変数名 | 説明 |
|--------|------|
| `ANTHROPIC_API_KEY` | `auth.api_key` が空の場合のフォールバック |

## プロバイダー

### `cli`（デフォルト）

[Claude Code CLI](https://docs.anthropic.com/en/docs/claude-code) をサブプロセスとして使用します。Claude CLI のインストールと認証が必要です。

```bash
ccmt --provider cli
```

### `api`

Anthropic Messages API への直接 HTTP 呼び出し。API キーが必要です。

```bash
# 設定ファイル経由
ccmt config init
# ~/.config/ccmt/config.toml を編集: provider = "api", api_key = "sk-ant-..."

# または環境変数経由
export ANTHROPIC_API_KEY="sk-ant-..."
ccmt --provider api
```

### モデルエイリアス

| エイリアス | モデル ID |
|-----------|----------|
| `sonnet` | `claude-sonnet-4-5-20250929` |
| `haiku` | `claude-haiku-4-5-20251001` |
| `opus` | `claude-opus-4-6` |

フルモデル ID の直接指定も可能です：`--model claude-sonnet-4-5-20250929`

## Git Hook

`prepare-commit-msg` hook をインストールすると、`git commit` 時にメッセージを自動生成できます：

```bash
ccmt hook install    # .git/hooks/prepare-commit-msg にインストール
ccmt hook remove     # 削除（バックアップがあれば復元）
```

hook インストール後は `git commit` を実行するだけで、ccmt が自動的にコミットメッセージを生成します。マージコミットやスカッシュコミットでは hook はスキップされます。

## プロジェクト構成

```
src/
├── main.rs              # エントリーポイント、メインフロー、確認ループ
├── cli.rs               # CLI 引数定義 (clap)
├── config.rs            # 設定の読み込み・マージ・init/show
├── git.rs               # Git サブプロセス操作
├── prompt.rs            # AI プロンプト構築
├── format.rs            # レスポンスの整形・クリーンアップ
├── hook.rs              # Git hook のインストール/アンインストール
└── provider/
    ├── mod.rs           # Provider トレイトとファクトリ
    ├── claude_cli.rs    # Claude CLI サブプロセスプロバイダー
    └── api.rs           # Anthropic HTTP API プロバイダー
```

## ライセンス

MIT
