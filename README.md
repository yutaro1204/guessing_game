# Rust Tutorial

## cargo

```bash
# 依存関係の解決
$ cargo build
# アプリケーションの実行
$ cargo run
# クレートのドキュメントをローカルでビルドしブラウザに表示する
$ cargo doc --open
```

## 所有権

### スタックとヒープ
https://doc.rust-jp.rs/book-ja/ch04-01-what-is-ownership.html

関数の引数やプリミティブな定数などはスタックで管理される。
動的な変数はヒープで管理される。

例えば、文字列リテラルはスタック、String 型はヒープとなる。

Rust ではスコープの終了地点でメモリが解放される。
Rust には GC が存在しない。

具体的には、変数がスコープを抜ける際、Rust は drop 関数を呼び出してメモリの割り当てを解放する。
C++ ではこのような要素の生存期間の終了地点でリソースを解放するパターンを RAII(Resource Acquisition Is Initialization) と呼ぶ。
```rust
{
    let s = String::from("hello"); // sはここから有効になる

    // sで作業をする
}                                  // このスコープはここでおしまい。sはもう有効ではない
```

### 所有権の移動
https://doc.rust-jp.rs/book-ja/ch04-01-what-is-ownership.html

Rust ではヒープメモリを利用するような変数を再代入した場合、そのデータまでを deep copy しない。
再代入時には最初に定義した変数のポインターと要素の長さ（その変数の中身が現在使用しているメモリ量をバイトで表したもの）と許容量（変数が OS から受け取った全メモリ量をバイトで表したもの）のみをコピー、つまり shallow copy のような挙動を取る。
```rust
let s1 = String::from("hello");
let s2 = s1; // ポインターと要素の長さと許容量のみコピー
```
そうなると、再代入時に同じヒープメモリのデータを指すポインターが複製されることになるが、この状態でこれらの変数がスコープを抜けると、Rust の仕様としてヒープメモリのデータが解放されてしまうことになり、二重解放エラーが発生してしまうことになる。
メモリの二重解放エラーはメモリ崩壊を導き、セキュリティ上の脆弱性を孕む危険性がある。
Rust ではこのような場合のために、最初に定義された変数が再代入された時点で前者が無効になったと判断し、スコープを抜けた際には後者のメモリのみを解放するように設計されている。
```rust
let s1 = String::from("hello"); // 無効と判断される
let s2 = s1; // s2 に所有権がムーブされる
```
Rust ではこのように先に定義された変数を再代入した場合の挙動を shallow copy と呼ぶ代わりに所有権のムーブと呼ぶ（ポインターなどの情報が移動するようなイメージ）。
```rust
let s1 = String::from("hello");
let s2 = s1;

println!("{}, world!", s1); // エラー
```

しかし、Rust でもヒープメモリのデータを再代入した時に deep copy したい場合があるかもしれない。
その場合は、clone メソッドを利用する。

また、スタックの場合はこのようなムーブはそもそも発生しない。

また Copy トレイト に適合している型の場合、再代入後も最初に定義された変数はその後も使用することができる。
ただし、Drop トレイトを実装している型の場合はこの影響を受けない。

### 所有権と関数
https://doc.rust-jp.rs/book-ja/ch04-01-what-is-ownership.html

関数に渡した引数は、その時点で所有権が関数のスコープにムーブされる。
つまり、関数に入る前のスコープでは、関数に入った時点でその変数は無効となる。
```rust
fn main() {
    let s = String::from("hello");  // sがスコープに入る
    takes_ownership(s);             // sの値が関数にムーブされ...
                                    // ... ここではもう有効ではない
    let x = 5;                      // xがスコープに入る
    makes_copy(x);                  // xも関数にムーブされるが、
                                    // i32はCopyなので、この後にxを使っても
                                    // 大丈夫
} // ここでxがスコープを抜け、sもスコープを抜ける。ただし、sの値はムーブされているので、何も特別なことは起こらない。
fn takes_ownership(some_string: String) { // some_stringがスコープに入る。
    println!("{}", some_string);
} // ここでsome_stringがスコープを抜け、`drop`が呼ばれる。後ろ盾してたメモリが解放される。
fn makes_copy(some_integer: i32) { // some_integerがスコープに入る
    println!("{}", some_integer);
} // ここでsome_integerがスコープを抜ける。何も特別なことはない。
```

