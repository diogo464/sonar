pub async fn create_context() -> sonar::Context {
    let config = sonar::Config::new(":memory:", sonar::StorageBackend::Memory);
    sonar::new(config).await.unwrap()
}

pub fn create_simple_genres() -> sonar::Genres {
    let mut genres = sonar::Genres::default();
    genres.set(&"heavy metal".parse().unwrap());
    genres.set(&"electronic".parse().unwrap());
    genres
}

pub fn create_simple_properties() -> sonar::Properties {
    let mut properties = sonar::Properties::default();
    properties.insert(
        sonar::PropertyKey::new_uncheked("key1"),
        sonar::PropertyValue::new_uncheked("value1"),
    );
    properties.insert(
        sonar::PropertyKey::new_uncheked("key2"),
        sonar::PropertyValue::new_uncheked("value2"),
    );
    properties
}
