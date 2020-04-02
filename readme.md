# KED : simple TUI(Terminal User Interface) text editor for rust training.

コロナウイルスで自宅に閉じ込められてしまったので、プログラミングの練習のためにテキストエディタを作ってみよう。

## 目標としている機能・実装

* 練習・教材用として十分な小ささ。

記事のネタになる。また、作っていて飽きない。ひと篭もりするプロジェクトとして適切なサイズ。

昔から、プログラマなら作ってみたいもの。(1)コンパイラ (2))OS (3)エディタ。コンパイラについては以前に書いた。

* ターミナル上で普通に使えるテキストエディタ。
* 日本語変換機能を自前で持つ（それも書いてみたい）。
* 移植性があること。Linux、Windowsターミナル、RaspberryPiのターミナルで同様に動作する。

これら3つの特徴によって、どこに持って行っても最低限のテキストファイル編集機能が手に入る。muslを使えば共有ライブラリの影響も極力減らすことができる。macOS、Windows、Windows(MSL)、RaspberryPiなどの環境で、COPY&RUNの環境が便利だと思っている。

日本語変換機能については、増井氏が簡易的な実装を提案していて、技術的な興味が大きい。

また、時間があれば、次の様な発展的な目標も検討したい。

* 設定変更機能を持つ（そういったものをハードコーディングせずにインタプリタ化する）。
* キーボードマクロを持つ。自動化テストにも役に立つだろう。。

## 実装上のポイント

テキストエディタを実装する時は、常に話題となるような事柄がいくつある。

### RAWモード

用語の定義として、CUI=コマンド・ユーザ・インターフェイス。シェルのプロンプトにコマンドやオプション列を入力して、結果が帰ってくるもの。TUI=ターミナル・ユーザ・インターフェイス。ターミナル上で動作するが、viなどのようにターミナルの全画面を使ってインタラクティブに操作するアプリケーション。

現状を見ていると、DockerやSSH経由などでのヘッドレス環境、WSLのようなターミナルがメインの環境が増えてくる。VS Codeのリモート開発的なものも活用されるだろう。しかし、簡易的には、そういった環境ではTUIが、環境の制約と操作性を両立するために便利だ。GUIだとWidgetの制約があったりして、大きくて移植性がないことが多い。TUIはエスケープ・シーケンスを元にしているので、小さくて移植性がいいのも良い。

TUIアプリケーションの基礎となるのが、ターミナルが解釈するエスケープシーケンスだ。特殊な文字列を出力することによって端末をコントロールすることができる。Cでは`curses`、それの派生である`ncuurse`、`termio`というライブラリが有名である。Rustで使えるTUIライブラリを調べた。termionはわりと薄く、動作を理解しやすわりに依存ライブラリが無くてPure Rustだ。今回はこれを選定した。

