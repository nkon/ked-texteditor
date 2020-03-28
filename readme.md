# KED : simple TUI text editor for rust training.

コロナウイルスで自宅に閉じ込められてしまったので、プログラミングの練習のためにテキストエディタを作ってみよう。

## 目標としている機能・実装

* 練習・教材用として十分な小ささ。

記事のネタになる。また、作っていて飽きない。

* ターミナル上で普通に使えるテキストエディタ。
* 日本語変換機能を自前で持つ（それも書いてみたい）。
* 移植性があること。Linux、Windowsターミナル、RaspberryPiのターミナルで同様に動作する。

これら3つの特徴によって、どこに持って行っても最低限のテキストファイル編集機能が手に入る。

* 設定変更機能を持つ（そういったものをハードコーディングせずにインタプリタ化する）。
* キーボードマクロを持つ。自動化テストにも役に立つだろう。

なんとなく現代的だね。


[Build Your Own Text Editor](https://viewsourcecode.org/snaptoken/kilo/)というチュートリアルで[kilo](https://github.com/antirez/kilo)というエディタを実装している。最初の立ち上がりはそれを参照しながらビルドアップしていくことにする。



## 実装上のポイント

### RAWモード

### 表示ウィンドウ

### バッファーの持ち方

### 開発環境

VS Codeの上で開発したのだが、非常に便利。

* 編集中に文法エラーがあると赤の波線で指定してくれる。マウスカーソルを合わせると、エラー原因と対策が表示されるので、それに合わせて修正する。
* 問題リストがクリアになったら`cargo run`。
* Rustは、フォーマット、命名規則、コメントの書き方などのコーディングルールが公式で決まっていて、ツールもサポートされている。「自転車置き場な議論」の余地なく、最良のプラクティスが手に入る。

* VS CodeにはGitが統合されているので、こまめにコミットする作業が苦にならない。

* 日本語のメモは「テキスト校正くん」という拡張で、即時訂正しながら。読みやすい文章にしたい。


### c480b785ea391faab3abbecc93670481e33ea59a

* `cargo init --bin` でプロジェクトを初期化した。

### 6a7eb5dd24b4da8fe8d846da5725b393ed2865ed

* 画面制御の方法について、いろいろ調べた結果termionを使うことにした。練習のために、手書きでエスケープシーケンスを出力してもいいが、ここは効率化のためにライブラリを使ってもいいと考えた。依存関係が少ないpure rustで、高速と評判のtermionにした。termionはわりと制御コードと一対一に対応しているので、隠蔽感が無く、勉強としても適していると思う。
* termionの練習問題を入力して、raw modeでの文字入力、エスケースシーケンスを使った画面描画ができることを確認した。

### c72d8108bf393a1b9ba1a5f4509b0afee2c0039e

* オプションを解析してファイルを表示できるようにする。
* オプションパーサとしては、単機能だが`getopts`を採用。たぶん、これで事足りると思う。
* ファイル名を引数として起動されると、ファイルを表示するようにした。
* Ctrl-cで終了。


### 0752c9c702d3f1a16ba2525721c5039483b06d5d

* 端末のサイズを取得して、（縦方向は）その範囲のみ描画するようにした。
* ついでに、ファイルの読み込みと描画の関数を分けた。

### aa30a136190b01b1d84711dd9cf72383c75f0776

* さらにウィンドウへの描画の関数を分離した。
* 早速、借用チェッカにいろいろ怒られた。

### 7b52809d4475bef53497315c673171d15a6726ff

* 上下の矢印キーで画面がスクロールできるようにした。
* バッファーの行数を数えて、オーバーランするスクロールはしないようにした。このへん、自動で出来ないのはCっぽい。
* こまめにrustfmtをかけておかなければ、フォーマットの修正もまとめてコミットされると見にくい。

### 428b0c69b140faced2a051cfe491fd8743cdd8fc

* AlternateScreenを使うようにした。終了時に、元の画面が復元される。
* スクロールキーを、↑↓からPageUp/PageDownにした。矢印キーをカーソル移動にしたかったため。
* ウィンドウの右で折り返さないようにした（単に表示しない）。

### 4aafe3469abbc337b274a606f34becf70bea8e51

* 矢印キーでカーソルが動くようにした。
* オーバーランの対策は未。

### c948ae03f530334b54dedf7496d12dc2a444baed

* Screen, Window, EditBufferの構造体を導入。ただし、型や借用はむちゃくちゃ。なんとかコンパイラエラーにならずに、動くようになっただけ。
* 続けて、リファクタリングも実施。
* 少しずつ、エラー処理や、カーソルの移動制限（ありえないカーソル位置に行かない）を導入。
* 無理に引数にすると借用で混乱するので、メンバーにできるところはメンバー関数にしたほうがいい。

### d6e1c7e7364c4635a26304f10bec799d8250491d

* 編集の第一歩、簡単な文字の置換を実装した。
* 文字列操作が効率的ではない。エラーチェックも未。
* 配列を使うとRustでもランタイムエラーが頻発する。範囲チェックをきちんと行う。

### e340570bb206f9708641ae232efbdb49c7c91dd2

* 文字の挿入を実装した。

### 

* カーソルが端に来た時は自動でスクロールするようにした。
* 境界条件に関するバグをいくつか修正。