元のスコープで所有権を引き継がせたい場合は、関数の戻り値を再代入することで、元のスコープにムーブさせることができる。

```rust
fn main() {
    let s1 = gives_ownership();         // gives_ownershipは、戻り値をs1に
                                        // ムーブする
    let s2 = String::from("hello");     // s2がスコープに入る
    let s3 = takes_and_gives_back(s2);  // s2はtakes_and_gives_backにムーブされ
                                        // 戻り値もs3にムーブされる
} // ここで、s3はスコープを抜け、ドロップされる。s2もスコープを抜けるが、ムーブされているので、
  // 何も起きない。s1もスコープを抜け、ドロップされる。
fn gives_ownership() -> String {             // gives_ownershipは、戻り値を
                                             // 呼び出した関数にムーブする
    let some_string = String::from("hello"); // some_stringがスコープに入る
    some_string                              // some_stringが返され、呼び出し元関数に
                                             // ムーブされる
}
// takes_and_gives_backは、Stringを一つ受け取り、返す。
fn takes_and_gives_back(a_string: String) -> String { // a_stringがスコープに入る。
    a_string  // a_stringが返され、呼び出し元関数にムーブされる
}
```

だが、このように元のスコープで所有権を継続的に維持させたい場合に、いちいち関数から引数で渡した値を戻り値として取得し直すのは非常に面倒である。
Rust ではこれを参照の機能で解決する。

### 参照と借用
https://doc.rust-jp.rs/book-ja/ch04-02-references-and-borrowing.html

関数に引数を渡したいときに、その関数が終了した後も引数となった変数を使いまわしたいが、そのままでは関数のスコープにおける引数に所有権がムーブしてしまい、元のスコープで引数となった変数が無効になってしまう。
これを有効なまま維持するには、前項でも触れたように関数から再度元のスコープへ所有権をムーブすれば良いのだが、もっと賢い方法がある。
それが参照を関数の引数に渡すことである。

以下の例では calculate_length 関数に s1 の参照を渡している。
参照は &s1 のように変数名の前にアンパサンドをつける。

```rust
fn main() {
    let s1 = String::from("hello");
    let len = calculate_length(&s1);
    // '{}'の長さは、{}です
    println!("The length of '{}' is {}.", s1, len);
}
fn calculate_length(s: &String) -> usize {
    s.len()
}
```

引数を受け取る関数側でも、引数の型定義にアンパサンドをつけている。
こうすることで、main 関数スコープにおける変数 s1 の所有権は　calculate_length 関数スコープの引数 s へムーブせず、main 関数スコープに留まり続ける。
そのため、calculate_length 関数の呼び出し後も有効で、println! の引数にも利用できている。
また、このような参照渡しを借用と呼ぶ。

借用した参照は基本的には不変であり、変更を加えることができない。
したがって、以下のコードではエラーが発生する。

```rust
fn main() {
    let s = String::from("hello");
    change(&s);
}
fn change(some_string: &String) {
    some_string.push_str(", world"); // 変更を加えようとするとエラーが発生する
}
```

しかし、このような参照にも変更を加えたい時がある。
以下のコードでは change 関数で引数に対して変更を加えているが、エラーにはならない。

```rust
fn main() {
    let mut s = String::from("hello");
    change(&mut s);
}
fn change(some_string: &mut String) {
    some_string.push_str(", world");
}
```

引数を渡すときと受け取るときに &mut を宣言してあげることで、これが可変な参照であることを伝えることができる。
ただし、このような可変な参照は特定のスコープにおける特定のデータに対しては一つまでしか持たせることができない。
つまり、以下のように一つの変数 s に対して複数の可変参照を作ることはできない。

```rust
let mut s = String::from("hello");

let r1 = &mut s;
let r2 = &mut s;

println!("{}, {}", r1, r2);
```