[termion](https://github.com/redox-os/termion)

端末をRAWモードにすれば、キー入力に対するエコーバックや改行入力の特別扱いがなくなり、アプリケーションでキー入力を直接ハンドリングすることができる。出力も、エコーバックが無くなるのでアプリケーションから自前で制御しなければならない。文字出力だけでなくエスケープシーケンスを出力することで、カーソル移動、色の変更などもできる。アプリ終了時にはCOOKEDモードに戻してあげよう。

すべての入力がアプリに吸い取られるので、アプリがキー入力で自発的に終了できるようにしておかないと、終了手段がなくなる。

基本的にはこれだけでTUIアプリケーションを作ることができる。


### 基本的なファイルビューワ

インクリメンタル開発での初期のコードをコメント付きで。だいたい雰囲気がわかると思う。

```rust
use std::io::{stdin, stdout, Write};
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::*;

use getopts::Options;
use std::env;

use std::fs::File;
use std::io::{BufRead, BufReader};

use std::str;

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options] FILE", program);
    print!("{}", opts.usage(&brief));
}

fn run_viewer_with_file(file_name: &str) {

    let mut lines = Vec::<String>::new();

    // ファイルを読み込んで行の配列に格納する。
    for result in BufReader::new(File::open(file_name).unwrap()).lines() {
        lines.push(result.unwrap().clone());
    }

    let stdin = stdin();
    let mut stdout = stdout().into_raw_mode().unwrap();        // RAWモードにする。
    write!(stdout, "{}{}", clear::All, cursor::Hide).unwrap(); // 画面をクリア、カーソル消去
    write!(stdout, "{}", cursor::Goto(1, 1)).unwrap();         // カーソルを左上(1,1)に移動
    stdout.flush().unwrap();

    let mut y = 0 as usize;

    for l in lines {                                            // カーソルを行頭に移動して行を出力
        write!(stdout, "{}{}",
            cursor::Goto(1, y as u16 +1),
            l,
        ).unwrap();
        y = y+1;
        stdout.flush().unwrap();
    }

    for c in stdin.keys() {                                       // キー入力ハンドラ
        match c {
            Ok(event::Key::Ctrl('c')) => break,                   // Ctrl-Cで終了
            _ => {}
        }
    }
    write!(stdout, "{}", termion::cursor::Show).unwrap();         // もとに戻す
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();                                 // getopts でオプション解析
    opts.optflag("h", "help", "print this help");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, opts);
        return;
    }
    if matches.free.is_empty() {
        print_usage(&program, opts);
    } else {                                                         // 引数で与えられたファイルを表示する
        let input_file_name = matches.free[0].clone();
        run_viewer_with_file(&input_file_name);
    }
}
```

### 表示ウィンドウ

エスケープ・シーケンスは端末の左上を(1,1)とした座標で動作する。一方、TUI上にはメインの表示領域の他にステータスバーや、ポップアップなどを出したくなるだろう。端末全体をルート・ウィンドウとしたうえで、子ウインドウを定義して、システマチックに座標計算できるようにしておいたほうが便利だろう。

### バッファーの持ち方

テキストエディタの基本概念は次のようなものになるだろう。

* ファイルをバッファに展開し、バッファに対して操作をし、バッファほファイルに保存する。ただし、ファイルと関連付けられないバッファも存在する。
* バッファに対する操作点はカーソルと言われる。カーソルを移動したり、カーソル位置に文字を挿入したり、カーソル位置の文字を削除したり。
* エディタは複数のバッファを持つことができる。

バッファの実装方法としてはいくつか提案されている。メモリ操作が十分に大きくて早ければどれを選んでも良い。しかし、巨大な、しばしばメモリに収まりきらないような大きなファイルを編集する時には、速度に関するトレードオフが発生するのでさまざまなデータ構造が考案されてきた。

* 行リスト
* Gap Buffer
* Rope

簡易的にはツールキットのテキストウィジェットを使えば、テキストエディタは簡単に実装できる。しかし、その内側では適切な実装を工夫しているのだ。

今回の実装は行リストで行った。しかし、バッファへの編集操作が抽象化されていれば、性能に問題が発生した場合に実装を入れ替えることが可能となる。

#### 行リスト

代表的な実装例はvi。

「バッファは行が集まったもの」として実装される。行は一般的には文字の配列または文字列だ。

バッファ全体は行（へのポインタ:行のサイズは不定なので）の配列としても良い。しかし、配列ではなく、行をダブル・リンクド・リストで繋げていったほうが、編集対象が大きい場合に挿入のコストが安い。

表示構造と実装構造が対応しているのでわかりやすい。しかし、ファイルの読み書き動作が行ごとになるので、行数が多いと重くなりがちだ。

コレ以外の実装は、この行リストではダメだから考え出されたもの。行リストで不自由しなければ、行リストでよい。

#### Gap Buffer

代表的な実装例はEmacs。

行を意識せずに、ファイル全体をメモリブロックに割りつける。ただしメモリブロックの大きさはファイル全体よりも大きい。カーソルがある所で、ファイル全体を2つに分けて、カーソルより前の部分をメモリブロックの前半に、カーソルより後の部分をメモリブロックの後半に割り当てる。カーソルの所にはファイルが割当たっていないギャップができる。

こうすることで、エディタにおいてもっとも頻繁に行われる「文字の挿入」はギャップの前端（前半のブロックの後端）に文字を追加するだけなので、非常に低コストで行うことができる。カーソルが移動した時は、その後に入力などの操作をする時に、移動した分のテキストブロックを、後半⇔前半で移動すればよい。これも比較的軽い。

「頻繁な操作は軽く、希な操作は重くても良い」というトレード・オフを具現化したデータ構造。プログラミング初心者のころに雑誌の記事で読んで、感心した記憶がある。

[https://en.wikipedia.org/wiki/Gap_buffer](https://en.wikipedia.org/wiki/Gap_buffer)

#### Rope / PieceTable

代表例はVS Code。

一端書かれたものは不変という追記型のデータ構造。StringではなくてRopeということらしい。

巨大なデータをハンドリングすることができる。

最初、ファイルは1つの塊としてメモリに割り当てられる。バッファの途中に挿入した時は、別のメモリを割り当て、挿入前半〜新規挿入部分〜挿入後半、のように別々のメモリをつなぎ合わせる形になる。これをテーブルで管理すればPiece Table、ツリーで管理すればRopeということ。

追記型なので、物理メモリではなく仮想メモリに割り当てておけば巨大ファイルも編集可能になるのだろうか。書き込み単位ごとにメモリ小片ができあがるのでUndoとの相性が良さそうだ。小片ごとに属性（構文ハイライトなど）も付けやすいというメリットもあるかもしれない。

[https://en.wikipedia.org/wiki/Piece_table](https://en.wikipedia.org/wiki/Piece_table)
[https://en.wikipedia.org/wiki/Rope_(data_structure)](https://en.wikipedia.org/wiki/Rope_(data_structure))

### 日本語の取り扱い（UTF-8）

今の時代、日本語・他言語対応はUTF-8に対応させておけばいいだろう。

Rustの場合、文字列を表すString型はUTF-8エンコードされたバイト列なので、Stringをそのまま使える。ただし、UTF-8は位置文字のバイト数が文字によって異なるので扱いが面倒だ。実装をよく理解して、適切なAPIを使いこなさなければならない。`String::chars().count()`は文字数を返すが、`String::len()`はバイト数を返す。こういった使い分けに注意すれば、絵文字などの多バイトUTF-8も、標準ライブラリがうまくハンドリングできる。同様に、`String`は`[]`で添字アクセスできないが`[..]`でスライスアクセスできる。スライスアクセスの時の添字はバイト列に対して働く。表示の文字幅を数えるためにはunicode-width crateが使える。

一般的に、一行の中にどのような文字（バイト数、表示幅）があるのかはわからない。行を移動したら、行の文字列の先頭からスキャンして、文字ごとのバイト数と表示幅の情報をキャッシュしておく。一行の長さはせいぜい100文字ぐらいなことが多いので現代のマシンではそれほど高コストにはならない。EditBufferのカーソルは文字数カウントで数え、表示用のカーソルは文字幅を考慮してカウントするようにする。

[https://doc.rust-jp.rs/book/second-edition/ch08-02-strings.html](https://doc.rust-jp.rs/book/second-edition/ch08-02-strings.html)

```rust
fn calc_line(&mut self) {
    self.cache_size = vec![];
    self.cache_width = vec![];
    for uni_c in self.buffer[self.cur_y].chars() {     // 行が変わるごとに行をスキャンする
        self.cache_size.push(uni_c.len_utf8());        // 各文字ごとのバイトサイズ
        self.cache_width.push(uni_c.width().unwrap()); // 各文字ごとの表示幅を調べてキャッシュしておく
    }
    self.cache_width.push(0); // dummy for newline
    self.cache_size.push(0);  // dummy for newline
}
```

### IME

未実装。これから。

## 開発の進め方

### インクリメンタルな開発

[Cコンパイラの作成](https://nkon.github.io/Compiler/)を通じて、すっかり、インクリメンタルな開発の信奉者になった。

最初に設計をしておかなければグチャグチャになる、とはよく言われる。しかし、テストしながら動作確認がきちんとできていればリファクタリングは怖くない。うまく改造できなければGitで戻れば良い。そして、大概のプログラマはそんなに馬鹿でないので、それほどグチャグチャにはならない。

それよりも、動いて動作が確認できている状態を維持しづつけることが大事。一気に大きなプログラムを作って、バグがあることを見つけ、バグの箇所を特定し、修正するのは、非常に時間のロスになる。

ある程度規模が大きな開発の場合は「設計が決まってないと…」ということもある。そういった場合でも、最初のうちは少人数の精鋭で集中して、コーディングの時間をきちんと確保して、書きながら設計を固めるのが良い。あなたのチームはエースに1か月集中してコーディングさせることはできるだろうか？動作状態を維持できるようになったら、人を増やして良い。

世の中の偉大なプログラム（LinuxやFirefroxなど）がエクセル仕様書からできあがったと思っている人はいないだろう。これらはみんなインクリメンタルに開発されている。

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

### Rustプロジェクトのディレクトリ構造

ほとんど標準化されている。

* `main.rs`には`fn main()`と必要最小限のみ書く。
* `lib.rs`では、必要なモジュールを読み込んでpublicにエクスポートする。
* それぞれのファイル（小文字.rs）には構造体（キャメルケース）と実装を定義し、必要なメソッド（スネークケース）を`pub`にする。

Rustのユニットテストはバイナリークレート（`fn main`を起点に実行される）には適用できない。ライブラリクレート（他の実行主体から呼ばれる）に対して適用される。同一のプロジェクトに対して、`main.rs`は、ほぼ`fn main()`のみを含む。それと並列に`lib.rs`を作り、ライブラリとして個々のモジュールを読み込む。

[https://doc.rust-jp.rs/book/second-edition/ch11-03-test-organization.html#a%E3%83%90%E3%82%A4%E3%83%8A%E3%83%AA%E3%82%AF%E3%83%AC%E3%83%BC%E3%83%88%E7%94%A8%E3%81%AE%E7%B5%90%E5%90%88%E3%83%86%E3%82%B9%E3%83%88]
(https://doc.rust-jp.rs/book/second-edition/ch11-03-test-organization.html#a%E3%83%90%E3%82%A4%E3%83%8A%E3%83%AA%E3%82%AF%E3%83%AC%E3%83%BC%E3%83%88%E7%94%A8%E3%81%AE%E7%B5%90%E5%90%88%E3%83%86%E3%82%B9%E3%83%88)

```
ked/
+ Cargo.toml
+ Cargo.lock
+ readme.md
+ src/
    + main.rs     バイナリクレートの起点。mainを含む。
    + lib.rs      テストしやすいようにするために、main.rs からモジュールを切り出し、lib.rsでまとめてライブラリとして扱えるようにする。
    + XXXXXXX.rs  個々のモジュール
+ tests/          結合テスト用のディレクトリ。
+ target/         コンパイラの生成物。.gitignore される。
    + 
```

## テスト

エディタはコーナーケースが多い。自動化テストを行いたい。ユニットテスト（内部）と結合テスト（外部）。ユニットテストはrustの標準機能を活用。結合テストのためにはエディタ自体にマクロ機能が必要。

`cargo test`ですべてのテストが実行される。テストがある開発環境は基本的人権に等しい。

### ユニットテスト

Rustは言語機能にユニットテストが統合されているため、簡単にユニットテストを行うことができる。各モジュールのファイルに、次のようにテストを記述していけば良い。関数単位で入出力を確認するのがユニットテストだ。

テストは、コードの共用化を考えずにベタ書きすることが多いので、テストコードは行数が多くなりがちだ。今回の例では`editbuffer.rs`の後半部分にユニットテストが書かれている。

```
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hogehoge() {
        assert_eq!(foo, bar);
    }
}
```

Doc-testとして関数のコメント中に入出力条件を記載してもよい。こちらは`cargo doc`でドキュメントを生成した時にも反映される。

### 結合テスト

統合テストを実施するための、アーキテクチャ設計が必要だ。rustでは結合テストとして、lib.rsに対するテスト機能を持っている。

ここでは、それ以外のプログラムの実行結果そのものを評価する外部テストについて考える。ここで作るエディタはマクロ機能を持っている。キー操作をマクロ機能として定義し、キー操作の結果として得られた編集結果を期待値と比較することでテストが可能だ。マクロ機能は、ユーザ機能としても使えるがテストに使いやすいように、というのが実装した主な理由だ。キーボードマクロをJSONファイルとして読み込んで、エディタが自動実行される。キーボードマクロによる編集結果が期待値と同じかどうかがテストの判定となる。MVCモデルがきちんとできていて、モデルのAPIが制定できていれば、キーボードマクロの実装は難しくない。APIを次々と呼ぶだけである。

RustではJSONファイルの読み込みはserdeというクレートを使って行われる。構造体に`Deserialize`属性をderiveしておいて`selde_json`で読みこめばよい。

```rust
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct MacroCommand {
    pub name: String,
    pub arg: usize,
    pub argstr: String,
}

let reader = std::io::BufReader::new(std::fs::File::open(script_file).unwrap());
let s: Vec<MacroCommand> = serde_json::from_reader(reader).unwrap();

`cargo test`としたときの自動実行について。まず`tests/`に適当なrustファイルを用意する。その関数に`#[tset]`属性を付けた関数を作成する。この関数が`cargo test`した時に呼ばれる。`std::process:Command`を利用してシェルスクリプトを実行する。その中でマクロでエディタを動作させ`diff`で期待値と比較する。NGだったら`assert`が失敗するようにしておけば、テストとしていい感じに実行される。

```rust
use std::fs;
use std::process::Command;

#[test]
fn find_dir_and_run() {
    println!("macro_tests_runner.rs");
    
    let target = "./tests/script/";
    for entry in fs::read_dir(target).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            let mut scr_file = path.display().to_string();
            scr_file.push_str("/run.sh");
            let status = Command::new(&scr_file).status().unwrap();
            assert!(status.success());
        }
    }
}
```

`macro.json`の例。見たまんんまだが、新規バッファを開き、`abcde`と入力し、ファイルに名前をつけて保存する。

```json
[{"name": "new_buffer","arg": 1,"argstr": ""},
 {"name": "insert_char","arg": 1,"argstr": "a"},
 {"name": "insert_char","arg": 1,"argstr": "b"},
 {"name": "insert_char","arg": 1,"argstr": "c"},
 {"name": "insert_char","arg": 1,"argstr": "d"},
 {"name": "insert_char","arg": 1,"argstr": "e"},
 {"name": "save_file_as","arg": 1,"argstr": "tests/script/test1/output.txt"}]
```

`tests/script/test1/run.sh`の例。マクロで生成された結果ファイル(`output.txt`)と期待値ファイル(`output_ok.txt`)を比較している。

```
#!/bin/sh

DIR=tests/script/test1
cargo run -- -s $DIR/macro.json 
diff $DIR/output.txt $DIR/output_ok.txt
if [ "$?" -eq 0 ]
then
    echo "OK"
    exit 0
else
    echo "******************** TEST FAIL *************************"
    exit 1
fi
```






### アサーション

テストで使われる`assert_eq!`や`assert!`マクロは通常の文脈でも使うことができる。しかしRustの場合はコンパイラが厳格なので、Cであれば`assert!`でチェックするようなところはコンパイラが警告してくれていることが多いという印象だ。たとえば配列のオーバーランなどは、assertを入れておかなくても、変に暴走せず即座に実行時エラーとなる。

## log

開発経緯については、GitHubのコミットログを参照していただきたい。その中でもマイルストーンとなるコミットについては[log.md](log.md) に記録しておく。これは実作業の記録なので、まとめとしては、この文書のほうがまとまっている。ビルドアップスタイルを追体験したいなら、コミットをたどっていってもおもしろいかもしれない。

[log.mg](log.md)

