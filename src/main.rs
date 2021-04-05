#[macro_use]
extern crate tantivy;
use rand::{distributions::Alphanumeric, Rng};
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::schema::*;
use tantivy::Index;
use tantivy::ReloadPolicy;
use tempfile::TempDir; // 0.8

fn rand_string(n: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(n)
        .map(char::from)
        .collect()
}

fn main() -> tantivy::Result<()> {
    let chars = 4;
    let n = 1_000_000;
    let index_path = TempDir::new()?;

    let mut schema_builder = Schema::builder();
    schema_builder.add_text_field("title", TEXT | STORED);
    schema_builder.add_text_field("body", TEXT);

    let schema = schema_builder.build();

    let index = Index::create_in_dir(&index_path, schema.clone())?;

    let mut index_writer = index.writer(50_000_000)?;

    let title = schema.get_field("title").unwrap();
    let body = schema.get_field("body").unwrap();

    for i in (0..n) {
        let bodys: String = (0..10)
            .map(|_| rand_string(chars))
            .collect::<Vec<_>>()
            .join(" ");
        index_writer.add_document(doc!(
            title => i.to_string(),
            body => bodys,
        ));
    }
    println!("commiting");
    index_writer.commit()?;
    println!("committed");

    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommit)
        .try_into()?;

    println!("opening searcher");
    let searcher = reader.searcher();
    println!("searcher opened");
    let query_parser = QueryParser::for_index(&index, vec![title, body]);

    loop {
        let mut query_string = String::new();
        println!("> ");
        std::io::stdin()
            .read_line(&mut query_string)
            .expect("failed to read line");

        let query = query_parser.parse_query(query_string.as_str())?;
        let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

        for (_score, doc_address) in top_docs {
            let retrieved_doc = searcher.doc(doc_address)?;
            println!(
                "{}: {}",
                query_string.trim(),
                schema.to_json(&retrieved_doc)
            );
        }
    }

    Ok(())
}
