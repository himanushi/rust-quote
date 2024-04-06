macro_rules! add {
    () => { 0 };
    ($head:expr $(; $tail:expr)*) => { $head + add!($($tail);*) };
}

macro_rules! add_plus {
    // ベースケース: 引数がない場合は0を返す
    () => { 0 };

    // リテラル1つのみの場合はそれを返す
    ($head:expr) => { $head };

    // 2つ以上の引数を`+`で区切って処理する
    ($head:expr , $($tail:tt)+) => {
        $head + add_plus!($($tail)+)
    };
}

#[allow(unused)]
macro_rules! info {
    ($($x:tt)*) => (
        #[cfg(feature = "log")] {
            log::info!($($x)*);
        }
    )
}

fn main() {
    info!(42);
}
