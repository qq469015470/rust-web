#[cfg(test)]
mod tests {
    #[test]
    fn test_1() {
        let a = rust_web::web::JsonType::Null;

        assert_eq!(String::from(&a), "null");
    }
}
