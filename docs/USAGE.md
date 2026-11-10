# RuneDesk の使い方

RuneDesk は、runetale ネットワーク上のデバイスにリモートデスクトップ接続するアプリです。
接続先は runetale のデバイス一覧から選ぶだけで、ID やパスワードはありません。
runetale のポリシーで通信が許可されているデバイスにだけ接続できます。

## 1. 事前準備

### runetale

- 操作する側・操作される側の **両方** で runetale を起動し、ログインしておきます。
- 管理画面のポリシーで、操作する側から操作される側への **TCP 21118** を許可します。

  | 項目 | 値 |
  |---|---|
  | Source | 操作する側のデバイス / ユーザー / グループ |
  | Destination | 操作される側のデバイス |
  | Protocol | TCP |
  | Port | 21118 |

  双方向で操作したい場合は、Source と Destination の両方に両デバイスを入れてください。

### 対応 OS

| OS | 対応バージョン | インストーラ |
|---|---|---|
| macOS (Apple Silicon) | macOS 11 以降 | `RuneDesk-macos-arm64.dmg` |
| macOS (Intel) | macOS 10.15 以降 | `RuneDesk-macos-x64.dmg` |
| Windows (x64) | Windows 10 以降 | `RuneDesk-windows-x64-setup.exe` |

Windows on ARM でも x64 版がエミュレーションで動作します。

## 2. ダウンロード

[GitHub Releases](https://github.com/runetale/runedesk/releases) から、お使いの環境のインストーラを取得します
（組織から別の配布 URL が案内されている場合は、そちらを使ってください）。

- macOS (Apple Silicon): `RuneDesk-<バージョン>-macos-arm64.dmg`
- macOS (Intel): `RuneDesk-<バージョン>-macos-x64.dmg`
- Windows: `RuneDesk-<バージョン>-windows-x64-setup.exe`
- チェックサム: `SHA256SUMS`

Mac のアプリは Apple の公証済み、Windows のインストーラとプログラムは Runetale Inc. の署名付きです。

## 3. インストール

### macOS

1. dmg を開き、**RuneDesk** を **Applications** フォルダへドラッグします。
2. Applications から RuneDesk を起動します。
3. 必要な権限を **システム設定 → プライバシーとセキュリティ** で RuneDesk に許可します。
   許可したあとは RuneDesk を再起動してください。

   | 権限 | 必要な場面 |
   |---|---|
   | 画面収録 | この Mac を **操作される側** にする（画面を送る） |
   | アクセシビリティ | この Mac を **操作される側** にする（マウス・キーボード操作を受け付ける） |
   | 入力監視 | この Mac から **操作する側** としてキーボード入力を送る |

   必要な権限が足りないときは、RuneDesk のメイン画面に案内が表示されます（「Configure」から設定画面を開けます）。

Mac を操作される側にする場合は、RuneDesk を起動したままにしておいてください。

### Windows

1. `RuneDesk-windows-x64-setup.exe` を実行し、管理者として許可します。
2. `C:\Program Files\RuneDesk` にインストールされ、Windows サービスとして常駐します。
   サインイン前の画面や再起動後でも、操作される側として接続を受け付けます。
3. スタートメニューの **RuneDesk** から画面を開けます。

## 4. 接続する

1. RuneDesk を開くと、上部に **このデバイスの runetale IP** と状態（Ready）が表示されます。
2. **runetale devices** に、接続できるデバイスが一覧表示されます。
   接続したいデバイスのカードをクリックすると接続します。
3. 一覧にないデバイスは、上部の入力欄に runetale IP（例: `100.64.0.1`）を入力して **Connect** を押します。

パスワードの入力はありません。操作される側の RuneDesk が、接続元が runetale ネットワーク上の
デバイスであることを runetale に問い合わせて確認し、確認できた場合だけ接続を受け付けます。

### 接続前に確認する（Ask before connecting）

操作される側のメイン画面で **Ask before connecting** をオンにすると、接続のたびに
操作される側の画面に確認が表示され、承認したときだけ接続されます。
オフ（既定）の場合は、ポリシーで許可されたデバイスからの接続をそのまま受け付けます。

## 5. 接続中の操作

接続中の画面上部のツールバー:

- **接続先の表示**: ホスト名と runetale IP。左の丸は接続状態です。

  | 色 | 意味 |
  |---|---|
  | 緑 | 直接接続（runetale の暗号化トンネル内） |
  | 橙 | 中継経由 |
  | 灰 | 接続中 |

  ディスプレイが複数ある場合は、番号を押して切り替えます。
- **チャット**: 相手側とメッセージをやりとりします。
- **アクション（⚡）**: ファイル転送、Ctrl + Alt + Del の送信、画面ロック、再起動、画面の再読み込みなど。
- **表示**: 画面の表示方法（Shrink: ウィンドウに収める / Original: 等倍 / Stretch: 拡大して合わせる）、
  画質、コーデック、リモートカーソル表示など。
- **キーボード**: キーボードの入力モード（Map / Legacy）。

### ファイル転送

メイン画面の **Transfer file**、または接続中のアクションメニューの **Transfer file** から、
ファイル転送用のウィンドウを開きます。

### クリップボード

RuneDesk では、情報漏えい防止のためクリップボードの共有（コピー＆ペースト）を無効にしています。

## 6. アンインストール

- **macOS**: RuneDesk を終了し、Applications から RuneDesk をゴミ箱に入れます。
- **Windows**: 設定 → アプリ → インストールされているアプリ → RuneDesk → アンインストール、
  またはスタートメニューの **Uninstall RuneDesk**。

## 7. 困ったとき

| 症状 | 確認すること |
|---|---|
| デバイス一覧が空 / 「runetale is not connected」 | runetale が起動・ログイン済みか。相手のデバイスがポリシーで通信可能か |
| 「Only devices on your runetale network can connect」 | 操作される側の runetale が起動しているか。接続元の IP が runetale の IP（`100.x.x.x`）か |
| 接続できない / Connection Error を繰り返す | ポリシーで TCP 21118 が許可されているか。操作される側で RuneDesk（Windows はサービス）が動いているか |
| 画面が出ない（Mac が操作される側） | 画面収録の権限。許可後に RuneDesk を再起動したか |
| マウス・キーボードが効かない（Mac が操作される側） | アクセシビリティの権限 |
| キーボード入力が送れない（Mac から操作） | 入力監視の権限 |
| 動作が遅い | 接続状態の丸が橙（中継経由）になっていないか。runetale の直接接続が確立できているか |

ログの場所（問い合わせの際に添付してください）:

- macOS: `~/Library/Logs/RuneDesk/`
- Windows（サービス側）: `C:\Windows\ServiceProfiles\LocalService\AppData\Roaming\RuneDesk\log\`
- Windows（画面側）: `%APPDATA%\RuneDesk\log\`
