macro_rules! add {
    () => { 0 };
    ($head:expr $(; $tail:expr)*) => { $head + add!($($tail);*) };
}

fn main() {
    let sum = add!(1; 1; 1);
    println!("The sum is: {}", sum);
}
