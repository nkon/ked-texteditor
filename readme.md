# KED : simple TUI text editor for rust training.

コロナウイルスで自宅に閉じ込められてしまったので、プログラミングの練習のためにテキストエディターを作ってみよう。

[Build Your Own Text Editor](https://viewsourcecode.org/snaptoken/kilo/)というチュートリアルで[kilo](https://github.com/antirez/kilo)というエディターを実装している。最初の立ち上がりはそれを参照しながらビルドアップしていくことにする。

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

###

* 上下の矢印キーで画面がスクロールできるようにした。
* バッファーの行数を数えて、オーバーランするスクロールはしないようにした。このへん、自動で出来ないのはCっぽい。
* こまめにrustfmtをかけておかなければ、フォーマットの修正もまとめてコミットされると見にくい。


