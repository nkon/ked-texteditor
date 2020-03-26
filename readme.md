# KED : simple TUI text editor for rust training.

コロナウイルスで自宅に閉じ込められてしまったので、プログラミングの練習のためにテキストエディターを作ってみよう。

[Build Your Own Text Editor](https://viewsourcecode.org/snaptoken/kilo/)というチュートリアルで[kilo](https://github.com/antirez/kilo)というエディターを実装している。最初の立ち上がりはそれを参照しながらビルドアップしていくことにする。

### c480b785ea391faab3abbecc93670481e33ea59a

* `cargo init --bin` でプロジェクトを初期化した。

### 

* 画面制御の方法について、いろいろ調べた結果termionを使うことにした。練習のために、手書きでエスケープシーケンスを出力してもいいが、ここは効率化のためにライブラリを使ってもいいと考えた。依存関係が少ないpure rustで、高速と評判のtermionにした。termionはわりと制御コードと一対一に対応しているので、隠蔽感が無く、勉強としても適していると思う。
* termionの練習問題を入力して、raw modeでの文字入力、エスケースシーケンスを使った画面描画ができることを確認した。