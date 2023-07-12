use rand::Rng;
use std::cmp::Ordering;
use std::io;

fn main() {
    println!("Guess the number!");

    let secret_number = rand::thread_rng().gen_range(1..101);

    loop {
        println!("Please input your guess");

        let mut guess = String::new();

        // Result 型は列挙型(enum)
        // 列挙型は決められた列挙子(variant)を持つ
        // Result 型の列挙子は Ok か Err
        // expect メソッドは read_line が Err を返した場合にプログラムをクラッシュさせて引数で与えられたメッセージを表示する
        // Ok を返した場合はそのまま結果を返す
        // expect を省くとコンパイルエラーになる
        io::stdin()
            .read_line(&mut guess)
            .expect("Failed to read line");

        // 指定した型にパースする
        // let guess: u32 = guess.trim().parse().expect("Please type a number!");

        // 指定した型にパースする
        // 正常にパースできなくてもクラッシュさせずにループを継続させる
        let guess: u32 = match guess.trim().parse() {
            Ok(num) => num,
            Err(_) => continue, // _ で全ての型にマッチさせる = どんな型でも continue させる
        };

        println!("You guessed: {}", guess);

        match guess.cmp(&secret_number) {
            Ordering::Less => println!("Too small!"),
            Ordering::Greater => println!("Too big!"),
            Ordering::Equal => {
                println!("You win!");
                break;
            },
        }
    }
}
