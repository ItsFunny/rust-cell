use logsdk::{init_log, log};
use log::info;

struct AA {
    name: String,
}

macro_rules! add {
    ($a:expr,$b:expr)=>{
        $a+$b
    }
}

macro_rules! ttt {
    ($a:tt)=>{
        println!($a)
    }
}
// 需求: 因为rust 函数是没有可变长参数的,所以要用宏来代替,并且要支持kv
macro_rules! add_as {
    (
  // repeated block
  $($a:expr)
 // seperator
   ,
// zero or more
   *
   )=>{
       {
   // to handle the case without any arguments
   0
   // block to be repeated
   $(+$a)*
     }
    }
}


fn main() {
    init_log();
    info!("asdd");
    // let a = AA { name: String::from("asd") };
    // println!("{},{},{}", add!(1,2), add!(4,5), "asd");
    // println!("aaaa");
    // println!("{}",add_as!(1,2,3,4));
}