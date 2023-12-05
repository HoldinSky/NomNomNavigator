use futures::StreamExt;
use mongodb::bson::Document;
use mongodb::Collection;

async fn test_mongo(client: &mongodb::Client) {
    let database = client.database("restaurant");
    let products: Collection<Document> = database.collection("products");

    let mut query = Document::new();

    let mut exists_filter = Document::new();
    exists_filter.insert("$exists", true);
    query.insert("amount_kg", exists_filter);

    let mut cursor = products.find(query, None).await.unwrap();

    while let Some(result) = cursor.next().await {
        match result {
            Ok(document) => {
                println!("Found document: {:?}", document);
            }
            Err(e) => println!("An error occurred: {e}"),
        }
    }
}