この制約によってコンパイル時にデータ競合を防ぐことができる。
データ競合とは以下の条件を満たすときに発生する、問題の特定などが難しくなるような問題である。
- 2つ以上のポインタが同じデータに同時にアクセスする。
- 少なくとも一つのポインタがデータに書き込みを行っている。
- データへのアクセスを同期する機構が使用されていない。

rust ではこのような問題が発生しないように、コンパイルの時点でこのような状況を見つけ次第エラーを発生させる。
だが、もちろん、スコープが変われば可変参照をいくら作っても問題はない。

```rust
let mut s = String::from("hello");
{
    let r1 = &mut s;
} // r1はここでスコープを抜けるので、問題なく新しい参照を作ることができる
let r2 = &mut s;
```

また、不変な参照と可変な参照について、これらを両立させることはできない。
複数の普遍な参照を同時に持つことはできるが、可変な参照の場合はデータが変わる恐れがあるので、不変参照と両立することができなし。
つまり、以下のコードはエラーで失敗する。

```rust
let mut s = String::from("hello");

let r1 = &s; // 問題なし
let r2 = &s; // 問題なし
let r3 = &mut s; // 大問題！
```

### ダングリングポインタ
https://doc.rust-jp.rs/book-ja/ch04-02-references-and-borrowing.html

ダングリングポインタとは、有効なオブジェクトを指していないポインターのことで、以下のコードにおける関数 dangle の戻り値 &s を指す。

```rust
fn main() {
    let reference_to_nothing = dangle();
}
fn dangle() -> &String {
    let s = String::from("hello");
    &s
}
```

dangle 関数の戻り値 &s の実体 s は当該関数を抜けると無効になるので、その参照が main 関数の引数に渡されたところで実体は失われたままである。
このようなコードではエラーが発生する。
こういった場合は実体そのものを戻り値に指定してあげることで、main 関数の変数へ所有権をムーブさせれば良い。

### スライス
https://doc.rust-jp.rs/book-ja/ch04-03-slices.html

配列や文字列の参照となる。
それらの要素から特定の添字の要素を抽出したい場合に使うのだろうか。
また、Rust では配列は固定長となり、スタックで管理されるので、可変長の場合はスライスを利用するのか。

### おさらい

#### 所有権とスコープ
所有権はスコープ内で有効であり、スコープを抜けると無効となる。
#### 所有権のムーブ
所有権は他の変数へ再代入されたり、関数の引数で渡された場合に無効となる。
このように所有権が他の変数に移ることをムーブと呼ぶ。
#### 所有権と関数
関数の引数として変数を渡すと、所有権が元のスコープから関数のスコープへわたる。
関数に渡された引数の所有権はその関数が終了するまで有効である。
関数が終了した後、元のスコープでは引数となった変数は無効となる。
元のスコープで変数を有効なまま利用するには、関数から戻り値を受け取り、それを元のスコープの変数に代入する（所有権のムーブを行う）必要がある。
#### 参照と借用
関数から戻り値を返す方法以外で元のスコープにおける変数を有効（所有権を維持）にしたままにする方法としては、関数への参照渡しがある。
参照私を行うと、関数へは参照のみが渡るので、元のスコープでの引数の本体の所有権はそのまま維持される。
このように関数が参照を引数として受け取ることを借用と呼ぶ。
#### 不変参照と可変参照
参照は基本的には不変であり変更ができないが、可変を明示的に宣言することで可変参照とすることもできる。


## 構造体
https://doc.rust-jp.rs/book-ja/ch05-00-structs.html

基本的な構造体の使い方は以下のスニペットの通り。

```rust
struct User {
    username: String,
    email: String,
    sign_in_count: u64,
    active: bool,
}
// 構造体での変数定義
let mut user1 = User {
    email: String::from("someone@example.com"),
    username: String::from("someusername123"),
    active: true,
    sign_in_count: 1,
};
// 値の更新
user1.email = String::from("anotheremail@example.com");

// 他の同型の構造体の要素を使いまわしつつ、部分的に定義
let user2 = User {
    email: String::from("another@example.com"),
    username: String::from("anotherusername567"),
    ..user1
};

// タプル形式の構造体
struct Color(i32, i32, i32);
struct Point(i32, i32, i32);
let black = Color(0, 0, 0);
let origin = Point(0, 0, 0);
```

