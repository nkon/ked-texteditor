# KED : simple TUI(Terminal User Interface) text editor for rust training.

コロナウイルスで自宅に閉じ込められてしまったので、プログラミングの練習のためにテキストエディタを作ってみよう。

## 目標としている機能・実装

* 練習・教材用として十分な小ささ。

記事のネタになる。また、作っていて飽きない。ひと篭もりするプロジェクトとしててきせつなサイズ。

昔から、プログラマなら作ってみたいもの。（1）コンパイラ（2）OS（3）エディタ。コンパイラについては以前に書いた。

* ターミナル上で普通に使えるテキストエディタ。
* 日本語変換機能を自前で持つ（それも書いてみたい）。
* 移植性があること。Linux、Windowsターミナル、RaspberryPiのターミナルで同様に動作する。

これら3つの特徴によって、どこに持って行っても最低限のテキストファイル編集機能が手に入る。標準の開発環境は Ubuntuだが、muslなどを使って、MacOS、Windows、Windows(MSL)、RaspberryPiなどの環境で、COPY&RUNの環境が便利だと思っている。

日本語変換機能については、技術的な興味が大きいこと。また、AIの業務応用としてどれだけの成果を上げられるのかに興味があることから実装してみようと思う。


* 設定変更機能を持つ（そういったものをハードコーディングせずにインタプリタ化する）。
* キーボードマクロを持つ。自動化テストにも役に立つだろう。

なんとなく現代的だね。


[Build Your Own Text Editor](https://viewsourcecode.org/snaptoken/kilo/)というチュートリアルで[kilo](https://github.com/antirez/kilo)というエディタを実装している。最初の立ち上がりはそれを参照しながらビルドアップしていくことにする。


## 実装上のポイント

テキストエディタを実装する時は、常に話題となるような事柄がいくつある。

### RAWモード

* CUI
* TUI
* GUI


TUIアプリケーションの基礎となるのがターミナルが解釈するエスケープシーケンスだ。特殊な文字列を出力することによって端末を自由自在にコントロールすることができる。Cでは`curses`、それの派生である`ncuurse`、`termio`というライブラリが有名である。Rustで使えるTUIライブラリを調べた。termionはわりと薄く動作が理解しやすわりに依存ライブラリが無くてPure Rustだ。今回はこれを選定した。

### 表示ウィンドウ

* オブジェクト志向的なクラスライブラリ
* 実装を頑張って、依存関係が少ないモジュール群

いずれにせよ、操作はある程度抽象化しておかなければ、あとで死ぬ。

### バッファーの持ち方

テキストエディタの基本概念はこんなものだろうか。概念としてはemacsの設計に完結しているだろう。

* ファイルをバッファに展開し、バッファに対して操作をし、バッファほファイルに保存する。ただし、ファイルと関連付けられないバッファも存在する。
* バッファに対する操作点はカーソルと言われる。ユーザはカーソルに対して操作を行う。
* ユーザは、バッファが投影されたウィンドウを操作対象とする。

バッファの実装方法としてはいくつか提案されている。基本的には編集対象をmmapして、メモリ操作が十分に早ければどれを選んでも良い。しかし、巨大な、しばしばメモリに収まりきらないような大きなファイルを編集する時も、速度に関するトレードオフが発生するのでさまざまなデータ構造が考案されてきた。


* 行リスト。
* Gap Buffer。
* Rope。

簡易的にはツールキットのテキストウィジェットを使えば、テキストエディタは簡単に実装できる。しかし、その内側では適切な実装を工夫しているのだ。本稿は教育用という趣旨もあるので、実装について、すこし詳しく話す。

#### 行リスト

代表的な実装例はvi。

基本的にメモリマップ。表示構造と実装構造が対応しているのでわかりやすい。コレ以外の実装は、この行リストではダメだかラ考え出されたもの。行リストで不自由しなければ、行リストでよい。

#### Gap Buffer

代表的な実装例はEmacs。

頻繁な操作は軽く、希な操作は重くても良い。

#### Rope

代表例はVS Code。

データベースの研究と関係がある。巨大なデータをハンドリングすることがでｋる。


## 日本語の取り扱い

### 表示（UTF-8）

### IME

## 開発の進め方

### ビルドアップスタイル

### 開発環境

VS Codeの上で開発したのだが、非常に便利。

* 編集中に文法エラーがあると赤の波線で指定してくれる。マウスカーソルを合わせると、エラー原因と対策が表示されるので、それに合わせて修正する。
* 問題リストがクリアになったら`cargo run`。
* Rustは、フォーマット、命名規則、コメントの書き方などのコーディングルールが公式で決まっていて、ツールもサポートされている。「自転車置き場な議論」の余地なく、最良のプラクティスが手に入る。

* VS CodeにはGitが統合されているので、こまめにコミットする作業が苦にならない。

* LLDBなどのデバッガは使わなかった。printfデバッグをメイン。

* 日本語のメモは「テキスト校正くん」という拡張で、即時訂正しながら。読みやすい文章にしたい。

## アーキテクチャの設計

APIは実装＆リファクタリングを進めていくうえで、自然に整理されていくだろう。しかし、一度も経験がないままで、このようなAPIを構築できるだろうか。結果から見れば当然と思うようなAPI集であっても、実装していく中で、抜け漏れの発見や引数の調整などが頻発する。

インクリメンタルな開発と実装優先順位のトレードオフのような上位設計川から見ても、実装＆テストでの抜け漏れから見ても、実装能力を持たない管理者によるウォーターフローモデルの破綻は明らかだと思う。あまり、実感として理解されていないように思うが。

## テスト

エディタはコーナーケースが多い。
自動化テストを行いたい。
ユニットテスト（内部）と結合テスト（外部）。ユニットテストはrustの標準機能を活用。結合テストのためにはマクロ機能が必要。

### ユニットテスト

Rustの言語機能に依存する部分が多いが、適切なAPI化やテストハーネスを使えば、Cでも同等のことはできるはずだ。

### 結合テスト

言語レベルで統合テストはサポートされていない。統合テストを実施するための、アーキテクチャ設計が必要だ。本プログラムではマクロ機能。ユーザ機能としても使えるがテストに使いやすいように、というのが設計の主眼だ。APIもそれに合わせて設計される。


## log

開発経緯については、GitHubのコミットログを参照していただきたい。その中でもマイルストーンとなるコミットについては[log.md](log.md) に記録しておく。これは実作業の記録なので、まとめとしては、この文書のほうがまとまっている。ビルドアップスタイルを追体験したいなら、コミットをたどっていってもおもしろいかもしれない。

[log.mg](log.md)

