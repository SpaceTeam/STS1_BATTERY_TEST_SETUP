#![cfg_attr(not(test), no_std)]

#[cfg(test)]
mod tests {
    use super::*;
    use transmission::add;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