また、構造体のフィールドにはスライスを定義できない。

### 構造体を利用した関数定義例
```rust
fn main() {
    let rect1 = (30, 50);
    println!(
        "The area of the rectangle is {} square pixels.",
        area(rect1)
    );
}
fn area(dimensions: (u32, u32)) -> u32 {
    dimensions.0 * dimensions.1
}
```
```rust
struct Rectangle {
    width: u32,
    height: u32,
}
fn main() {
    let rect1 = Rectangle { width: 30, height: 50 };
    println!(
        "The area of the rectangle is {} square pixels.",
        area(&rect1)
    );
}
fn area(rectangle: &Rectangle) -> u32 {
    rectangle.width * rectangle.height
}
```

### 構造体のデバッグ

以下のようにコーディングすると println! でエラーが発生する。

```rust
struct Rectangle {
    width: u32,
    height: u32,
}
fn main() {
    let rect1 = Rectangle { width: 30, height: 50 };
    // rect1は{}です
    println!("rect1 is {}", rect1); // エラー
}
```

このような場合は、以下のように #[derive(Debug)] 注釈を構造体の上に追加することで正常に出力を得られる。

```rust
#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}
fn main() {
    let rect1 = Rectangle { width: 30, height: 50 };
    println!("rect1 is {:?}", rect1);
}
```

### 構造体のメソッド定義

メソッドとは特定の構造体に属する一連の処理である。
関数とメソッドは異なる。
関数は構造体には属しない。

以下のように構造体にメソッドを定義することができる。

```rust
#[derive(Debug)]
struct Rectangle {
    width: u32,
    height: u32,
}
impl Rectangle {
    fn area(&self) -> u32 {
        self.width * self.height
    }
}
fn main() {
    let rect1 = Rectangle { width: 30, height: 50 };
    println!(
        "The area of the rectangle is {} square pixels.",
        rect1.area()
    );
}
```

基本的にメソッドの self は &self か &mut self を利用する。
self を利用するのは稀である（積極的に使わない）。

### 構造体の関連関数

関数とはいうものの、関連関数も構造体に属する。
関連関数は引数に self を取らない関数である。
呼び出しには :: を利用する。

```rust
impl Rectangle {
    fn square(size: u32) -> Rectangle {
        Rectangle { width: size, height: size }
    }
}

let sq = Rectangle::square(3);
```

String::from も関連関数である。

## Enum

以下では Enum を定義し、それを元に変数を定義している。

```rust
enum IpAddr {
    V4(u8, u8, u8, u8),
    V6(String),
}

let home = IpAddr::V4(127, 0, 0, 1);

let loopback = IpAddr::V6(String::from("::1"));
```

### Option Enum

https://doc.rust-jp.rs/book-ja/ch06-01-defining-an-enum.html#option-enum%E3%81%A8null%E5%80%A4%E3%81%AB%E5%8B%9D%E3%82%8B%E5%88%A9%E7%82%B9

Rust には null が存在ないが、Option<T> によって存在しない状態を表現することができる。

```rust
enum Option<T> {
    Some(T),
    None,
}
```

Option<T> は T とは異なるので、両者を足し合わせるなどといった処理はできない。
Option<T> が T として認められるためには、それを T へ変換する必要がある。
そうすることで、T の存在有無に従った処理の分岐などを捌くことができるようになる。

https://doc.rust-jp.rs/book-ja/ch06-02-match.html

Option<T> を T へ変換して値を操作するには Enum を利用することができる。

```rust
fn plus_one(x: Option<i32>) -> Option<i32> {
    match x {
        None => None,
        Some(i) => Some(i + 1), // Some に match すれば i で取り出して加算ができる
    }
}
let five = Some(5);
let six = plus_one(five);
let none = plus_one(None);
```

Option<T> を T に変換するには、unwrap_or() メソッドも利用できる。

```rust
fn plus_one(x: Option<i32>) -> i32 {
    x.unwrap_or(0) + 1
}
```
https://stackoverflow.com/questions/62811196/type-casting-for-option-type

