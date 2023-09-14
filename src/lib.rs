// pub fn add(left: usize, right: usize) -> usize {
//     left + right
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn it_works() {
//         let result = add(2, 2);
//         assert_eq!(result, 4);
//     }
// }



pub mod pid {
    pub mod builder;
    pub mod traits;
    pub mod sync;
    // #[cfg(feature = "tokio")]
    pub mod streaming;
}
pub mod error;
