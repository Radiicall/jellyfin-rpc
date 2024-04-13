use crate::prelude::{config::*, Content, MediaType};
use std::env;

#[test]
fn load_example_config() {
    let example = Config {
        jellyfin: Jellyfin {
            url: "https://example.com".to_string(),
            api_key: "sadasodsapasdskd".to_string(),
            username: Username::String("your_username_here".to_string()),
            music: Some(Music {
                display: Some(Display::String("genres".to_string())),
                separator: Some("-".to_string()),
            }),
            blacklist: Some(Blacklist {
                media_types: Some(vec![
                    MediaType::Music,
                    MediaType::Movie,
                    MediaType::Episode,
                    MediaType::LiveTv,
                ]),
                libraries: Some(vec!["Anime".to_string(), "Anime Movies".to_string()]),
            }),
            self_signed_cert: Some(false),
            show_simple: Some(false)
        },
        discord: Some(Discord {
            application_id: Some("1053747938519679018".to_string()),
            buttons: Some(vec![
                Button {
                    name: "dynamic".to_string(),
                    url: "dynamic".to_string(),
                },
                Button {
                    name: "dynamic".to_string(),
                    url: "dynamic".to_string(),
                },
            ]),
            show_paused: Some(true),
        }),
        imgur: Some(Imgur {
            client_id: Some("asdjdjdg394209fdjs093".to_string()),
        }),
        images: Some(Images {
            enable_images: Some(true),
            imgur_images: Some(true),
        }),
    };

    let config =
        Config::load(&(env::var("CARGO_MANIFEST_DIR").unwrap() + "/example.json")).unwrap();

    assert_eq!(example, config);
}

#[test]
#[should_panic]
fn try_get_content() {
    let config = Config {
        jellyfin: Jellyfin {
            url: "https://example.com".to_string(),
            api_key: "sadasodsapasdskd".to_string(),
            username: Username::String("your_username_here".to_string()),
            music: None,
            blacklist: None,
            self_signed_cert: None,
            show_simple: Some(false)
        },
        discord: None,
        imgur: None,
        images: None,
    };

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(Content::get(&config)).unwrap();
}

#[cfg(feature = "imgur")]
#[test]
#[should_panic]
fn try_imgur() {
    use crate::services::imgur;
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(imgur::Imgur::get(
        "",
        "",
        "",
        Some(env::var("CARGO_MANIFEST_DIR").unwrap() + "/example.json"),
        false,
    ))
    .unwrap();
}

#[test]
fn media_type_is_none() {
    let media_type_1 = MediaType::Movie;
    let media_type_2 = MediaType::None;

    assert_eq!(
        media_type_1.is_none() == false,
        media_type_2.is_none() == true
    )
}