### match vs if let

Enum の列挙子を判別するために match を利用することができるが、決められた特定のパターンのみにマッチするかどうかを確認したいだけのために match を利用するのは冗長な場合がある。
例えば以下のように、3 のみにマッチしたい場合に match を利用するのは冗長な感がある。

```rust
let some_u8_value = Some(0u8);
match some_u8_value {
    Some(3) => println!("three"),
    _ => (),
}
```

ちなみに _ は else に該当する。
このようなパターンには if let を活用することができる。
以下はロジックとしては同じである。

```rust
if let Some(3) = some_u8_value {
    println!("three");
}
```

## パッケージング

https://doc.rust-jp.rs/book-ja/ch07-00-managing-growing-projects-with-packages-crates-and-modules.html

### パッケージとクレート

Rust プロジェクト構造は概ね以下の関係で成り立っている。

パッケージ > クレート > モジュール > 関数、構造体など

パッケージはある機能群を提供する一つ以上のクレートを要するが、ここには0個か1個のライブラリークレートと1個以上のバイナリークレートを持っている必要がある。
`cargo new my-project` を実行した時に生成される src/main.rc はバイナリークレートと呼ばれる。
src/lib.rc を作成すると、これはライブラリークレートと見なされる。
これらの src ディレクトリに配置されたクレートはルートクレートである。
バイナリークレートは src/bin ディレクトリ配下にも複数配置することができる。

### モジュール

以下はモジュールの定義例である。

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}

        fn seat_at_table() {}
    }
    mod serving {
        fn take_order() {}

        fn serve_order() {}

        fn take_payment() {}
    }
}
```

上記のモジュール front_of_house は、同一のファイルで定義された以下の関数 eat_at_restaurant から絶対パスと相対パスを利用して呼び出すことができる。

```rust
pub fn eat_at_restaurant() {
    // 絶対パス
    crate::front_of_house::hosting::add_to_waitlist();
    // 相対パス
    front_of_house::hosting::add_to_waitlist();
}
```

絶対パスはクレートルートから辿るパスとなり、相対パスはこの呼び出しが行われる関数を基準としたパスとなる。
基本的には絶対パスをデフォルトで利用し、必要に迫られた場合には相対パスを使うようにする。

また、Rust ではあらゆる要素が標準では非公開となるので、モジュールや関数もその例外ではない。
従って pub で公開しなければ、上記のパスからモジュールを呼び出すことはできない。
上記では、front_of_house::hosting::add_to_waitlist はモジュールも関数も pub となっているので呼び出すことができる。

ちなみに、eat_at_restaurant は front_of_house と同じファイル、つまりモジュール内に定義されている前提なので、この場合、このモジュールと関数は兄弟関係となり、互いに pub 抜きで参照することができるようになっている。

#### 公開についての余談

構造体の場合も同様に pub で定義することで公開することができるが、そのままでは中のフィールドは公開されない。
Rust ではあらゆる要素が標準では非公開となっているが、構造体のフィールドもその例外ではない。

```rust
mod back_of_house {
    pub struct Breakfast { // 公開
        pub toast: String, // 公開
        seasonal_fruit: String, // 非公開
    }
}
```

Enum の場合は、それ自体を公開すると、中の要素も全て公開されるようになっている。

```rust
mod back_of_house {
    pub enum Appetizer { // 公開
        Soup, // 公開
        Salad, // 公開
    }
}
pub fn eat_at_restaurant() {
    let order1 = back_of_house::Appetizer::Soup;
    let order2 = back_of_house::Appetizer::Salad;
}
```

#### モジュールの use

絶対パスや相対パスを都度利用するのはその実面倒であるが、use によってモジュールをもっと簡単に扱えるようになる。
use を利用することで指定したモジュールを同じスコープに持ってくることができるようになるのである（ちょうど、同じファイルに定義した兄弟のように簡単に扱うことができる）。

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

use crate::front_of_house::hosting;

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist();
    hosting::add_to_waitlist();
    hosting::add_to_waitlist();
}
```

