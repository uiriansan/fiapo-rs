use mangadex_api::CDN_URL;
use mangadex_api::v5::{MangaDexClient, schema::RelatedAttributes};
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

pub async fn get_r_manga() -> anyhow::Result<()> {
    let client = MangaDexClient::default();

    let random_manga = client.manga().random().get().send().await?;

    println!("{:?}", random_manga);

    Ok(())
}

use mangadex_api_types_rust::{Language, MangaSortOrder, OrderDirection};

#[derive(Clone)]
pub struct MangadexSearchData {
    pub id: Uuid,
    pub english_title: String,
    pub romaji_title: String,
    pub author: String,
    pub artist: String,
    pub cover_url: String,
}
impl MangadexSearchData {
    pub fn new(
        id: Uuid,
        english_title: String,
        romaji_title: String,
        author: String,
        artist: String,
        cover_url: String,
    ) -> Self {
        MangadexSearchData {
            id,
            english_title,
            romaji_title,
            author,
            artist,
            cover_url,
        }
    }
}

pub async fn search_manga(title: String) -> anyhow::Result<Vec<MangadexSearchData>> {
    // TODO: Handle errors and everything
    let search_future = async {
        let client = MangaDexClient::default();
        client
            .manga()
            .get()
            .title(title)
            .order(MangaSortOrder::FollowedCount(OrderDirection::Descending))
            .include(&mangadex_api_types_rust::ReferenceExpansionResource::Author)
            .include(&mangadex_api_types_rust::ReferenceExpansionResource::Artist)
            .include(&mangadex_api_types_rust::ReferenceExpansionResource::CoverArt)
            .send()
            .await
    };

    let manga_results = timeout(Duration::from_secs(5), search_future)
        .await
        .map_err(|_| anyhow::anyhow!("Search request timed out"))?
        .map_err(|e| anyhow::anyhow!("Search request failed: {}", e))?;

    let mut results: Vec<MangadexSearchData> = Vec::new();

    for (_, manga) in manga_results.data.iter().enumerate() {
        let id = &manga.id;
        let english_title = &manga.attributes.title[&Language::English];
        let mut romaji_title = "";
        for title in &manga.attributes.alt_titles {
            if let Some(j_title) = title.get(&Language::JapaneseRomanized) {
                romaji_title = j_title;
                break;
            }
        }
        let author = manga
            .find_first_relationships(mangadex_api_types_rust::RelationshipType::Author)
            .and_then(|e| {
                e.attributes.clone().map(|rel| match rel {
                    RelatedAttributes::Author(a) => a.name,
                    _ => "Author not found!".to_string(),
                })
            })
            .unwrap_or("Author not found!".to_string());
        let artist = manga
            .find_first_relationships(mangadex_api_types_rust::RelationshipType::Artist)
            .and_then(|e| {
                e.attributes.clone().map(|rel| match rel {
                    RelatedAttributes::Author(a) => a.name,
                    _ => "Artist not found!".to_string(),
                })
            })
            .unwrap_or("Artist not found!".to_string());

        // TODO: Cache covers?
        let cover_filename = manga
            .find_first_relationships(mangadex_api_types_rust::RelationshipType::CoverArt)
            .and_then(|e| {
                e.attributes.clone().map(|rel| match rel {
                    RelatedAttributes::CoverArt(a) => a.file_name,
                    _ => "Could not retrieve cover".to_string(),
                })
            })
            .unwrap_or("Could not retrieve cover".to_string());

        let cover_url = String::from(format!(
            "{}/covers/{}/{}.512.jpg",
            CDN_URL, &id, cover_filename
        ));

        results.push(MangadexSearchData::new(
            *id,
            english_title.to_string(),
            romaji_title.to_string(),
            author,
            artist,
            cover_url,
        ));
    }

    Ok(results)
}
