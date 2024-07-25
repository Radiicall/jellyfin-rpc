use crate::ClientBuilder;

#[test]
fn build_client_error() {
    let client = ClientBuilder::new().build();

    if let Ok(_) = client {
        panic!("client was constructed even though required values are missing!");
    }
}

#[test]
fn invalid_url() {
    let mut builder = ClientBuilder::new();
    builder
        .api_key("a1b2c3d4")
        .username("test")
        .url("url_without_base.com");

    let client = builder.build();

    if let Ok(_) = client {
        panic!("client constructed without a valid url!")
    }
}