上記のソースコードでは front_of_house crate::front_of_house::hosting を use することで、eat_at_restaurant 関数ないでは hosting から参照できるようになっている。
つまり、front_of_house を明示的に参照することなく、直接 hosting を指定することができるようになっているわけである。

また、上記では絶対パスで use を行っているが、相対パスでも可能である。

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

use self::front_of_house::hosting; // self:: から相対パスで辿る

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist();
    hosting::add_to_waitlist();
    hosting::add_to_waitlist();
}
```

ここで hosting モジュールを use して add_to_waitlist メソッドを直接 use していないのは、ただの慣例である。
公開か非公開かを厳密に制御したい場合やその他の明確な理由を除けば、基本的にはモジュールを use すべきだろう。
ただし、構造体や Enum は基本的にフルパスで use するのが慣例である。

また、use したパスが重複するようなケースが発生した場合、as を利用して参照名にエイリアスを与えることができる。

```rust
use std::fmt::Result;
use std::io::Result as IoResult; // エイリアス

fn function1() -> Result {
    // --snip--
}

fn function2() -> IoResult<()> {
    // --snip--
}
```

#### 再公開

デフォルトでは use されたモジュール等はそのスコープではあたかも同一のスコープに定義されたものとして動作するが、別のスコープから呼び出す場合は再公開を行う必要がある。

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

use crate::front_of_house::hosting;

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist();
    hosting::add_to_waitlist();
    hosting::add_to_waitlist();
}
```

上記のコードでは、同一ファイル内では hosting::add_to_waitlist で呼び出すことができるものの、外部ファイルからは同じ形で呼び出すことはできないため、そのファイルから絶対パスや相対パスで辿る必要がある。
しかし、pub use で公開することで外部ファイルからも参照することができるようになる。
この手法は再公開と呼ばれている（つまり、ファイル内のスコープで use で公開したものを今度はファイル外のスコープにも公開したということで再公開と呼ばれているわけである）。

#### 外部パッケージの利用

外部パッケージを利用するのにも use は使われる。

```rust
use rand::Rng;

fn main() {
    let secret_number = rand::thread_rng().gen_range(1..101);
}
```

#### パスの整理

同一のクレートやモジュール内で複数定義された要素を利用したい場合に、それぞれを記述するのは骨が折れる作業となりうる。
以下のようにひとまとめに波括弧でまとめることができる。

```rust
// before
use std::cmp::Ordering;
use std::io;

// after
use std::{cmp::Ordering, io};
```

また、use したモジュール自身もまとめたい場合には、self を利用することができる。

```rust
// before
use std::io;
use std::io::Write;


// after
use std::io::{self, Write};
```

#### glob 演算子

glob 演算子(`*`)を利用することで、それ以下のすべての要素をスコープに持ち込むことができる。

```rust
use std::collections::*;
```

### モジュールのファイル分割

以下のようなソースコードのモジュールをファイルに分割するにはどうすれば良いか。

```rust
mod front_of_house {
    pub mod hosting {
        pub fn add_to_waitlist() {}
    }
}

pub use crate::front_of_house::hosting;

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist();
    hosting::add_to_waitlist();
    hosting::add_to_waitlist();
}
```

以下は分割した例である。
mod front_of_house だけは残し、あとは front_of_house.rs にその中身を定義している。
src/lib.rs の mod front_of_house の後ろにセミコロンをつけることで、この中身を別のファイルから読み取ろうとするのだが、その中身が front_of_house.rs に定義されているわけである。

```rust
// src/lib.rs
mod front_of_house;

pub use crate::front_of_house::hosting;

pub fn eat_at_restaurant() {
    hosting::add_to_waitlist();
    hosting::add_to_waitlist();
    hosting::add_to_waitlist();
}

// src/front_of_house.rs
pub mod hosting {
    pub fn add_to_waitlist() {}
}
```

同様に src/front_of_house.rs の中身を分割しようとすれば、以下のようになるだろう。

```rust
// src/front_of_house.rs
pub mod hosting;

// src/front_of_house/hosting.rs
pub fn add_to_waitlist() {}
```

## コレクション

## エラー処理

## ジェネリック型

## トレイト

## ライフタイム

## 自動テスト
